use log::error;
use wasmer::{
    imports, AsStoreRef, Engine, Function, FunctionEnv, FunctionEnvMut, Instance, Memory,
    MemoryType, MemoryView, Module, Store, TypedFunction, ValueType, WasmPtr, WasmSlice,
};

use crate::{
    console::{self, pixel_width_of_flags, Console},
    utils,
    wasm4::{self, DRAW_COLORS_ADDR, FRAMEBUFFER_ADDR, FRAMEBUFFER_SIZE},
    Backend, Sink, Source,
};

pub struct WasmerBackend {
    fn_env: FunctionEnv<WasmerRuntimeEnv>,
    store: Store,
    instance: Instance,
}

impl WasmerBackend {
    pub fn new(wasm_bytes: &[u8], console: &Console) -> anyhow::Result<Self> {
        Self::precompiled(
            &Module::new(&Store::default(), wasm_bytes)?.serialize()?,
            console,
        )
    }

    pub fn precompiled(module_bytes: &[u8], console: &Console) -> anyhow::Result<Self> {
        let mut store = Store::new(Engine::headless());
        let module = unsafe { Module::deserialize(&store, module_bytes)? };

        // init memory and env
        let wasm_env = WasmerRuntimeEnv::new(&mut store, console.create_api())?;
        let fn_env = FunctionEnv::new(&mut store, wasm_env);

        // see https://wasm4.org/docs/reference/functions
        let imports = imports! {
            "env" => {
                "memory" => fn_env.as_mut(&mut store).memory.clone(),
                "trace" => Function::new_typed_with_env(&mut store, &fn_env, trace),
                "tracef" => Function::new_typed_with_env(&mut store, &fn_env, tracef),
                "traceUtf8" => Function::new_typed_with_env(&mut store, &fn_env, trace_utf8),
                "traceUtf16" => Function::new_typed_with_env(&mut store, &fn_env, trace_utf16),
                "blit" => Function::new_typed_with_env(&mut store, &fn_env, blit),
                "blitSub" => Function::new_typed_with_env(&mut store, &fn_env, blit_sub),
                "line" => Function::new_typed_with_env(&mut store, &fn_env, line),
                "hline" => Function::new_typed_with_env(&mut store, &fn_env, hline),
                "vline" => Function::new_typed_with_env(&mut store, &fn_env, vline),
                "oval" => Function::new_typed_with_env(&mut store, &fn_env, oval),
                "rect" => Function::new_typed_with_env(&mut store, &fn_env, rect),
                "text" => Function::new_typed_with_env(&mut store, &fn_env, text),
                "textUtf8" => Function::new_typed_with_env(&mut store, &fn_env, text_utf8),
                "textUtf16" => Function::new_typed_with_env(&mut store, &fn_env, text_utf16),
                "tone" => Function::new_typed_with_env(&mut store, &fn_env, tone),
                "diskr" => Function::new_typed_with_env(&mut store, &fn_env, diskr),
                "diskw" => Function::new_typed_with_env(&mut store, &fn_env, diskw),
            }
        };

        let instance = Instance::new(&mut store, &module, &imports)?;

        Ok(Self {
            fn_env,
            store,
            instance,
        })
    }
}

impl Backend for WasmerBackend {
    fn call_update(&mut self) {
        let view = self.fn_env.as_ref(&self.store).memory.view(&self.store);

        // clear the framebuffer (important)
        if 0 == wasm4::SYSTEM_PRESERVE_FRAMEBUFFER
            & view.read_u8(wasm4::SYSTEM_FLAGS_ADDR as u64).unwrap()
        {
            let slice = WasmPtr::<u8>::new(wasm4::FRAMEBUFFER_ADDR as u32)
                .slice(&view, FRAMEBUFFER_SIZE as u32)
                .unwrap();
            console::clear(&mut WasmSliceSinkSource { slice });
        }

        if let Ok(update) = self.instance.exports.get_function("update") {
            let typed: TypedFunction<(), ()> = update
                .typed(&self.store)
                .expect("update function incorrect type");
            typed.call(&mut self.store).expect("call update function");
        }
    }

    fn call_start(&mut self) {
        if let Ok(start) = self.instance.exports.get_function("start") {
            let typed: TypedFunction<(), ()> = start
                .typed(&self.store)
                .expect("start function incorrect type");
            typed.call(&mut self.store).expect("call start function");
        }
    }

    fn read_screen(&self, framebuffer: &mut [u8; wasm4::FRAMEBUFFER_SIZE], palette: &mut [u8; 16]) {
        let view = self.fn_env.as_ref(&self.store).memory.view(&self.store);

        view.read(wasm4::FRAMEBUFFER_ADDR as u64, framebuffer)
            .expect("read to framebuffer");
        view.read(wasm4::PALETTE_ADDR as u64, palette)
            .expect("read to palette");
    }

    fn read_system_flags(&self) -> u8 {
        let view = self.fn_env.as_ref(&self.store).memory.view(&self.store);
        view.read_u8(wasm4::SYSTEM_FLAGS_ADDR as u64).unwrap()
    }

    fn set_gamepad(&mut self, gamepad: u32) {
        let view = self.fn_env.as_ref(&self.store).memory.view(&self.store);

        view.write(
            wasm4::GAMEPAD1_ADDR as u64,
            bytemuck::cast_slice(&[gamepad]),
        )
        .expect("write to GAMEPAD1_ADDR");
    }

    fn set_mouse(&mut self, x: i16, y: i16, buttons: u8) {
        let view = self.fn_env.as_ref(&self.store).memory.view(&self.store);

        view.write(wasm4::MOUSE_X_ADDR as u64, bytemuck::cast_slice(&[x]))
            .expect("write to MOUSE_X_ADDR");
        view.write(wasm4::MOUSE_Y_ADDR as u64, bytemuck::cast_slice(&[y]))
            .expect("write to MOUSE_X_ADDR");
        view.write(
            wasm4::MOUSE_BUTTONS_ADDR as u64,
            bytemuck::cast_slice(&[buttons]),
        )
        .expect("write to MOUSE_BUTTONS_ADDR");
    }

    fn write_save_cache(&mut self) -> Option<[u8; 1024]> {
        self.fn_env.as_mut(&mut self.store).write_save()
    }

    fn set_save_cache(&mut self, data: [u8; 1024]) {
        self.fn_env.as_mut(&mut self.store).api.save_cache.set(data);
    }
}

struct WasmerRuntimeEnv {
    memory: Memory,
    api: console::Api,
}

impl WasmerRuntimeEnv {
    pub fn new(store: &mut Store, api: console::Api) -> anyhow::Result<Self> {
        // this is important, it's all the memory that the game is allowed to use.
        let memory = Memory::new(store, MemoryType::new(1, Some(1), false))?;

        // set initial values of memory
        let mem_view = memory.view(store);
        mem_view.write(wasm4::PALETTE_ADDR as u64, &utils::default_palette())?;
        mem_view.write(
            wasm4::DRAW_COLORS_ADDR as u64,
            &utils::default_draw_colors(),
        )?;
        mem_view.write(
            wasm4::FRAMEBUFFER_ADDR as u64,
            &utils::default_framebuffer(),
        )?;

        Ok(Self { memory, api })
    }

    pub fn write_save(&self) -> Option<[u8; 1024]> {
        self.api.write_save()
    }
}

struct Context<'a> {
    view: MemoryView<'a>,
}

impl<'a> Context<'a> {
    fn from_env(env: &'a FunctionEnvMut<'a, WasmerRuntimeEnv>) -> Context<'a> {
        let view = env.data().memory.view(&env.as_store_ref());

        Self { view }
    }

    fn view(&'a self) -> &'a MemoryView<'a> {
        &self.view
    }

    fn fb(&self) -> WasmSliceSinkSource<u8> {
        let slice = WasmPtr::<u8>::new(FRAMEBUFFER_ADDR as u32)
            .slice(&self.view, FRAMEBUFFER_SIZE as u32)
            .unwrap();

        WasmSliceSinkSource { slice }
    }

    fn draw_colors(&self) -> u16 {
        WasmPtr::<u16>::new(DRAW_COLORS_ADDR as u32)
            .read(&self.view)
            .unwrap()
    }
}

struct WasmSliceSinkSource<'a, T>
where
    T: ValueType + Copy,
{
    slice: WasmSlice<'a, T>,
}

impl<'a, T> Source<T> for WasmSliceSinkSource<'a, T>
where
    T: ValueType + Copy,
{
    fn item_at(&self, offset: usize) -> T {
        self.slice.index(offset as u64).read().unwrap()
    }
}

impl<'a, T> Sink<T> for WasmSliceSinkSource<'a, T>
where
    T: ValueType + Copy,
{
    fn set_item_at(&mut self, offset: usize, item: T) {
        self.slice
            .index(offset as u64)
            .write(item)
            .expect("writing to wasm memory failed");
    }

    fn fill(&mut self, item: T) {
        for n in 0..FRAMEBUFFER_SIZE as u64 {
            self.slice.write(n, item).unwrap();
        }
    }
}

fn trace(env: FunctionEnvMut<WasmerRuntimeEnv>, ptr: WasmPtr<u8>) {
    let ctx = Context::from_env(&env);

    // WASM4's trace is "supposed" to just be ASCII but UTF-8
    // is a superset of ASCII so this should be fine.
    let str = match ptr.read_utf8_string_with_nul(ctx.view()) {
        Ok(str) => str,
        Err(err) => {
            error!(
                "Failed to read null-terminated string at {:X?}: {err}",
                ptr.offset()
            );
            return;
        }
    };

    println!("{str}");
}

fn tracef(env: FunctionEnvMut<WasmerRuntimeEnv>, fmt: WasmPtr<u8>, args: WasmPtr<u8>) {
    let ctx = Context::from_env(&env);

    let mut fmt_offset = fmt.offset();
    let mut arg_offset = args.offset();

    let mut output = String::new();

    while let Ok(ch) = ctx.view().read_u8(fmt_offset as u64) {
        fmt_offset += 1;

        // null-terminated string
        if ch == 0 {
            break;
        }

        // character other than formatting character '%' is just added.
        if ch != 37 {
            output.push(char::from_u32(ch as u32).unwrap_or('!'));
            continue;
        }

        // if we've hit the formatting character ('%') then move to the next
        // char and check to see what it is. Then replace it.
        if let Ok(ch) = ctx.view().read_u8(fmt_offset as u64) {
            match ch {
                // 'c' - character
                99 => {
                    let mut val: [u8; 4] = [0; 4];

                    match WasmPtr::<u8>::new(arg_offset).slice(ctx.view(), 4) {
                        Ok(slice) => match slice.read_slice(&mut val) {
                            Ok(_) => (),
                            Err(err) => {
                                error!("failed to read char WasmSlice to slice");
                                continue;
                            }
                        },
                        Err(err) => {
                            error!("failed to read WasmSlice from arg_offset");
                            continue;
                        }
                    }

                    output.push(char::from_u32(bytemuck::cast(val)).unwrap_or('!'));
                    arg_offset += 4;
                }
                // 'd'/'x' - integer/hex
                100 | 120 => {
                    let mut val: [u8; 4] = [0; 4];

                    match WasmPtr::<u8>::new(arg_offset).slice(ctx.view(), 4) {
                        Ok(slice) => match slice.read_slice(&mut val) {
                            Ok(_) => (),
                            Err(err) => {
                                error!("failed to read int WasmSlice");
                                continue;
                            }
                        },
                        Err(err) => {
                            error!("failed to read WasmSlice from arg_offset");
                            continue;
                        }
                    }

                    output.push_str(&i32::from_le_bytes(val).to_string());
                    arg_offset += 4;
                }
                // 's' - null terminated string
                115 => {
                    let mut ptr_bytes: [u8; 4] = [0; 4];

                    match WasmPtr::<u8>::new(arg_offset).slice(ctx.view(), 4) {
                        Ok(slice) => match slice.read_slice(&mut ptr_bytes) {
                            Ok(_) => (),
                            Err(err) => {
                                error!("failed to read str WasmSlice");
                                continue;
                            }
                        },
                        Err(err) => {
                            error!("failed to read WasmSlice from arg_offset");
                            continue;
                        }
                    }

                    let str = match WasmPtr::<u8>::new(bytemuck::cast(ptr_bytes))
                        .read_utf8_string_with_nul(ctx.view())
                    {
                        Ok(str) => str,
                        Err(err) => {
                            error!("failed to read null terminated string");
                            continue;
                        }
                    };

                    output.push_str(&str);
                    arg_offset += 4;
                }
                // 'f' - float
                102 => {
                    let mut val: [u8; 8] = [0; 8];

                    match WasmPtr::<u8>::new(arg_offset).slice(ctx.view(), 8) {
                        Ok(slice) => match slice.read_slice(&mut val) {
                            Ok(_) => (),
                            Err(err) => {
                                error!("failed to read float WasmSlice");
                                continue;
                            }
                        },
                        Err(err) => {
                            error!("failed to read WasmSlice from arg_offset");
                            continue;
                        }
                    }

                    output.push_str(&f64::from_le_bytes(val).to_string());
                    arg_offset += 8;
                }
                _ => output.push(char::from_u32(ch as u32).unwrap_or('!')),
            }

            fmt_offset += 1;
        }
    }

    println!("{output}");
}

fn trace_utf8(env: FunctionEnvMut<WasmerRuntimeEnv>, ptr: WasmPtr<u8>, len: u32) {
    let ctx = Context::from_env(&env);

    let str = match ptr.read_utf8_string(ctx.view(), len) {
        Ok(bytes) => bytes,
        Err(err) => {
            error!(
                "Error getting bytes from cart in traceUtf8 at {:X?}: {}",
                ptr.offset(),
                err
            );
            return;
        }
    };

    println!("{str}");
}

fn trace_utf16(env: FunctionEnvMut<WasmerRuntimeEnv>, ptr: WasmPtr<u8>, len: u32) {
    let ctx = Context::from_env(&env);

    let slice = match ptr.slice(ctx.view(), len) {
        Ok(bytes) => bytes,
        Err(err) => {
            error!(
                "Error getting bytes from cart in traceUtf16 at {:X?}: {}",
                ptr.offset(),
                err
            );
            return;
        }
    };

    let bytes = match slice.read_to_vec() {
        Ok(bytes) => bytes,
        Err(err) => {
            error!("Error reading WasmSlice to Vec: {err}");
            return;
        }
    };

    // lossy conversion is better here, it's not likely
    // that the cart will give us an incompatible character.
    let str = String::from_utf16_lossy(bytemuck::cast_slice(&bytes));

    println!("{str}");
}

fn blit(
    env: FunctionEnvMut<WasmerRuntimeEnv>,
    ptr: WasmPtr<u8>,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    flags: u32,
) {
    blit_sub(env, ptr, x, y, width, height, 0, 0, width, flags)
}

#[allow(clippy::too_many_arguments)]
fn blit_sub(
    env: FunctionEnvMut<WasmerRuntimeEnv>,
    sprite: WasmPtr<u8>,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    src_x: u32,
    src_y: u32,
    stride: u32,
    flags: u32,
) {
    let ctx = Context::from_env(&env);
    let num_bits = stride * (height + src_y) * pixel_width_of_flags(flags);
    let len = (num_bits + 7) / 8;
    let sprite_slice = sprite.slice(ctx.view(), len).unwrap();

    let src = WasmSliceSinkSource {
        slice: sprite_slice,
    };

    console::blit_sub(
        &mut ctx.fb(),
        &src,
        x,
        y,
        width,
        height,
        src_x,
        src_y,
        stride,
        flags,
        ctx.draw_colors(),
    );
}

fn line(env: FunctionEnvMut<WasmerRuntimeEnv>, x1: i32, y1: i32, x2: i32, y2: i32) {
    let ctx = Context::from_env(&env);
    console::line(&mut ctx.fb(), ctx.draw_colors(), x1, y1, x2, y2);
}

fn hline(env: FunctionEnvMut<WasmerRuntimeEnv>, x: i32, y: i32, len: u32) {
    let ctx = Context::from_env(&env);
    console::hline(&mut ctx.fb(), ctx.draw_colors(), x, y, len);
}

fn vline(env: FunctionEnvMut<WasmerRuntimeEnv>, x: i32, y: i32, len: u32) {
    let ctx = Context::from_env(&env);
    console::vline(&mut ctx.fb(), ctx.draw_colors(), x, y, len);
}

fn oval(env: FunctionEnvMut<WasmerRuntimeEnv>, x: i32, y: i32, width: u32, height: u32) {
    let ctx = Context::from_env(&env);
    console::oval(&mut ctx.fb(), ctx.draw_colors(), x, y, width, height);
}
fn rect(env: FunctionEnvMut<WasmerRuntimeEnv>, x: i32, y: i32, width: u32, height: u32) {
    let ctx = Context::from_env(&env);
    console::rect(&mut ctx.fb(), ctx.draw_colors(), x, y, width, height);
}

fn text(env: FunctionEnvMut<WasmerRuntimeEnv>, ptr: WasmPtr<u8>, x: i32, y: i32) {
    let ctx = Context::from_env(&env);
    let w4_string = ptr.read_until(ctx.view(), |b| *b == 0).unwrap();

    console::text(&mut ctx.fb(), &w4_string, x, y, ctx.draw_colors())
}

fn text_utf8(env: FunctionEnvMut<WasmerRuntimeEnv>, ptr: WasmPtr<u8>, length: u32, x: i32, y: i32) {
    let ctx = Context::from_env(&env);
    let slice = ptr.slice(ctx.view(), length).unwrap();
    let w4_string = slice.read_to_vec().unwrap();

    console::text(&mut ctx.fb(), &w4_string, x, y, ctx.draw_colors())
}

fn text_utf16(
    env: FunctionEnvMut<WasmerRuntimeEnv>,
    ptr: WasmPtr<u8>,
    length: u32,
    x: i32,
    y: i32,
) {
    let ctx = Context::from_env(&env);
    let slice = ptr.slice(ctx.view(), length).unwrap();
    let w4_string = slice.read_to_vec().unwrap();

    console::text(
        &mut ctx.fb(),
        bytemuck::cast_slice::<u8, u16>(&w4_string),
        x,
        y,
        ctx.draw_colors(),
    )
}

fn diskr(env: FunctionEnvMut<WasmerRuntimeEnv>, dest: WasmPtr<u8>, size: u32) -> u32 {
    let ctx = Context::from_env(&env);
    let bytes_read = u32::min(size, 1024);

    let mut src = env.data().api.save_cache.get().to_vec();
    src.resize(bytes_read as usize, 0);

    dest.slice(ctx.view(), bytes_read)
        .expect("get memory slice")
        .write_slice(&src)
        .expect("write slice to memory");

    return bytes_read;
}

fn diskw(env: FunctionEnvMut<WasmerRuntimeEnv>, src: WasmPtr<u8>, size: u32) -> u32 {
    let ctx = Context::from_env(&env);
    let bytes_written = u32::min(size, 1024);

    let mut buf = src
        .slice(&ctx.view(), bytes_written)
        .expect("get memory slice")
        .read_to_vec()
        .expect("get memory slice to vec");
    buf.resize(1024, 0);

    env.data().api.save_cache.set(buf.try_into().unwrap());
    env.data().api.needs_write.set(true);

    return bytes_written;
}

fn tone(
    env: FunctionEnvMut<WasmerRuntimeEnv>,
    frequency: u32,
    duration: u32,
    volume: u32,
    flags: u32,
) {
    env.data().api.tone(frequency, duration, volume, flags)
}
