use std::error::Error;

use wgpu::{
    core::command::{RenderPassError, RenderPassErrorInner}, CommandEncoder, CommandEncoderDescriptor, IndexFormat, LoadOp, Operations, RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor, StoreOp, SurfaceTexture, TextureView, TextureViewDescriptor
};
use winit::window::{Window, WindowLevel};

use crate::{imgui_state::Message, State};

pub(crate) enum RenderMessage {
    ChangeWindowLevel(WindowLevel),
}

pub fn render(output: SurfaceTexture, state: &mut State, window: &Window) {
    let handle_render_pass_err = |state: &mut State, err: Result<(), RenderPassError>| {
        if let Err(err) = err {
            if let Some(source) = err.source() {
                let source = source.downcast_ref::<RenderPassErrorInner>();
                if let Some(err) = source {
                    state.handle_render_pass_err(err)
                } else {
                    panic!("Error")
                }
            } else {
                panic!("Error")
            }
        } else {
            None
        }
    };

    let view = output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default())
        .unwrap();
    let mut encoder1 = state
        .gpu
        .device
        .create_command_encoder(&CommandEncoderDescriptor { label: None })
        .unwrap();
    let depth_view = state.depth_textures.background.create_view(&TextureViewDescriptor::default()).unwrap();
    let res = draw_image(state, &mut encoder1, &view, &depth_view);
    let message = handle_render_pass_err(state, res);
    handle_message(state, message, window);
    let (imgui_encoder, message) = state.im_state.render(window, &state.gpu, &view);
    handle_message(state, message, window);
    let view = state.im_state.get_texture_view();
    let depth_view = state.depth_textures.imgui.create_view(&TextureViewDescriptor::default()).unwrap();
    let mut encoder2 = state
        .gpu
        .device
        .create_command_encoder(&CommandEncoderDescriptor { label: None })
        .unwrap();
    let res = draw_image(state, &mut encoder2, view, &depth_view);
    let message = handle_render_pass_err(state, res);
    handle_message(state, message, window);
    state.gpu.queue.submit(
        vec![encoder1.finish(), encoder2.finish(), imgui_encoder.finish()]
            .into_iter()
            .filter_map(|encoder| encoder.ok()),
    );
    output.present();
}

fn handle_message(state: &mut State, message: Option<Message>, window: &Window) {
    if let Some(message) = message {
        if let Some(message) = state.handle_message(message) {
            match message {
                RenderMessage::ChangeWindowLevel(window_level) => {
                    window.set_window_level(window_level)
                }
            }
        }
    }
}

fn draw_image(
    state: &State,
    encoder: &mut CommandEncoder,
    view: &TextureView,
    depth_view: &TextureView,
) -> Result<(), RenderPassError> {
    draw_custom_shader(state, encoder, view, &depth_view)?;
    if state.im_state.ui.draw_grid {
        draw_grid(state, encoder, view, &depth_view)
    } else {
        Ok(())
    }
}

fn draw_grid(
    state: &State,
    encoder: &mut CommandEncoder,
    view: &TextureView,
    depth_view: &TextureView,
) -> Result<(), RenderPassError> {
    assert!(state.im_state.ui.draw_grid);
    let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
        label: None,
        color_attachments: &[Some(RenderPassColorAttachment {
            view,
            resolve_target: None,
            ops: Operations {
                load: LoadOp::Load,
                store: StoreOp::Store,
            },
        })],
        depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
            view: depth_view,
            depth_ops: Some(Operations {
                load: LoadOp::Load,
                store: StoreOp::Store
            }),
            stencil_ops: None,
        }),
        timestamp_writes: None,
        occlusion_query_set: None,
    }).unwrap();
    render_pass.set_pipeline(&state.pipelines.grid);
    for (g_index, group) in state.im_state.ui.inputs.groups.iter().enumerate() {
        render_pass.set_bind_group(g_index as u32, &group.bind_group, &[]);
    }

    render_pass.set_vertex_buffer(0, state.vertices.grid.vertex_buffer.slice(..));
    render_pass.set_index_buffer(state.vertices.grid.index_buffer.slice(..), IndexFormat::Uint32);
    render_pass.draw_indexed(0..state.vertices.grid.indices.len() as u32, 0, 0..1);
    render_pass.end()
}

fn draw_custom_shader(
    state: &State,
    encoder: &mut CommandEncoder,
    view: &TextureView,
    depth_view: &TextureView,
) -> Result<(), RenderPassError> {
    let background_color = state.get_background_color();
    let ops = Operations {
        load: LoadOp::Clear(background_color),
        store: StoreOp::Store,
    };
    let depth_stencil_attachment = Some(RenderPassDepthStencilAttachment {
        view: depth_view,
        depth_ops: Some(Operations {
            load: LoadOp::Clear(1.0),
            store: StoreOp::Store,
        }),
        stencil_ops: None,
    });

    let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
        label: None,
        color_attachments: &[Some(RenderPassColorAttachment {
            view,
            resolve_target: None,
            ops,
        })],
        depth_stencil_attachment,
        timestamp_writes: None,
        occlusion_query_set: None,
    }).unwrap();
    render_pass.set_pipeline(&state.pipelines.custom_shader).unwrap();
    for (g_index, group) in state.im_state.ui.inputs.groups.iter().enumerate() {
        render_pass.set_bind_group(g_index as u32, &group.bind_group, &[]);
    }

    render_pass.set_vertex_buffer(0, state.vertices.custom_shader.vertex_buffer.slice(..)).unwrap();
    render_pass.set_index_buffer(state.vertices.custom_shader.index_buffer.slice(..), IndexFormat::Uint32).unwrap();
    render_pass.draw_indexed(0..state.vertices.custom_shader.indices.len() as u32, 0, 0..1).unwrap();
    render_pass.end()
}
