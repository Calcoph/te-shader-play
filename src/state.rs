use std::{any::Any, borrow::Cow, error::Error, path::Path, time::{Duration, Instant}};

use wgpu::{core::{binding_model::LateMinBufferBindingSizeMismatch, command::{DrawError, PassErrorScope, RenderPassError, RenderPassErrorInner}, pipeline::{CreateRenderPipelineError, CreateShaderModuleError}, validation::{BindingError, StageError}}, naga::compact, BlendState, ColorTargetState, ColorWrites, Device, FragmentState, FrontFace, MultisampleState, PipelineLayout, PipelineLayoutDescriptor, PolygonMode, PrimitiveState, PrimitiveTopology, Queue, RenderPipeline, RenderPipelineDescriptor, ShaderModule, ShaderModuleDescriptor, ShaderSource, ShaderStages, Surface, SurfaceConfiguration, VertexState};
use winit::window::Window;

use crate::imgui_state::{ImState, Message, Uniforms};

pub struct TimeKeeper {
    last_render_time: Instant,
    starting_time: Instant,
}

impl TimeKeeper {
    fn new() -> TimeKeeper {
        let now = Instant::now();

        TimeKeeper {
            last_render_time: now,
            starting_time: now,
        }
    }

    pub fn update_time(&mut self, queue: &Queue, uniforms: &mut Uniforms) -> Duration {
        let now = Instant::now();
        let dt = now - self.last_render_time;
        self.last_render_time = now;

        let elapsed_time = self.starting_time.elapsed().as_millis() as u32;
        uniforms.update_time(elapsed_time, queue);

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

struct Shader {
    contents: String,
    shader: ShaderModule
}

pub struct State<'surface> {
    pub gpu: Gpu<'surface>,
    pub pipeline: RenderPipeline,
    pub time: TimeKeeper,
    pub im_state: ImState,
    current_shader_path: String,
    current_shader: Shader
}

impl<'surface> State<'surface> {
    pub fn new(gpu: Gpu<'surface>, window: &Window) -> State<'surface> {
        let current_shader = std::fs::read_to_string(Path::new("shaders").join("shader.wgsl")).unwrap();
        let dummy_shader_src: Cow<'static, str> = "
struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
};

@vertex
fn vs_main() -> VertexOutput {
    var out: VertexOutput;
    out.pos = vec4(0.0,0.0,0.0,0.0);
    return out;
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4(0.0,0.0,0.0,0.0);
}
".into();
        let dummy_shader = gpu.device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(dummy_shader_src.clone()),
        }).unwrap();
        let shader = gpu.device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(current_shader.clone().into()),
        }).unwrap_or(gpu.device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(dummy_shader_src),
        }).unwrap());

        let time = TimeKeeper::new();
        let layout = gpu.device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("dummy pipeline layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        }).unwrap();
        let pipeline = gpu.device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("dummy pipeline"),
            layout: Some(&layout),
            vertex: VertexState {
                module: &dummy_shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            fragment: None,
            multiview: None,
        }).unwrap();

        let im_state = ImState::new(window, &gpu);
        let current_shader = Shader {
            contents: current_shader,
            shader: shader
        };
        let mut state = State {
            time,
            gpu,
            pipeline,
            im_state,
            current_shader_path: "shader.wgsl".into(),
            current_shader
        };
        state.refresh_pipeline();

        state
    }

    fn refresh_pipeline(&mut self) {
        let pipeline = self.recreate_pipeline();
        self.pipeline = pipeline;
    }

    fn recreate_pipeline(&mut self) -> RenderPipeline {
        let layout = self.get_pipeline_layout();
        match self.gpu.device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&layout),
            vertex: VertexState {
                module: &self.current_shader.shader,
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
                module: &self.current_shader.shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: self.gpu.config.format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            multiview: None,
        }) {
            Ok(pipeline) => pipeline,
            Err(err) => self.handle_pipeline_err(err),
        }
    }

    fn handle_pipeline_err(&mut self, err: CreateRenderPipelineError) -> RenderPipeline {
        match err {
            CreateRenderPipelineError::Stage { stage, error } => {
                if let ShaderStages::FRAGMENT = stage {
                    match error {
                        StageError::Binding(binding, error) => {
                            match error {
                                BindingError::Missing => {
                                    self.im_state.ui.inputs.define_binding(binding.group, binding.binding, &self.gpu.device)
                                },
                                BindingError::Invisible => todo!(),
                                BindingError::WrongType => todo!(),
                                BindingError::WrongAddressSpace { .. } => todo!(),
                                BindingError::WrongBufferSize(_) => todo!(),
                                BindingError::WrongTextureViewDimension { .. } => todo!(),
                                BindingError::WrongTextureClass { .. } => todo!(),
                                BindingError::WrongSamplerComparison => todo!(),
                                BindingError::InconsistentlyDerivedType => todo!(),
                                BindingError::BadStorageFormat(_) => todo!(),
                                BindingError::UnsupportedTextureStorageAccess(_) => todo!(),
                                _ => todo!(),
                            }
                        },
                        StageError::InvalidModule => todo!(),
                        StageError::InvalidWorkgroupSize { .. } => todo!(),
                        StageError::TooManyVaryings { .. } => todo!(),
                        StageError::MissingEntryPoint(_) => todo!(),
                        StageError::Filtering { .. } => todo!(),
                        StageError::Input { .. } => todo!(),
                        StageError::InputNotConsumed { .. } => todo!(),
                        _ => todo!(),
                    }
                } else {
                    panic!("Cannot add parameters used in places other than the fragment shader")
                }
            },
            CreateRenderPipelineError::ColorAttachment(_) => todo!(),
            CreateRenderPipelineError::Device(_) => todo!(),
            CreateRenderPipelineError::InvalidLayout => todo!(),
            CreateRenderPipelineError::Implicit(_) => todo!(),
            CreateRenderPipelineError::ColorState(_, _) => todo!(),
            CreateRenderPipelineError::DepthStencilState(_) => todo!(),
            CreateRenderPipelineError::InvalidSampleCount(_) => todo!(),
            CreateRenderPipelineError::TooManyVertexBuffers { .. } => todo!(),
            CreateRenderPipelineError::TooManyVertexAttributes { .. } => todo!(),
            CreateRenderPipelineError::VertexStrideTooLarge { .. } => todo!(),
            CreateRenderPipelineError::UnalignedVertexStride { .. } => todo!(),
            CreateRenderPipelineError::InvalidVertexAttributeOffset { .. } => todo!(),
            CreateRenderPipelineError::ShaderLocationClash(_) => todo!(),
            CreateRenderPipelineError::StripIndexFormatForNonStripTopology { .. } => todo!(),
            CreateRenderPipelineError::ConservativeRasterizationNonFillPolygonMode => todo!(),
            CreateRenderPipelineError::MissingFeatures(_) => todo!(),
            CreateRenderPipelineError::MissingDownlevelFlags(_) => todo!(),
            CreateRenderPipelineError::Internal { .. } => todo!(),
            CreateRenderPipelineError::UnalignedShader { .. } => todo!(),
            CreateRenderPipelineError::BlendFactorOnUnsupportedTarget { .. } => todo!(),
            CreateRenderPipelineError::PipelineExpectsShaderToUseDualSourceBlending => todo!(),
            CreateRenderPipelineError::ShaderExpectsPipelineToUseDualSourceBlending => todo!(),
            _ => todo!(),
        }

        self.recreate_pipeline()
    }

    pub fn refresh_shader(&mut self) {
        let shader_contents = std::fs::read_to_string(Path::new("shaders").join(&self.current_shader_path)).unwrap();
        match self.gpu.device.create_shader_module(ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(shader_contents.clone().into()),
        }) {
            Ok(shader) => {
                self.im_state.destroy_errors();
                self.current_shader.contents = shader_contents;
                self.current_shader.shader = shader;
                self.refresh_pipeline()
            },
            Err(err) => self.handle_shader_err(err),
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
        let mut layouts = vec![];
        for group in self.im_state.ui.inputs.groups.iter() {
            let bgl = group.bg_layout(&self.gpu.device);
            layouts.push(bgl)
        }

        let mut layout_refs = Vec::with_capacity(layouts.len());
        for l in layouts.iter() {
            layout_refs.push(l)
        }

        self.gpu.device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &layout_refs,
            push_constant_ranges: &[],
        }).unwrap()
    }

    fn handle_shader_err(&mut self, err: CreateShaderModuleError) {
        match err {
            e => self.im_state.show_crate_shader_err(e),
        }
    }

    pub(crate) fn handle_render_pass_err(&mut self, err: &RenderPassErrorInner) -> Option<Message> {
        match err {
            RenderPassErrorInner::Draw(err) => match err {
                DrawError::BindingSizeTooSmall(LateMinBufferBindingSizeMismatch {
                    group_index,
                    compact_index,
                    shader_size,
                    ..
                }) => {
                    self.im_state.ui.inputs.change_binding_size(
                        *group_index as usize,
                        *compact_index,
                        *shader_size,
                        &self.gpu.device,
                        &self.gpu.queue
                    );
                    Some(Message::ReloadPipeline)
                },
                _ => todo!(),
            },
            _ => None
        }
    }
}
