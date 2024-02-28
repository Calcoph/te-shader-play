use std::error::Error;

use wgpu::{core::command::{RenderPassError, RenderPassErrorInner}, Color, CommandEncoder, CommandEncoderDescriptor, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor, StoreOp, SurfaceTexture, TextureView};
use winit::window::Window;

use crate::State;

pub fn render(output: SurfaceTexture, state: &mut State, window: &Window) {
    let handle_render_pass_err = |state: &mut State, err: Result<(), RenderPassError>| {
        if let Err(err) = err {
            if let Some(source) = err.source() {
                let source = source.downcast_ref::<RenderPassErrorInner>();
                if let Some(err) = source {
                    state.handle_render_pass_err(err)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    };

    let view = output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default())
        .unwrap();
    let mut encoder = state.gpu.device.create_command_encoder(&CommandEncoderDescriptor { label: None }).unwrap();
    let res = draw_image(state, &mut encoder, &view);
    let message = handle_render_pass_err(state, res);
    if let Some(message) = message {
        state.handle_message(message);
    }
    let (imgui_encoder, message) = state.im_state.render(window, &state.gpu, &view);
    if let Some(message) = message {
        state.handle_message(message);
    }
    let view = state.im_state.get_texture_view();
    let res = draw_image(state, &mut encoder, &view);
    let message = handle_render_pass_err(state, res);
    if let Some(message) = message {
        state.handle_message(message);
    }
    state.gpu.queue.submit(
        vec![
            encoder.finish(),
            imgui_encoder.finish()
        ].into_iter()
        .filter(|encoder| encoder.is_ok())
        .map(|encoder| encoder.unwrap())
    );
    output.present();
}

fn draw_image(state: &State, encoder: &mut CommandEncoder, view: &TextureView) -> Result<(), RenderPassError> {
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
    for (g_index, group) in state.im_state.ui.inputs.groups.iter().enumerate() {
        render_pass.set_bind_group(g_index as u32, &group.bind_group, &[]);
    }

    render_pass.draw(0..3, 0..2);
    render_pass.encode()
}
