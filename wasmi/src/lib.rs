#![no_std]
#![allow(dead_code, unused_variables)]

extern crate alloc;

use alloc::string::String;
use core::array;

use wasmi::{
    AsContext, AsContextMut, Caller, Engine, Func, Instance, Linker, Memory, MemoryType, Module,
    Store,
};
use wasmstation_core::{framebuffer, utils, wasm4, Backend, Sink, Source};

pub struct WasmiBackend {
    instance: Instance,
    store: Store<WasmiBackendState>,
    start: Option<Func>,
    update: Option<Func>,
}

impl WasmiBackend {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, wasmi::Error> {
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
        };

        let mut linker = <Linker<WasmiBackendState>>::new(&engine);
        linker
            .define("env", "memory", store.data().memory.unwrap())
            .map_err(|e| wasmi::Error::from(e))?;

        let env: [(&str, Func); 17] = [
            ("trace", trace(&mut store)),
            ("tracef", tracef(&mut store)),
            ("traceUtf8", trace_utf8(&mut store)),
            ("traceUtf16", trace_utf16(&mut store)),
            ("blit", blit(&mut store)),
            ("blitSub", blit_sub(&mut store)),
            ("line", line(&mut store)),
            ("hline", hline(&mut store)),
            ("vline", vline(&mut store)),
            ("oval", oval(&mut store)),
            ("rect", rect(&mut store)),
            ("text", text(&mut store)),
            ("textUtf8", text_utf8(&mut store)),
            ("textUtf16", text_utf16(&mut store)),
            ("diskr", diskr(&mut store)),
            ("diskw", diskw(&mut store)),
            ("tone", tone(&mut store)),
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
            instance,
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
        mem.read(&self.store, wasm4::SYSTEM_FLAGS_ADDR, &mut flags)
            .expect("read system flags");

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
        self.store
            .data()
            .memory()
            .read(&self.store, wasm4::FRAMEBUFFER_ADDR, framebuffer)
            .expect("read to screen");
    }

    fn read_system_flags(&self) -> u8 {
        todo!()
    }

    fn set_gamepad(&mut self, gamepad: u32) {
        todo!()
    }

    fn set_mouse(&mut self, x: i16, y: i16, buttons: u8) {
        todo!()
    }

    fn write_save_cache(&mut self) -> Option<[u8; 1024]> {
        todo!()
    }

    fn set_save_cache(&mut self, data: [u8; 1024]) {
        todo!()
    }
}

fn trace(store: &mut Store<WasmiBackendState>) -> Func {
    Func::wrap(
        store,
        |caller: Caller<'_, WasmiBackendState>, ptr: u32| todo!(),
    )
}

fn tracef(store: &mut Store<WasmiBackendState>) -> Func {
    Func::wrap(
        store,
        |caller: Caller<'_, WasmiBackendState>, fmt: u32, args: u32| todo!(),
    )
}

fn trace_utf8(store: &mut Store<WasmiBackendState>) -> Func {
    Func::wrap(
        store,
        |caller: Caller<'_, WasmiBackendState>, ptr: u32, len: u32| {
            let mut buf = alloc::vec![0; len as usize];
            caller
                .data()
                .memory()
                .read(&caller, ptr as usize, &mut buf)
                .expect("read traceUtf8 string");

            log::info!("traceUtf8: {}", String::from_utf8_lossy(&buf));
        },
    )
}

fn trace_utf16(store: &mut Store<WasmiBackendState>) -> Func {
    Func::wrap(
        store,
        |caller: Caller<'_, WasmiBackendState>, ptr: u32, len: u32| todo!(),
    )
}

fn blit(store: &mut Store<WasmiBackendState>) -> Func {
    Func::wrap(
        store,
        |caller: Caller<'_, WasmiBackendState>,
         ptr: u32,
         x: i32,
         y: i32,
         width: u32,
         height: u32,
         flags: u32| { todo!() },
    )
}

fn blit_sub(store: &mut Store<WasmiBackendState>) -> Func {
    Func::wrap(
        store,
        |caller: Caller<'_, WasmiBackendState>,
         sprite: u32,
         x: i32,
         y: i32,
         width: u32,
         height: u32,
         src_x: u32,
         src_y: u32,
         stride: u32,
         flags: u32| todo!(),
    )
}

fn line(store: &mut Store<WasmiBackendState>) -> Func {
    Func::wrap(
        store,
        |mut caller: Caller<'_, WasmiBackendState>, x1: i32, y1: i32, x2: i32, y2: i32| {
            framebuffer::line(
                &mut WasmiSlice {
                    offset: wasm4::FRAMEBUFFER_ADDR,
                    len: wasm4::FRAMEBUFFER_SIZE,
                    mem: caller.data().memory(),
                    ctx: &mut caller,
                },
                0x1,
                x1,
                y1,
                x2,
                y2,
            );
        },
    )
}

fn hline(store: &mut Store<WasmiBackendState>) -> Func {
    Func::wrap(
        store,
        |caller: Caller<'_, WasmiBackendState>, x: i32, y: i32, len: u32| todo!(),
    )
}

fn vline(store: &mut Store<WasmiBackendState>) -> Func {
    Func::wrap(
        store,
        |caller: Caller<'_, WasmiBackendState>, x: i32, y: i32, len: u32| todo!(),
    )
}

fn oval(store: &mut Store<WasmiBackendState>) -> Func {
    Func::wrap(
        store,
        |caller: Caller<'_, WasmiBackendState>, x: i32, y: i32, width: u32, height: u32| todo!(),
    )
}

fn rect(store: &mut Store<WasmiBackendState>) -> Func {
    Func::wrap(
        store,
        |caller: Caller<'_, WasmiBackendState>, x: i32, y: i32, width: u32, height: u32| todo!(),
    )
}

fn text(store: &mut Store<WasmiBackendState>) -> Func {
    Func::wrap(
        store,
        |caller: Caller<'_, WasmiBackendState>, ptr: u32, x: i32, y: i32| todo!(),
    )
}

fn text_utf8(store: &mut Store<WasmiBackendState>) -> Func {
    Func::wrap(
        store,
        |caller: Caller<'_, WasmiBackendState>, ptr: u32, len: u32, x: i32, y: i32| todo!(),
    )
}

fn text_utf16(store: &mut Store<WasmiBackendState>) -> Func {
    Func::wrap(
        store,
        |caller: Caller<'_, WasmiBackendState>, ptr: u32, len: u32, x: i32, y: i32| todo!(),
    )
}

fn diskr(store: &mut Store<WasmiBackendState>) -> Func {
    Func::wrap(
        store,
        |caller: Caller<'_, WasmiBackendState>, dest: u32, len: u32| todo!(),
    )
}

fn diskw(store: &mut Store<WasmiBackendState>) -> Func {
    Func::wrap(
        store,
        |caller: Caller<'_, WasmiBackendState>, src: u32, len: u32| todo!(),
    )
}

fn tone(store: &mut Store<WasmiBackendState>) -> Func {
    Func::wrap(
        store,
        |caller: Caller<'_, WasmiBackendState>, freq: u32, dura: u32, vol: u32, flags: u32| todo!(),
    )
}

struct WasmiSlice<C: AsContextMut> {
    offset: usize,
    len: usize,
    mem: Memory,
    ctx: C,
}

impl<C: AsContextMut> Source<u8> for WasmiSlice<C> {
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

impl<'a, C: AsContextMut> Sink<u8> for WasmiSlice<C> {
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
}

impl WasmiBackendState {
    // eh... haha :)
    pub fn memory(&self) -> Memory {
        self.memory.unwrap()
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

    mem.read(ctx, wasm4::DRAW_COLORS_ADDR, &mut buf)
        .expect("WasmiBackendState read DRAW_COLORS");

    bytemuck::cast(buf)
}
