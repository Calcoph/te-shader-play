use std::{time::{Instant, Duration}, borrow::Cow, path::Path};

use wgpu::{Device, Surface, Queue, SurfaceConfiguration, RenderPipeline, ShaderSource, ShaderModuleDescriptor, PipelineLayoutDescriptor, VertexState, PrimitiveState, PrimitiveTopology, FrontFace, PolygonMode, MultisampleState, FragmentState, ColorTargetState, BlendState, ColorWrites, RenderPipelineDescriptor, BindingType, ShaderStages, BufferBindingType, BindGroupLayoutEntry, BindGroupDescriptor, BindGroupEntry, BufferUsages, BindGroupLayoutDescriptor, BindGroup, BindGroupLayout, Buffer, util::{BufferInitDescriptor, DeviceExt}};
use winit::window::Window;

use crate::imgui_state::ImState;

pub struct TimeKeeper {
    last_render_time: Instant,
    starting_time: Instant,
    pub millis_buffer: BoundBuffer
}

pub struct BoundBuffer {
    buffer: Buffer,
    bg_layout: BindGroupLayout,
    pub bg: BindGroup
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

    pub fn update_time(&mut self, queue: &Queue) -> Duration {
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

pub struct Gpu {
    pub surface: Surface,
    pub device: Device,
    pub queue: Queue,
    pub config: SurfaceConfiguration
}

impl Gpu {
    pub fn new(surface: Surface, device: Device, queue: Queue, config: SurfaceConfiguration) -> Gpu {
        Gpu {
            surface,
            device,
            queue,
            config
        }
    }
}

pub struct State {
    pub gpu: Gpu,
    pub pipeline: RenderPipeline,
    pub time: TimeKeeper,
    pub im_state: ImState
}

impl State {
    pub fn new(gpu: Gpu, window: &Window) -> State {
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

        let im_state = ImState::new(window, &gpu);

        State {
            time,
            gpu,
            pipeline,
            im_state
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

    pub fn refresh_default_shader(&mut self) {
        let shader = std::fs::read_to_string(Path::new("shaders").join("shader.wgsl")).unwrap().into();
        self.swap_shader(shader);
    }
}
