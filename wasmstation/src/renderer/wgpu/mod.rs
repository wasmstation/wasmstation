use std::{iter, num::NonZeroU32};

use wgpu::{
    include_wgsl,
    util::{BufferInitDescriptor, DeviceExt},
    *,
};
use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::{
    utils,
    wasm4::{self, FRAMEBUFFER_SIZE, SCREEN_SIZE},
    Renderer,
};

const VERTICES: &[[f32; 2]; 4] = &[[1.0, 1.0], [1.0, -1.0], [-1.0, -1.0], [-1.0, 1.0]];
const INDICES: &[i16] = &[3, 2, 0, 1, 0, 2];

struct WgpuRendererInternal {
    surface: Surface,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration,
    pub size: PhysicalSize<u32>,
    render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    palette_buffer: Buffer,
    window_size_buffer: Buffer,
    framebuffer_texture: Texture,
    bind_group: BindGroup,
}

impl WgpuRendererInternal {
    pub fn new_blocking(window: &Window) -> anyhow::Result<Self> {
        pollster::block_on(Self::new(window))
    }

    pub async fn new(window: &Window) -> anyhow::Result<Self> {
        let size = window.inner_size();
        let instance = Instance::new(Backends::all());
        let surface = unsafe { instance.create_surface(&window) };
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("get gpu adapter");

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    features: Features::empty(),
                    limits: Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Auto,
        };
        surface.configure(&device, &config);

        let palette_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Palette Buffer"),
            contents: bytemuck::cast_slice(&[[
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0],
            ]]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let framebuffer_texture = device.create_texture(&TextureDescriptor {
            label: Some("framebuffer_texture"),
            size: Extent3d {
                width: 6400,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D1,
            format: TextureFormat::R8Uint,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        });

        let window_size_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Window Size Buffer"),
            contents: bytemuck::cast_slice(&[size.width, size.height]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        queue.write_texture(
            ImageCopyTexture {
                texture: &framebuffer_texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            &utils::default_framebuffer(),
            ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(6400),
                rows_per_image: NonZeroU32::new(1),
            },
            Extent3d {
                width: 6400,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        let framebuffer_texture_view =
            framebuffer_texture.create_view(&TextureViewDescriptor::default());

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("metadata_bind_group_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D1,
                        sample_type: TextureSampleType::Uint,
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("metadata_bind_group"),
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: palette_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: window_size_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::TextureView(&framebuffer_texture_view),
                },
            ],
        });

        let shader = device.create_shader_module(include_wgsl!("shader.wgsl"));
        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let vertex_data_slice = bytemuck::cast_slice(VERTICES);
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: vertex_data_slice,
            usage: BufferUsages::VERTEX,
        });

        let vertex_buffer_layout = VertexBufferLayout {
            array_stride: (vertex_data_slice.len() / VERTICES.len()) as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[VertexAttribute {
                format: VertexFormat::Float32x2,
                offset: 0,
                shader_location: 0,
            }],
        };

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: BufferUsages::INDEX,
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[vertex_buffer_layout],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: config.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                polygon_mode: PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Ok(Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            palette_buffer,
            framebuffer_texture,
            bind_group,
            window_size_buffer,
        })
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn render(
        &mut self,
        framebuffer: [u8; FRAMEBUFFER_SIZE],
        palette: [u8; 16],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let palette_rgba: [[f32; 4]; 4] = bytemuck::cast::<[u8; 16], [u32; 4]>(palette)
            .iter()
            .map(|color| {
                // convert values to float rgba values (sRGB decoding?)
                let f = |xu: u32| {
                    let x = (xu & 0xFF) as f32 / 255.0;
                    if x > 0.04045 {
                        ((x + 0.055) / 1.055).powf(2.4)
                    } else {
                        x / 12.92
                    }
                };

                [f(color >> 16), f(color >> 8), f(*color), 1.0]
            })
            .collect::<Vec<[f32; 4]>>()
            .try_into()
            .unwrap();

        self.queue.write_buffer(
            &self.palette_buffer,
            0,
            bytemuck::cast_slice(&[palette_rgba]),
        );

        self.queue.write_buffer(
            &self.window_size_buffer,
            0,
            bytemuck::cast_slice(&[self.size.width, self.size.height]),
        );

        // update framebuffer
        self.queue.write_texture(
            ImageCopyTexture {
                texture: &self.framebuffer_texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            &framebuffer,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(6400),
                rows_per_image: NonZeroU32::new(1),
            },
            Extent3d {
                width: 6400,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        let output = self.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);

            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);

            render_pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
        }

        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

pub struct WgpuRenderer {
    pub display_scale: u32,
}

impl Default for WgpuRenderer {
    fn default() -> Self {
        Self { display_scale: 3 }
    }
}

impl Renderer for WgpuRenderer {
    fn present(self, mut backend: impl crate::Backend + 'static) {
        let event_loop = EventLoop::new();
        let window = {
            let size = LogicalSize::new(
                SCREEN_SIZE * self.display_scale,
                SCREEN_SIZE * self.display_scale,
            );
            WindowBuilder::new()
                .with_title("wasmstation")
                .with_inner_size(LogicalSize::new(
                    SCREEN_SIZE * self.display_scale,
                    SCREEN_SIZE * self.display_scale,
                ))
                .with_min_inner_size(LogicalSize::new(SCREEN_SIZE, SCREEN_SIZE))
                .build(&event_loop)
                .unwrap()
        };

        backend.call_start();

        let mut framebuffer: [u8; wasm4::FRAMEBUFFER_SIZE] = utils::default_framebuffer();
        let mut palette: [u8; 16] = utils::default_palette();

        let mut renderer =
            WgpuRendererInternal::new_blocking(&window).expect("initialize renderer");

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
                backend.read_screen(&mut framebuffer, &mut palette);

                if let Err(e) = renderer.render(framebuffer, palette) {
                    eprintln!("{e}");
                }
            }
            Event::MainEventsCleared => window.request_redraw(),
            _ => (),
        });
    }
}
