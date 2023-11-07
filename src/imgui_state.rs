use imgui::{Context, Ui};
use imgui_wgpu::{Renderer, RendererConfig};
use imgui_winit_support::{WinitPlatform, HiDpiMode};
use wgpu::{TextureView, CommandEncoder};
use winit::{window::Window as WinitWindow, event::Event};

use crate::state::Gpu;

struct UiState {

}

impl UiState {
    fn new() -> UiState {
        UiState {

        }
    }

    fn create_ui(&mut self, ui: &Ui) {
        ui.window("Aa").build(|| {});
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
        let mut platform = WinitPlatform::init(&mut context);
        platform.attach_window(context.io_mut(), window, HiDpiMode::Default);
        let renderer_config = RendererConfig {
            texture_format: gpu.config.format,
            ..Default::default()
        };
        let renderer = Renderer::new(&mut context, &gpu.device, &gpu.queue, renderer_config);

        let ui = UiState::new();
        ImState {
            context,
            platform,
            renderer,
            ui
        }
    }

    pub fn render(&mut self, window: &WinitWindow, gpu: &Gpu, view: &TextureView) -> CommandEncoder {
        self.platform
            .prepare_frame(self.context.io_mut(), window)
            .expect("Failed to prepare frame");
        let ui = self.context.frame();

        self.ui.create_ui(&ui);

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
        encoder
    }

    pub fn handle_event(&mut self, event: &Event<()>, window: &WinitWindow) {
        self.platform.handle_event(self.context.io_mut(), window, event);
    }
}
