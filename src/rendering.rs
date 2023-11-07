use wgpu::{SurfaceTexture, CommandEncoderDescriptor, RenderPassDescriptor, RenderPassColorAttachment, Operations, LoadOp, Color, StoreOp, util::RenderEncoder, CommandEncoder, TextureView};
use winit::window::Window;

use crate::State;

pub fn render(output: SurfaceTexture, state: &mut State, window: &Window) {
    let view = output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = state.gpu.device.create_command_encoder(&CommandEncoderDescriptor { label: None });
    draw_image(state, &mut encoder, &view);
    let imgui_encoder = state.im_state.render(window, &state.gpu, &view);
    let view = state.im_state.get_texture_view();
    draw_image(state, &mut encoder, view);
    state.gpu.queue.submit(vec![
        encoder.finish(),
        imgui_encoder.finish()
    ].into_iter());
    output.present();
}

fn draw_image(state: &State, encoder: &mut CommandEncoder, view: &TextureView) {
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
