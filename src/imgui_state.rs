use std::path::Path;

use imgui::{Context, Ui, ConfigFlags, Image, TextureId, StyleVar};
use imgui_wgpu::{Renderer, RendererConfig, Texture as ImTexture, TextureConfig};
use imgui_winit_support::{WinitPlatform, HiDpiMode};
use wgpu::{TextureView, CommandEncoder, Device, util::{DeviceExt, BufferInitDescriptor}, BufferUsages, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BufferBindingType, ShaderStages, BindGroupDescriptor, BindGroupEntry};
use winit::{window::Window as WinitWindow, event::Event};

use crate::state::{Gpu, BoundBuffer};

const IMAGE_HEIGHT: f32 = 512.0;
const IMAGE_WIDTH: f32 = 512.0;

pub enum Message {
    ReloadShader,
    LoadShader(String),
    ReloadPipeline
}

pub struct Inputs {
    pub ints: Vec<(i32, BoundBuffer)>,
    pub floats: Vec<(f32, BoundBuffer)>
}

impl Inputs {
    fn new() -> Inputs {
        Inputs {
            ints: vec![],
            floats: vec![],
        }
    }

    fn add_int(&mut self, device: &Device) {
        let new_int: i32 = 0;
        let bb = Self::int_bound_buffer(device, new_int);

        self.ints.push((
            new_int,
            bb
        ))
    }

    fn add_float(&mut self, device: &Device) {
        let new_float: f32 = 0.0;
        let bb = Self::float_bound_buffer(device, new_float);

        self.floats.push((
            new_float,
            bb
        ))
    }

    fn int_bound_buffer(device: &Device, new_int: i32) -> BoundBuffer {
        let buffer = device.create_buffer_init(&BufferInitDescriptor{
            label: None,
            contents: &new_int.to_le_bytes(),
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
                    resource: buffer.as_entire_binding(),
                }
            ],
        }).unwrap();

        BoundBuffer { buffer, bg_layout, bg }
    }

    fn float_bound_buffer(device: &Device, new_float: f32) -> BoundBuffer {
        let buffer = device.create_buffer_init(&BufferInitDescriptor{
            label: None,
            contents: &new_float.to_le_bytes(),
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
                    resource: buffer.as_entire_binding(),
                }
            ],
        }).unwrap();

        BoundBuffer { buffer, bg_layout, bg }
    }

    fn edit_int(&mut self, index: usize, int: i32, device: &Device) {
        let bb = Self::int_bound_buffer(device, int);

        self.ints[index] = (int, bb);
    }

    fn edit_float(&mut self, index: usize, float: f32, device: &Device) {
        let bb = Self::float_bound_buffer(device, float);

        self.floats[index] = (float, bb);
    }
}

pub struct UiState {
    pub texture_id: TextureId,
    shader_name: String,
    shader_exists: bool,
    pub inputs: Inputs
}

impl UiState {
    fn new(texture_id: TextureId) -> UiState {
        UiState {
            texture_id,
            shader_name: "shader.wgsl".to_string(),
            shader_exists: true,
            inputs: Inputs::new(),
        }
    }

    fn create_ui(&mut self, ui: &Ui, device: &Device) -> Option<Message> {
        let mut message = None;
        ui.dockspace_over_main_viewport();
        ui.window("Render").build(|| {
            let a = ui.push_style_var(StyleVar::FrameBorderSize(50.0));
            Image::new(self.texture_id, mint::Vector2{ x: IMAGE_WIDTH, y: IMAGE_HEIGHT }).border_col([1.0;4]).build(ui);
            a.pop()
        });

        ui.window("Control").build(|| {
            if ui.button("Reload shader") {
                message = Some(Message::ReloadShader)
            };
            ui.separator();
            if ui.input_text("Shader file", &mut self.shader_name).build() {
                self.check_shader_exists()
            };
            ui.disabled(!self.shader_exists, || {
                if ui.button("Load") {
                    message = Some(Message::LoadShader(self.shader_name.clone()))
                };
            });
            if !self.shader_exists {
                ui.text(format!("shaders/{} doesn't exist", self.shader_name));
            }
        });

        ui.window("Shader parameters").build(|| {
            if ui.button("add int") {
                self.inputs.add_int(device);
                message = Some(Message::ReloadPipeline)
            }
            let mut bg_index = 1;
            let mut edit_int = None;
            for (index, (int, _)) in self.inputs.ints.iter_mut().enumerate() {
                if ui.input_int(bg_index.to_string(), int).build() {
                    edit_int = Some((index, *int));
                }
                bg_index += 1;
            }
            if let Some((index, int)) = edit_int {
                self.inputs.edit_int(index, int, device);
                message = Some(Message::ReloadPipeline)
            }
            ui.separator();
            if ui.button("add float") {
                self.inputs.add_float(device);
                message = Some(Message::ReloadPipeline)
            }
            let mut edit_float = None;
            for (index, (float, _)) in self.inputs.floats.iter_mut().enumerate() {
                if ui.input_float(bg_index.to_string(), float).build() {
                    edit_float = Some((index, *float))
                }
                bg_index += 1;
            }
            if let Some((index, float)) = edit_float {
                self.inputs.edit_float(index, float, device);
                message = Some(Message::ReloadPipeline)
            }
        });

        message
    }

    fn check_shader_exists(&mut self) {
        let path = Path::new("shaders").join(&self.shader_name);
        self.shader_exists = path.exists();
    }
}

pub struct ImState {
    context: Context,
    platform: WinitPlatform,
    renderer: Renderer,
    pub ui: UiState
}

impl ImState {
    pub fn new(window: &WinitWindow, gpu: &Gpu) -> ImState {
        let mut context = Context::create();
        context.io_mut().config_flags |= ConfigFlags::DOCKING_ENABLE;
        let mut platform = WinitPlatform::init(&mut context);
        platform.attach_window(context.io_mut(), window, HiDpiMode::Default);
        let renderer_config = RendererConfig {
            texture_format: gpu.config.format,
            ..Default::default()
        };
        let mut renderer = Renderer::new(&mut context, &gpu.device, &gpu.queue, renderer_config);

        let texture = ImTexture::new(&gpu.device, &renderer, TextureConfig {
            size: wgpu::Extent3d {
                width: IMAGE_WIDTH as u32,
                height: IMAGE_HEIGHT as u32,
                ..Default::default()
            },
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            ..Default::default()
        });
        let texture_id = renderer.textures.insert(texture);

        let ui = UiState::new(texture_id);
        ImState {
            context,
            platform,
            renderer,
            ui,
        }
    }

    pub fn render(&mut self, window: &WinitWindow, gpu: &Gpu, view: &TextureView) -> (CommandEncoder, Option<Message>) {
        self.platform
            .prepare_frame(self.context.io_mut(), window)
            .expect("Failed to prepare frame");
        let ui = self.context.frame();

        let message = self.ui.create_ui(&ui, &gpu.device);

        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ImGui Render Encoder"),
            }).unwrap();
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None
            });
            self.renderer
                .render(self.context.render(), &gpu.queue, &gpu.device, &mut render_pass)
                .expect("Rendering failed");
        }
        (encoder, message)
    }

    pub fn handle_event(&mut self, event: &Event<()>, window: &WinitWindow) {
        self.platform.handle_event(self.context.io_mut(), window, event);
    }

    pub fn get_texture_view(&self) -> &TextureView {
        self.renderer.textures.get(self.ui.texture_id).unwrap().view()
    }
}
