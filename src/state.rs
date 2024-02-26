use std::{time::{Instant, Duration}, borrow::Cow, path::Path};

use wgpu::{core::pipeline::CreateShaderModuleError, util::{BufferInitDescriptor, DeviceExt}, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendState, Buffer, BufferBindingType, BufferUsages, ColorTargetState, ColorWrites, Device, FragmentState, FrontFace, MultisampleState, PipelineLayout, PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology, Queue, RenderPipeline, RenderPipelineDescriptor, ShaderModule, ShaderModuleDescriptor, ShaderSource, ShaderStages, Surface, SurfaceConfiguration, VertexState};
use winit::window::Window;

use crate::imgui_state::{ImState, Message};

pub struct TimeKeeper {
    last_render_time: Instant,
    starting_time: Instant,
    pub millis_buffer: BoundBuffer
}

pub struct BoundBuffer {
    pub buffer: Buffer,
    pub bg_layout: BindGroupLayout,
    pub bg: BindGroup
}

impl TimeKeeper {
    fn new(device: &Device) -> TimeKeeper {
        let now = Instant::now();
        let millis_uniform = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: &(now.elapsed().as_millis() as u32).to_le_bytes(),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        }).unwrap();

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
        }).unwrap();

        let bg = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bg_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: millis_uniform.as_entire_binding(),
                }
            ],
        }).unwrap();

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

pub struct Gpu<'surface> {
    pub surface: Surface<'surface>,
    pub device: Device,
    pub queue: Queue,
    pub config: SurfaceConfiguration
}

impl<'surface> Gpu<'surface> {
    pub fn new(surface: Surface<'_>, device: Device, queue: Queue, config: SurfaceConfiguration) -> Gpu<'_> {
        Gpu {
            surface,
            device,
            queue,
            config
        }
    }

    fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
    }
}

pub struct State<'surface> {
    pub gpu: Gpu<'surface>,
    pub pipeline: RenderPipeline,
    pub time: TimeKeeper,
    pub im_state: ImState,
    current_shader_path: String,
    current_shader: String
}

impl<'surface> State<'surface> {
    pub fn new(gpu: Gpu<'surface>, window: &Window) -> State<'surface> {
        let current_shader = std::fs::read_to_string(Path::new("shaders").join("shader.wgsl")).unwrap();
        let shader = gpu.device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(current_shader.clone().into()),
        }).unwrap();
        let time = TimeKeeper::new(&gpu.device);
        let layout = gpu.device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                &time.millis_buffer.bg_layout
            ],
            push_constant_ranges: &[],
        }).unwrap();
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
        }).unwrap();

        let im_state = ImState::new(window, &gpu);

        State {
            time,
            gpu,
            pipeline,
            im_state,
            current_shader_path: "shader.wgsl".into(),
            current_shader
        }
    }

    fn refresh_pipeline(&mut self, shader: ShaderModule) {
        let layout = self.get_pipeline_layout();

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
        }).unwrap();

        self.pipeline = pipeline;
    }

    pub fn refresh_shader(&mut self) {
        let shader = std::fs::read_to_string(Path::new("shaders").join(&self.current_shader_path)).unwrap();
        self.current_shader = shader;
        match self.gpu.device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(self.current_shader.clone().into()),
        }) {
            Ok(shader) => self.refresh_pipeline(shader),
            Err(err) => match err {
                CreateShaderModuleError::Parsing(_) => todo!(),
                CreateShaderModuleError::Generation => todo!(),
                CreateShaderModuleError::Device(_) => todo!(),
                CreateShaderModuleError::Validation(_) => todo!(),
                CreateShaderModuleError::MissingFeatures(_) => todo!(),
                CreateShaderModuleError::InvalidGroupIndex { bind, group, limit } => todo!(),
                _ => todo!(),
            },
        };
    }

    pub(crate) fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>) {
        self.gpu.resize(size)
    }

    pub(crate) fn handle_message(&mut self, message: Message) {
        match message {
            Message::ReloadShader => self.refresh_shader(),
            Message::LoadShader(shader) => {
                self.current_shader_path = shader;
                self.refresh_shader();
            },
            Message::ReloadPipeline => self.refresh_pipeline(),
        }
    }

    fn get_pipeline_layout(&mut self) -> PipelineLayout {
        let mut layouts = vec![
            &self.time.millis_buffer.bg_layout
        ];

        for (_, int) in self.im_state.ui.inputs.ints.iter() {
            layouts.push(&int.bg_layout)
        }

        for (_, float) in self.im_state.ui.inputs.floats.iter() {
            layouts.push(&float.bg_layout)
        }

        self.gpu.device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &layouts,
            push_constant_ranges: &[],
        }).unwrap()
    }
}
