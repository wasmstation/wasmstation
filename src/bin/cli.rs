use std::{fs, path::PathBuf};

use argh::FromArgs;
use wasmstation::{utils, wasm4, Backend, Renderer, WasmerBackend, WgpuRenderer};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

#[derive(FromArgs)]
#[argh(description = "Run wasm4 compatible games.")]
struct Args {
    #[argh(positional)]
    path: PathBuf,
    #[argh(option, short = 's', default = "3", description = "scale factor for the window")]
    display_scale: u8,
}

fn main() {
    let args: Args = argh::from_env();
    let wasm_bytes = fs::read(&args.path).expect("failed to read game");

    let event_loop = EventLoop::new();
    let window = {
        let size = LogicalSize::new(
            160 * args.display_scale as u32,
            160 * args.display_scale as u32,
        );
        WindowBuilder::new()
            .with_title("wasmstation")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .with_resizable(false)
            .build(&event_loop)
            .unwrap()
    };

    let mut backend = WasmerBackend::new(&wasm_bytes).expect("failed to start wasm runtime");
    let mut renderer = WgpuRenderer::new_blocking(&window).expect("failed to start renderer");

    backend.call_start();

    let mut framebuffer: [u8; wasm4::FRAMEBUFFER_SIZE] = utils::default_framebuffer();
    let mut palette: [u8; 16] = utils::default_palette();

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { window_id, event } if window_id == window.id() => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(size) => renderer.resize(size),
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                renderer.resize(*new_inner_size)
            }
            _ => (),
        },
        Event::RedrawRequested(window_id) if window_id == window.id() => {
            backend.call_update();

            renderer.update_display_scale(args.display_scale as u32);
            backend.read_screen(&mut framebuffer, &mut palette);

            if let Err(e) = renderer.render(framebuffer, palette) {
                eprintln!("{e}");
            }
        }
        Event::MainEventsCleared => window.request_redraw(),
        _ => (),
    })
}
