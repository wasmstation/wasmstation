#![no_std]

extern crate alloc;

use core::{array, str};

use alloc::{string::String, vec::Vec};
use wasmi::{
    AsContext, AsContextMut, Caller, Engine, Func, Linker, Memory, MemoryType, Module, Store,
};
use wasmstation_core::{framebuffer, utils, wasm4, Api, Backend, Console, Sink, Source};

pub struct WasmiBackend {
    store: Store<WasmiBackendState>,
    start: Option<Func>,
    update: Option<Func>,
}

impl WasmiBackend {
    pub fn from_bytes(bytes: &[u8], console: &Console) -> Result<Self, wasmi::Error> {
        let engine = Engine::default();
        let module = Module::new(&engine, bytes).map_err(|e| wasmi::Error::from(e))?;

        let mut store: Store<WasmiBackendState> = Store::new(&engine, WasmiBackendState::default());
        let memory = Memory::new(&mut store, MemoryType::new(1, Some(1)).unwrap())
            .map_err(|e| wasmi::Error::from(e))?;

        memory.write(&mut store, wasm4::PALETTE_ADDR, &utils::default_palette())?;
        memory.write(
            &mut store,
            wasm4::DRAW_COLORS_ADDR,
            &utils::default_draw_colors(),
        )?;
        memory.write(
            &mut store,
            wasm4::FRAMEBUFFER_ADDR,
            &utils::default_framebuffer(),
        )?;

        // hacky, but strangely I don't think the API has an elegant way to do this
        *store.data_mut() = WasmiBackendState {
            memory: Some(memory),
            api: Some(console.create_api()),
        };

        let mut linker = <Linker<WasmiBackendState>>::new(&engine);
        linker
            .define("env", "memory", store.data().memory.unwrap())
            .map_err(|e| wasmi::Error::from(e))?;

        let env: [(&str, Func); 17] = [
            ("trace", Func::wrap(&mut store, trace)),
            ("tracef", Func::wrap(&mut store, tracef)),
            ("traceUtf8", Func::wrap(&mut store, trace_utf8)),
            ("traceUtf16", Func::wrap(&mut store, trace_utf16)),
            ("blit", Func::wrap(&mut store, blit)),
            ("blitSub", Func::wrap(&mut store, blit_sub)),
            ("line", Func::wrap(&mut store, line)),
            ("hline", Func::wrap(&mut store, hline)),
            ("vline", Func::wrap(&mut store, vline)),
            ("oval", Func::wrap(&mut store, oval)),
            ("rect", Func::wrap(&mut store, rect)),
            ("text", Func::wrap(&mut store, text)),
            ("textUtf8", Func::wrap(&mut store, text_utf8)),
            ("textUtf16", Func::wrap(&mut store, text_utf16)),
            ("diskr", Func::wrap(&mut store, diskr)),
            ("diskw", Func::wrap(&mut store, diskw)),
            ("tone", Func::wrap(&mut store, tone)),
        ];

        for (name, func) in env {
            linker
                .define("env", name, func)
                .map_err(|e| wasmi::Error::from(e))?;
        }

        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| wasmi::Error::from(e))?
            .start(&mut store)
            .map_err(|e| wasmi::Error::from(e))?;

        let start: Option<Func> = instance.get_func(&store, "start").map_or(None, Some);
        let update: Option<Func> = instance.get_func(&store, "update").map_or(None, Some);

        Ok(Self {
            store,
            start,
            update,
        })
    }
}

impl Backend for WasmiBackend {
    fn call_update(&mut self) {
        let mem = self.store.data().memory();

        let mut flags: [u8; 1] = [0];
        if let Err(err) = mem.read(&self.store, wasm4::SYSTEM_FLAGS_ADDR, &mut flags) {
            log::error!("error reading system flags: {err}");
        }

        if wasm4::SYSTEM_PRESERVE_FRAMEBUFFER & flags[0] == 0 {
            framebuffer::clear(&mut framebuffer(&mut self.store, mem));
        }

        if let Some(update) = &self.update {
            if let Err(err) = update.call(&mut self.store, &[], &mut []) {
                log::error!("error calling 'update': {:?}", err);
            };
        }
    }

    fn call_start(&mut self) {
        if let Some(start) = &self.start {
            if let Err(err) = start.call(&mut self.store, &[], &mut []) {
                log::error!("error calling 'update': {:?}", err);
            };
        }
    }

    fn read_screen(&self, framebuffer: &mut [u8; wasm4::FRAMEBUFFER_SIZE], palette: &mut [u8; 16]) {
        if let Err(err) =
            self.store
                .data()
                .memory()
                .read(&self.store, wasm4::FRAMEBUFFER_ADDR, framebuffer)
        {
            log::error!("error reading framebuffer to screen: {err}");
        }

        if let Err(err) = self
            .store
            .data()
            .memory()
            .read(&self.store, wasm4::PALETTE_ADDR, palette)
        {
            log::error!("error reading palette to screen: {err}");
        }
    }

    fn read_system_flags(&self) -> u8 {
        let mut flags = [0];

        if let Err(err) =
            self.store
                .data()
                .memory()
                .read(&self.store, wasm4::SYSTEM_FLAGS_ADDR, &mut flags)
        {
            log::error!("error reading system flags: {err}");
        }

        flags[0]
    }

    fn set_gamepad(&mut self, gamepad: u32) {
        if let Err(err) = self.store.data().memory().write(
            &mut self.store,
            wasm4::GAMEPAD1_ADDR,
            bytemuck::cast_slice(&[gamepad]),
        ) {
            log::error!("error writing to gamepad: {err}");
        }
    }

    fn set_mouse(&mut self, x: i16, y: i16, buttons: u8) {
        if let Err(err) = self.store.data().memory().write(
            &mut self.store,
            wasm4::MOUSE_X_ADDR,
            bytemuck::cast_slice(&[x]),
        ) {
            log::error!("error setting mouse x: {err}");
        }

        if let Err(err) = self.store.data().memory().write(
            &mut self.store,
            wasm4::MOUSE_Y_ADDR,
            bytemuck::cast_slice(&[y]),
        ) {
            log::error!("error setting mouse y: {err}");
        }

        if let Err(err) = self.store.data().memory().write(
            &mut self.store,
            wasm4::MOUSE_BUTTONS_ADDR,
            bytemuck::cast_slice(&[buttons]),
        ) {
            log::error!("error setting mouse buttons: {err}");
        }
    }

    fn write_save_cache(&mut self) -> Option<[u8; 1024]> {
        self.store.data().api().write_save()
    }

    fn set_save_cache(&mut self, data: [u8; 1024]) {
        self.store.data().api().save_cache.set(data);
    }
}

fn trace(caller: Caller<'_, WasmiBackendState>, mut ptr: u32) {
    let mut msg = String::new();
    let mut buf: [u8; 1] = [1];

    while buf[0] != 0 {
        if let Err(err) = caller.data().memory().read(&caller, ptr as usize, &mut buf) {
            log::error!("error reading bytes in trace: {err}");
            return;
        };

        ptr += 1;
        msg.push(buf[0] as char);
    }

    caller.data().api().print(&msg);
}

fn tracef(mut caller: Caller<'_, WasmiBackendState>, fmt: u32, args: u32) {
    let msg = wasmstation_core::tracef(
        fmt as usize,
        args as usize,
        &WasmiSlice {
            offset: 0,
            len: 64000,
            mem: caller.data().memory(),
            ctx: &mut caller,
        },
    );

    caller.data().api().print(&msg);
}

fn trace_utf8(caller: Caller<'_, WasmiBackendState>, ptr: u32, len: u32) {
    let mut buf = alloc::vec![0; len as usize];
    if let Err(err) = caller.data().memory().read(&caller, ptr as usize, &mut buf) {
        log::error!("error reading traceUtf8 bytes: {err}");
    }

    caller.data().api().print(&String::from_utf8_lossy(&buf));
}

fn trace_utf16(caller: Caller<'_, WasmiBackendState>, ptr: u32, len: u32) {
    let mut buf = alloc::vec![0; len as usize];
    if let Err(err) = caller.data().memory().read(&caller, ptr as usize, &mut buf) {
        log::error!("error reading traceUtf16 bytes: {err}");
    }

    caller
        .data()
        .api()
        .print(&String::from_utf16_lossy(bytemuck::cast_slice(&buf)));
}

fn blit(
    caller: Caller<'_, WasmiBackendState>,
    ptr: u32,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    flags: u32,
) {
    blit_sub(caller, ptr, x, y, width, height, 0, 0, width, flags)
}

fn blit_sub(
    mut caller: Caller<'_, WasmiBackendState>,
    sprite: u32,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    src_x: u32,
    src_y: u32,
    stride: u32,
    flags: u32,
) {
    let mem = caller.data().memory();
    let dc = draw_colors(&caller, mem);

    let num_bits = stride * (height + src_y) * framebuffer::pixel_width_of_flags(flags);

    let mut sprite_buf = alloc::vec![0; ((num_bits + 7) / 8) as usize];
    if let Err(err) = caller
        .data()
        .memory()
        .read(&caller, sprite as usize, &mut sprite_buf)
    {
        log::error!("error reading sprite bytes: {err}");
    }

    framebuffer::blit_sub(
        &mut framebuffer(&mut caller, mem),
        &sprite_buf,
        x,
        y,
        width,
        height,
        src_x,
        src_y,
        stride,
        flags,
        dc,
    );
}

fn line(mut caller: Caller<'_, WasmiBackendState>, x1: i32, y1: i32, x2: i32, y2: i32) {
    let mem = caller.data().memory();
    let dc = draw_colors(&caller, mem);

    framebuffer::line(&mut framebuffer(&mut caller, mem), dc, x1, y1, x2, y2);
}

fn hline(mut caller: Caller<'_, WasmiBackendState>, x: i32, y: i32, len: u32) {
    let mem = caller.data().memory();
    let dc = draw_colors(&caller, mem);

    framebuffer::hline(&mut framebuffer(&mut caller, mem), dc, x, y, len);
}

fn vline(mut caller: Caller<'_, WasmiBackendState>, x: i32, y: i32, len: u32) {
    let mem = caller.data().memory();
    let dc = draw_colors(&caller, mem);

    framebuffer::vline(&mut framebuffer(&mut caller, mem), dc, x, y, len);
}

fn oval(mut caller: Caller<'_, WasmiBackendState>, x: i32, y: i32, width: u32, height: u32) {
    let mem = caller.data().memory();
    let dc = draw_colors(&caller, mem);

    framebuffer::oval(&mut framebuffer(&mut caller, mem), dc, x, y, width, height);
}

fn rect(mut caller: Caller<'_, WasmiBackendState>, x: i32, y: i32, width: u32, height: u32) {
    let mem = caller.data().memory();
    let dc = draw_colors(&caller, mem);

    framebuffer::rect(&mut framebuffer(&mut caller, mem), dc, x, y, width, height);
}

fn text(mut caller: Caller<'_, WasmiBackendState>, mut ptr: u32, x: i32, y: i32) {
    let mem = caller.data().memory();
    let dc = draw_colors(&caller, mem);

    let mut text: Vec<u8> = Vec::new();
    let mut buf: [u8; 1] = [1];

    while buf[0] != 0 {
        if let Err(err) = caller.data().memory().read(&caller, ptr as usize, &mut buf) {
            log::error!("error reading bytes in text: {err}");
            return;
        };

        ptr += 1;
        text.push(buf[0]);
    }

    framebuffer::text(&mut framebuffer(&mut caller, mem), &text, x, y, dc);
}

fn text_utf8(mut caller: Caller<'_, WasmiBackendState>, ptr: u32, len: u32, x: i32, y: i32) {
    let mem = caller.data().memory();
    let dc = draw_colors(&caller, mem);

    let mut text = alloc::vec![0; len as usize];
    if let Err(err) = caller
        .data()
        .memory()
        .read(&caller, ptr as usize, &mut text)
    {
        log::error!("error reading traceUtf8 bytes: {err}");
    }

    framebuffer::text(&mut framebuffer(&mut caller, mem), &text, x, y, dc);
}

fn text_utf16(mut caller: Caller<'_, WasmiBackendState>, ptr: u32, len: u32, x: i32, y: i32) {
    let mem = caller.data().memory();
    let dc = draw_colors(&caller, mem);

    let mut text = alloc::vec![0; len as usize];
    if let Err(err) = caller
        .data()
        .memory()
        .read(&caller, ptr as usize, &mut text)
    {
        log::error!("error reading traceUtf8 bytes: {err}");
    }

    framebuffer::text(
        &mut framebuffer(&mut caller, mem),
        &bytemuck::cast_slice::<u8, u16>(&text),
        x,
        y,
        dc,
    );
}

fn diskr(mut caller: Caller<'_, WasmiBackendState>, dest: u32, len: u32) -> u32 {
    let data = caller.data().api().save_cache.get();

    if let Err(err) = caller
        .data()
        .memory()
        .write(&mut caller, dest as usize, &data)
    {
        log::error!("error writing disk cache to memory: {err}");
    };

    u32::min(len, 1024)
}

fn diskw(mut caller: Caller<'_, WasmiBackendState>, src: u32, len: u32) -> u32 {
    let mut buf = alloc::vec![0; len as usize];
    if let Err(err) = caller
        .data()
        .memory()
        .read(&mut caller, src as usize, &mut buf)
    {
        log::error!("error reading disk cache from memory: {err}");
    };

    buf.resize(1024, 0);

    caller.data().api().needs_write.set(true);
    caller
        .data()
        .api()
        .save_cache
        .set(buf.try_into().expect("resize disk write contents"));

    u32::min(len, 1024)
}

fn tone(caller: Caller<'_, WasmiBackendState>, freq: u32, dura: u32, vol: u32, flags: u32) {
    caller.data().api().tone(freq, dura, vol, flags);
}

struct WasmiSlice<C> {
    offset: usize,
    len: usize,
    mem: Memory,
    ctx: C,
}

impl<C> Source<u8> for WasmiSlice<C>
where
    C: AsContext,
{
    fn item_at(&self, offset: usize) -> Option<u8> {
        if self.len < offset {
            return None;
        }

        let mut buf: [u8; 1] = [0];

        if self
            .mem
            .read(&self.ctx, self.offset + offset, &mut buf)
            .is_err()
        {
            return None;
        }

        Some(buf[0])
    }

    fn items_at<const L: usize>(&self, offset: usize) -> Option<[u8; L]> {
        if self.len < (offset + L) {
            return None;
        }

        let mut buf: [u8; L] = array::from_fn(|_| 0);

        if self
            .mem
            .read(&self.ctx, self.offset + offset, &mut buf)
            .is_err()
        {
            return None;
        }

        Some(buf)
    }
}

impl<C> Sink<u8> for WasmiSlice<C>
where
    C: AsContextMut,
{
    fn set_item_at(&mut self, offset: usize, item: u8) {
        if self.len < offset {
            return;
        }

        if let Err(err) = self.mem.write(&mut self.ctx, self.offset + offset, &[item]) {
            log::error!("Couldn't set item at in WasmiSlice: {err}");
        }
    }

    fn fill(&mut self, item: u8) {
        for n in self.offset..(self.offset + self.len) {
            if let Err(err) = self.mem.write(&mut self.ctx, n, &[item]) {
                log::error!("Couldn't fill WasmiSlice: {err}");
            }
        }
    }
}

#[derive(Default)]
struct WasmiBackendState {
    pub memory: Option<Memory>,
    pub api: Option<Api>,
}

impl WasmiBackendState {
    // eh... haha :)
    pub fn memory(&self) -> Memory {
        self.memory.unwrap()
    }

    pub fn api(&self) -> &Api {
        self.api.as_ref().unwrap()
    }
}

fn framebuffer<C: AsContextMut>(ctx: C, mem: Memory) -> WasmiSlice<C> {
    WasmiSlice {
        offset: wasm4::FRAMEBUFFER_ADDR,
        len: wasm4::FRAMEBUFFER_SIZE,
        mem,
        ctx,
    }
}

fn draw_colors<C: AsContext>(ctx: C, mem: Memory) -> u16 {
    let mut buf: [u8; 2] = [0, 0];

    if let Err(err) = mem.read(ctx, wasm4::DRAW_COLORS_ADDR, &mut buf) {
        log::error!("error reading draw colors: {err}");
    }

    bytemuck::cast(buf)
}
