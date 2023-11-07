use std::{time::{Instant, Duration}, borrow::Cow, path::Path};

use wgpu::{Instance, InstanceDescriptor, Backends, Dx12Compiler, InstanceFlags, Gles3MinorVersion, RequestAdapterOptionsBase, PowerPreference, Device, DeviceDescriptor, Features, Limits, TextureUsages, PresentMode, CompositeAlphaMode, Surface, CommandEncoderDescriptor, RenderPassDescriptor, RenderPassColorAttachment, Operations, LoadOp, Color, StoreOp, SurfaceTexture, RenderPipeline, ShaderModuleDescriptor, ShaderSource, RenderPipelineDescriptor, VertexState, PrimitiveState, PrimitiveTopology, FrontFace, PolygonMode, MultisampleState, FragmentState, Queue, ColorTargetState, SurfaceConfiguration, ColorWrites, BlendState, util::{DeviceExt, BufferInitDescriptor}, BufferUsages, BindGroupLayoutDescriptor, BindGroupLayoutEntry, ShaderStages, BindingType, BufferBindingType, BindGroupDescriptor, BindGroupEntry, BindGroup, Buffer, BindGroupLayout, PipelineLayoutDescriptor};
use winit::{event_loop::{EventLoopBuilder, EventLoopWindowTarget, ControlFlow}, window::{WindowBuilder, Window}, dpi, event::{Event, WindowEvent}};

const SCREEN_WIDTH: u32 = 512;
const SCREEN_HEIGHT: u32 = 512;

fn main() {
    let event_loop = EventLoopBuilder::default().build().expect("Couldn't create event loop");

    let wb = WindowBuilder::new()
        .with_inner_size(dpi::LogicalSize::new(SCREEN_WIDTH, SCREEN_HEIGHT))
        .with_resizable(false);

    let window = wb.build(&event_loop).expect("Couldn't create window");
    window.set_window_level(winit::window::WindowLevel::AlwaysOnTop);
    let instance = Instance::new(InstanceDescriptor {
        backends: Backends::all(),
        flags: InstanceFlags::default(),
        dx12_shader_compiler: Dx12Compiler::Fxc,
        gles_minor_version: Gles3MinorVersion::Automatic,
    });

    let surface = unsafe {
         instance
            .create_surface(&window)
            .expect("Unable to create surface")
    };

    let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptionsBase {
        power_preference: PowerPreference::default(),
        force_fallback_adapter: false,
        compatible_surface: Some(&surface)
    })).expect("Unable to request adapter");

    let (device, queue) = pollster::block_on(adapter.request_device(
        &DeviceDescriptor {
            label: None,
            features: Features::default(),
            limits: Limits::downlevel_webgl2_defaults()
        },
        None
    )).expect("Unable to request device");

    let config = wgpu::SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format: surface.get_capabilities(&adapter).formats[0],
        width: SCREEN_WIDTH,
        height: SCREEN_HEIGHT,
        present_mode: PresentMode::Fifo,
        alpha_mode: CompositeAlphaMode::Auto,
        view_formats: vec![surface.get_capabilities(&adapter).formats[0]],
    };
    dbg!(config.format);
    surface.configure(
        &device,
        &config
    );

    let gpu = Gpu::new(surface, device, queue, config);
    let mut state = State::new(gpu);
    event_loop.run(move |event, window_target| run_event_loop(event, window_target, &window, &mut state)).unwrap()
}

fn run_event_loop(event: Event<()>, window_target: &EventLoopWindowTarget<()>, window: &Window, state: &mut State) {
    window_target.set_control_flow(ControlFlow::Poll);
    match event {
        Event::WindowEvent { window_id: _, event } => handle_window_event(event, window_target, state),
        Event::Suspended => window_target.set_control_flow(ControlFlow::Wait),
        Event::AboutToWait => window.request_redraw(),
        _ => ()
    }
}

fn handle_window_event(event: WindowEvent, window_target: &EventLoopWindowTarget<()>, state: &mut State) {
    match event {
        WindowEvent::CloseRequested => window_target.exit(),
        WindowEvent::RedrawRequested => {
            let dt = state.time.update_time(&state.gpu.queue);
            if let Ok(output) = state.gpu.surface.get_current_texture() {
                render(output, state);
            }
        },
        WindowEvent::KeyboardInput { event, .. } => handle_keyboard(event, state),
        _ => ()
    }
}

fn handle_keyboard(event: winit::event::KeyEvent, state: &mut State) {
    match event.physical_key {
        winit::keyboard::PhysicalKey::Code(c) => match c {
            winit::keyboard::KeyCode::KeyQ => state.refresh_default_shader(),
            _ => (),
        },
        winit::keyboard::PhysicalKey::Unidentified(_) => (),
    }
}

fn render(output: SurfaceTexture, state: &mut State) {
    let view = output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = state.gpu.device.create_command_encoder(&CommandEncoderDescriptor { label: None });
    {
        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color {
                        r: 1.0,
                        g: 0.5,
                        b: 0.5,
                        a: 1.0,
                    }),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        render_pass.set_pipeline(&state.pipeline);
        render_pass.set_bind_group(0, &state.time.millis_buffer.bg, &[]);
        render_pass.draw(0..3, 0..2)
    }
    state.gpu.queue.submit(std::iter::once(encoder.finish()));
    output.present();
}

struct Gpu {
    surface: Surface,
    device: Device,
    queue: Queue,
    config: SurfaceConfiguration
}

impl Gpu {
    fn new(surface: Surface, device: Device, queue: Queue, config: SurfaceConfiguration) -> Gpu {
        Gpu {
            surface,
            device,
            queue,
            config
        }
    }
}

struct TimeKeeper {
    last_render_time: Instant,
    starting_time: Instant,
    millis_buffer: BoundBuffer
}

struct BoundBuffer {
    buffer: Buffer,
    bg_layout: BindGroupLayout,
    bg: BindGroup
}

impl TimeKeeper {
    fn new(device: &Device) -> TimeKeeper {
        let now = Instant::now();
        let millis_uniform = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: &(now.elapsed().as_millis() as u32).to_le_bytes(),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let bg_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None
                    },
                    count: None,
                }
            ],
        });

        let bg = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bg_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: millis_uniform.as_entire_binding(),
                }
            ],
        });

        TimeKeeper {
            last_render_time: now,
            starting_time: now,
            millis_buffer: BoundBuffer {
                buffer: millis_uniform,
                bg_layout,
                bg
            }
        }
    }

    fn update_time(&mut self, queue: &Queue) -> Duration {
        let now = Instant::now();
        let dt = now - self.last_render_time;
        self.last_render_time = now;

        queue.write_buffer(
            &self.millis_buffer.buffer,
            0,
            &(self.starting_time.elapsed().as_millis() as u32).to_le_bytes()
        );

        dt
    }
}

struct State {
    gpu: Gpu,
    pipeline: RenderPipeline,
    time: TimeKeeper
}

impl State {
    fn new(gpu: Gpu) -> State {
        let shader = include_str!("../shaders/shader.wgsl").into();
        let shader = gpu.device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(shader),
        });
        let time = TimeKeeper::new(&gpu.device);
        let layout = gpu.device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                &time.millis_buffer.bg_layout
            ],
            push_constant_ranges: &[],
        });
        let pipeline = gpu.device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: gpu.config.format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        State {
            time,
            gpu,
            pipeline
        }
    }

    fn swap_shader(&mut self, shader: Cow<'_, str>) {
        let shader = self.gpu.device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(shader),
        });

        let layout = self.gpu.device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[
                    &self.time.millis_buffer.bg_layout
                ],
                push_constant_ranges: &[],
            });

        let pipeline = self.gpu.device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: self.gpu.config.format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        self.pipeline = pipeline;
    }

    fn refresh_default_shader(&mut self) {
        let shader = std::fs::read_to_string(Path::new("shaders").join("shader.wgsl")).unwrap().into();
        self.swap_shader(shader);
    }
}
