use std::path::Path;

use imgui::{Context, Ui, ConfigFlags, Image, TextureId, StyleVar};
use imgui_wgpu::{Renderer, RendererConfig, Texture as ImTexture, TextureConfig};
use imgui_winit_support::{WinitPlatform, HiDpiMode};
use wgpu::{TextureView, CommandEncoder};
use winit::{window::Window as WinitWindow, event::Event};

use crate::state::Gpu;

const IMAGE_HEIGHT: f32 = 512.0;
const IMAGE_WIDTH: f32 = 512.0;

pub enum Message {
    ReloadShader,
    LoadShader(String)
}

pub struct UiState {
    pub texture_id: TextureId,
    shader_name: String,
    shader_exists: bool
}

impl UiState {
    fn new(texture_id: TextureId) -> UiState {
        UiState {
            texture_id,
            shader_name: "shader.wgsl".to_string(),
            shader_exists: true
        }
    }

    fn create_ui(&mut self, ui: &Ui) -> Option<Message> {
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
    ui: UiState
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

        let message = self.ui.create_ui(&ui);

        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ImGui Render Encoder"),
            });
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
