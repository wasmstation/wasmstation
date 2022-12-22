use wasmer::{
    imports, AsStoreRef, Engine, Function, FunctionEnv, FunctionEnvMut, Instance, Memory,
    MemoryType, MemoryView, Module, Store, TypedFunction, ValueType, WasmPtr, WasmSlice,
};

use crate::{
    console, utils,
    wasm4::{self, DRAW_COLORS_ADDR, FRAMEBUFFER_ADDR, FRAMEBUFFER_SIZE},
    Backend, Sink, Source,
};

pub struct WasmerBackend {
    fn_env: FunctionEnv<WasmerRuntimeEnv>,
    store: Store,
    instance: Instance,
}

impl WasmerBackend {
    pub fn new(wasm_bytes: &[u8]) -> anyhow::Result<Self> {
        Self::precompiled(&Module::new(&Store::default(), wasm_bytes)?.serialize()?)
    }

    pub fn precompiled(module_bytes: &[u8]) -> anyhow::Result<Self> {
        let mut store = Store::new(Engine::headless());
        let module = unsafe { Module::deserialize(&store, module_bytes)? };

        // init memory and env
        let wasm_env = WasmerRuntimeEnv::new(&mut store)?;
        let fn_env = FunctionEnv::new(&mut store, wasm_env);

        // see https://wasm4.org/docs/reference/functions
        let imports = imports! {
            "env" => {
                "memory" => fn_env.as_ref(&store).memory.clone(), "traceUtf8" => Function::new_typed_with_env(&mut store, &fn_env, trace_utf8),
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
        WasmPtr::<[u8; wasm4::FRAMEBUFFER_SIZE]>::new(wasm4::FRAMEBUFFER_ADDR as u32)
            .write(&view, utils::default_framebuffer())
            .expect("clear framebuffer");

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

    fn set_gamepad(gamepad: u32) {}

    fn set_mouse(x: i16, y: i16, buttons: u8) {}
}

struct WasmerRuntimeEnv {
    memory: Memory,
}

impl WasmerRuntimeEnv {
    pub fn new(store: &mut Store) -> anyhow::Result<Self> {
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

        Ok(Self { memory })
    }
}

struct Context<'a> {
    view: MemoryView<'a>,
}

impl <'a> Context<'a> {
    fn from_env(env: &'a FunctionEnvMut<'a,WasmerRuntimeEnv>) -> Context<'a> {
        let view = env.data().memory.view(&env.as_store_ref());

        Self{
            view,
        }    
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
}

fn trace_utf8(env: FunctionEnvMut<WasmerRuntimeEnv>, ptr: WasmPtr<u8>, len: i32) {}
fn trace_utf16(env: FunctionEnvMut<WasmerRuntimeEnv>, ptr: WasmPtr<u8>, len: i32) {}
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
    let num_bits = stride * (height + src_y) * console::fb::pixel_width_of_flags(flags);
    let len = (num_bits + 7) / 8;
    let sprite_slice = sprite.slice(ctx.view(), len).unwrap();

    let src = WasmSliceSinkSource {
        slice: sprite_slice,
    };

    console::fb::blit_sub(
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
    console::fb::line(&mut ctx.fb(), ctx.draw_colors(), x1, y1, x2, y2);
}

fn hline(env: FunctionEnvMut<WasmerRuntimeEnv>, x: i32, y: i32, len: u32) {
    let ctx = Context::from_env(&env);
    console::fb::hline(&mut ctx.fb(), ctx.draw_colors(), x, y, len);
}

fn vline(env: FunctionEnvMut<WasmerRuntimeEnv>, x: i32, y: i32, len: u32) {
    let ctx = Context::from_env(&env);
    console::fb::vline(&mut ctx.fb(), ctx.draw_colors(), x, y, len);
}

fn oval(env: FunctionEnvMut<WasmerRuntimeEnv>, x: i32, y: i32, width: u32, height: u32) {
    let ctx = Context::from_env(&env);
    console::fb::oval(&mut ctx.fb(), ctx.draw_colors(), x, y, width, height);
}
fn rect(env: FunctionEnvMut<WasmerRuntimeEnv>, x: i32, y: i32, width: u32, height: u32) {
    let ctx = Context::from_env(&env);
    console::fb::rect(&mut ctx.fb(), ctx.draw_colors(), x, y, width, height);
}

fn text(env: FunctionEnvMut<WasmerRuntimeEnv>, ptr: WasmPtr<u8>, x: i32, y: i32) {
    let ctx = Context::from_env(&env);
    let w4_string = ptr.read_until(ctx.view(), |b| *b == 0).unwrap();

    console::fb::text(&mut ctx.fb(), &w4_string, x, y, ctx.draw_colors())
}

fn text_utf8(env: FunctionEnvMut<WasmerRuntimeEnv>, ptr: WasmPtr<u8>, length: u32, x: i32, y: i32) {
    let ctx = Context::from_env(&env);
    let slice = ptr.slice(ctx.view(), length).unwrap();
    let w4_string = slice.read_to_vec().unwrap();

    console::fb::text(&mut ctx.fb(), &w4_string, x, y, ctx.draw_colors())
}

fn text_utf16(
    env: FunctionEnvMut<WasmerRuntimeEnv>,
    ptr: WasmPtr<u8>,
    length: u32,
    x: i32,
    y: i32,
) {
}
fn diskr(env: FunctionEnvMut<WasmerRuntimeEnv>, dest: WasmPtr<u8>, size: u32) {}
fn diskw(env: FunctionEnvMut<WasmerRuntimeEnv>, src: WasmPtr<u8>, size: u32) {}
fn tone(
    env: FunctionEnvMut<WasmerRuntimeEnv>,
    frequency: u32,
    duration: u32,
    volume: u32,
    flags: u32,
) {
}
