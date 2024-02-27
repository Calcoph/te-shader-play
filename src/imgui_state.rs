use std::{borrow::Cow, path::Path};

use imgui::{ConfigFlags, Context, Image, StyleVar, TextureId, TreeNodeFlags, Ui};
use imgui_wgpu::{Renderer, RendererConfig, Texture as ImTexture, TextureConfig};
use imgui_winit_support::{WinitPlatform, HiDpiMode};
use wgpu::{core::pipeline::CreateShaderModuleError, util::{BufferInitDescriptor, DeviceExt}, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferUsages, CommandEncoder, Device, Queue, ShaderStages, TextureView};
use winit::{window::Window as WinitWindow, event::Event};

use crate::state::Gpu;

const IMAGE_HEIGHT: f32 = 512.0;
const IMAGE_WIDTH: f32 = 512.0;

const DEFAULT_U32_UNIFORM: u32 = 0;
const DEFAULT_I32_UNIFORM: i32 = 0;
const DEFAULT_F32_UNIFORM: f32 = 0.0;

pub enum Message {
    ReloadShader,
    LoadShader(String),
    ReloadPipeline
}

enum UniformEditEvent {
    EditU32(usize, usize, u32),
    EditI32(usize, usize, i32),
    EditF32(usize, usize, f32),
    AddU32(usize),
    AddI32(usize),
    AddF32(usize),
    AddBindGroup,
    ChangeType(UniformType, usize, usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BuiltinValue {
    Time
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum UniformValue {
    BuiltIn(BuiltinValue),
    U32(u32),
    I32(i32),
    F32(f32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UniformType {
    U32,
    I32,
    F32
}

impl<'a> Into<Cow<'a, str>> for &'a UniformType {
    fn into(self) -> Cow<'static, str> {
        match self {
            UniformType::U32 => Cow::Borrowed("u32"),
            UniformType::I32 => Cow::Borrowed("i32"),
            UniformType::F32 => Cow::Borrowed("f32"),
        }
    }
}

impl UniformValue {
    fn show_editor(&mut self, ui: &Ui, group_index: usize, binding_index: usize, val_name: &mut String) -> Option<UniformEditEvent> {
        const TYPES: &[UniformType] = &[
            UniformType::U32,
            UniformType::I32,
            UniformType::F32,
        ];
        const COMBO_WIDTH: f32 = 50.0;
        const VAR_NAME_WIDTH: f32 = 150.0;

        let show_primitive_selector = |message: &mut Option<UniformEditEvent>, type_index, val_name: &mut String| {
            ui.text(format!("({binding_index})"));
            ui.same_line();
            ui.set_next_item_width(VAR_NAME_WIDTH);
            ui.input_text(format!("##name_edit{group_index}_{binding_index}"), val_name).build();
            ui.set_next_item_width(COMBO_WIDTH);
            let mut selection = type_index;
            if ui.combo(
                format!("##combo_g{group_index}_b{binding_index}"),
                &mut selection,
                TYPES,
                |unitype| unitype.into()
            ) {
                let selected_type = TYPES[selection];
                if selected_type != TYPES[type_index] {
                    *message = Some(UniformEditEvent::ChangeType(selected_type, group_index, binding_index))
                }
            };
        };

        let mut message = None;
        match self {
            UniformValue::BuiltIn(builtin) => match builtin {
                BuiltinValue::Time => ui.text(format!("({binding_index}) Time")),
            },
            UniformValue::U32(v) => {
                show_primitive_selector(&mut message, 0, val_name);
                ui.same_line();
                if ui.input_scalar(format!("##editor{group_index}_{binding_index}"), v).build() {
                    message = Some(UniformEditEvent::EditU32(group_index, binding_index, *v));
                }
            },
            UniformValue::I32(v) => {
                show_primitive_selector(&mut message, 1, val_name);
                ui.same_line();
                if ui.input_int(format!("##editor{group_index}_{binding_index}"), v).build() {
                    message = Some(UniformEditEvent::EditI32(group_index, binding_index, *v))
                }
            },
            UniformValue::F32(v) => {
                show_primitive_selector(&mut message, 2, val_name);
                ui.same_line();
                if ui.input_float(format!("##editor{group_index}_{binding_index}"), v).build() {
                    message = Some(UniformEditEvent::EditF32(group_index, binding_index, *v))
                }
            },
        };

        message
    }
}

struct UniformBinding {
    pub buffer: Buffer,
    value: UniformValue,
    name: String
}
impl UniformBinding {
    fn bgl_entry(&self, index: u32) -> BindGroupLayoutEntry {
        BindGroupLayoutEntry {
            binding: index,
            visibility: ShaderStages::FRAGMENT,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None
            },
            count: None
        }
    }

    fn bg_entry(&self, index: u32) -> BindGroupEntry<'_> {
        BindGroupEntry {
            binding: index,
            resource: self.buffer.as_entire_binding(),
        }
    }

    fn new(device: &Device, value: UniformValue) -> UniformBinding {
        let contents = match value {
            UniformValue::BuiltIn(builtin) => match builtin {
                BuiltinValue::Time => 0u32.to_be_bytes(),
            },
            UniformValue::U32(v) => v.to_le_bytes(),
            UniformValue::I32(v) => v.to_le_bytes(),
            UniformValue::F32(v) => v.to_le_bytes(),
        };

        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: &contents,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        }).unwrap();

        UniformBinding {
            buffer: buffer,
            value,
            name: "unnamed".to_string()
        }
    }

    fn change_type(&mut self, new_type: UniformType, queue: &Queue) {
        let old_value = self.value;
        let new_value = match (old_value, new_type) {
            (UniformValue::U32(v), UniformType::I32) => {
                let v = v as i32;
                let new_value = UniformValue::I32(v);
                // Don't have to update buffer
                new_value
            },
            (UniformValue::U32(v), UniformType::F32) => {
                let v = v as f32;
                let new_value = UniformValue::F32(v);
                // Have to update buffer
                queue.write_buffer(&self.buffer, 0, &v.to_le_bytes()).unwrap();
                new_value
            },

            (UniformValue::I32(v), UniformType::U32) => {
                let res = v.try_into();
                let v = res.unwrap_or(DEFAULT_U32_UNIFORM);
                let new_value = UniformValue::U32(v);
                if let Err(_) = res {
                    // Have to update buffer
                    queue.write_buffer(&self.buffer, 0, &v.to_le_bytes()).unwrap();
                }
                new_value
            },
            (UniformValue::I32(v), UniformType::F32) => {
                let v = v as f32;
                let new_value = UniformValue::F32(v);
                // Have to update buffer
                queue.write_buffer(&self.buffer, 0, &v.to_le_bytes()).unwrap();
                new_value
            },

            (UniformValue::F32(v), UniformType::U32) => {
                let v = (v as i32).try_into();
                let v = v.unwrap_or(DEFAULT_U32_UNIFORM);
                let new_value = UniformValue::U32(v);
                // Have to update buffer
                queue.write_buffer(&self.buffer, 0, &v.to_le_bytes()).unwrap();
                new_value
            },
            (UniformValue::F32(v), UniformType::I32) => {
                let v = v as i32;
                let new_value = UniformValue::I32(v);
                // Have to update buffer
                queue.write_buffer(&self.buffer, 0, &v.to_le_bytes()).unwrap();
                new_value
            },

            (UniformValue::I32(_), UniformType::I32) => unreachable!(),
            (UniformValue::F32(_), UniformType::F32) => unreachable!(),
            (UniformValue::U32(_), UniformType::U32) => unreachable!(),
            (UniformValue::BuiltIn(_), _) => unreachable!(),
        };

        self.value = new_value
    }

    fn show_editor(&mut self, ui: &Ui, group_index: usize, binding_index: usize) -> Option<UniformEditEvent> {
        self.value.show_editor(ui, group_index, binding_index, &mut self.name)
    }
}

pub struct UniformGroup {
    bindings: Vec<UniformBinding>,
    pub bind_group: BindGroup,
}

impl UniformGroup {
    fn new(device: &Device) -> UniformGroup{
        let bg = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Automaticall created layout in new"),
                entries: &[],
            }).unwrap(),
            entries: &[],
        }).unwrap();

        UniformGroup {
            bindings: Vec::new(),
            bind_group: bg,
        }
    }

    pub fn bg_layout(&self, device: &Device) -> BindGroupLayout {
        let mut entries = Vec::new();
        for (index, binding) in self.bindings.iter().enumerate() {
            entries.push(binding.bgl_entry(index as u32))
        }

        device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Automatically created layout in bg_layout"),
            entries: &entries,
        }).unwrap()
    }

    fn add_u32(&mut self, device: &Device) {
        self.bindings.push(UniformBinding::new(device, UniformValue::U32(DEFAULT_U32_UNIFORM)));
        self.refresh_bind_group(device)
    }

    fn add_i32(&mut self, device: &Device) {
        self.bindings.push(UniformBinding::new(device, UniformValue::I32(DEFAULT_I32_UNIFORM)));
        self.refresh_bind_group(device)
    }

    fn add_f32(&mut self, device: &Device) {
        self.bindings.push(UniformBinding::new(device, UniformValue::F32(DEFAULT_F32_UNIFORM)));
        self.refresh_bind_group(device)
    }

    fn add_custom(&mut self, device: &Device, uniform: UniformValue) {
        self.bindings.push(UniformBinding::new(device, uniform));
        self.refresh_bind_group(device)
    }

    fn edit_u32(&mut self, b_index: usize, value: u32, queue: &Queue) {
        let binding = &mut self.bindings[b_index];
        queue.write_buffer(&binding.buffer, 0, &value.to_le_bytes()).unwrap();
        binding.value = UniformValue::U32(value)
    }

    fn edit_i32(&mut self, b_index: usize, value: i32, queue: &Queue) {
        let binding = &mut self.bindings[b_index];
        queue.write_buffer(&binding.buffer, 0, &value.to_le_bytes()).unwrap();
        binding.value = UniformValue::I32(value)
    }

    fn edit_f32(&mut self, b_index: usize, value: f32, queue: &Queue) {
        let binding = &mut self.bindings[b_index];
        queue.write_buffer(&binding.buffer, 0, &value.to_le_bytes()).unwrap();
        binding.value = UniformValue::F32(value)
    }

    fn refresh_bind_group(&mut self, device: &Device) {
        let mut layout_entries = Vec::new();
        let mut bindgroup_entries = Vec::new();
        for (index, binding) in self.bindings.iter().enumerate() {
            layout_entries.push(binding.bgl_entry(index as u32));
            bindgroup_entries.push(binding.bg_entry(index as u32));
        };

        let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("Autogenerated bind group layout in refresh_bind_group"),
            entries: &layout_entries,
        }).unwrap();
        let bg = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Autogenerated bind group in refresh_bind_group"),
            layout: &layout,
            entries: &bindgroup_entries,
        }).unwrap();

        self.bind_group = bg;
    }

    fn define_binding(&mut self, binding: u32, device: &Device) {
        while binding >= self.bindings.len() as u32 {
            self.add_f32(device)
        }
    }

    fn change_type(&mut self, unitype: UniformType, b_index: usize, queue: &Queue) {
        self.bindings[b_index].change_type(unitype, queue)
    }
}

pub struct Uniforms {
    pub groups: Vec<UniformGroup>,
    time_uniform_location: (usize, usize)
}

impl Uniforms {
    fn new(device: &Device) -> Uniforms {
        let mut group0 = UniformGroup::new(device);
        group0.add_custom(device, UniformValue::BuiltIn(BuiltinValue::Time));
        let time_uniform_location = (0, 0);
        Uniforms {
            groups: vec![group0],
            time_uniform_location
        }
    }

    fn add_u32(&mut self, g_index: usize, device: &Device) {
        self.groups[g_index].add_u32(device)
    }

    fn add_i32(&mut self, g_index: usize, device: &Device) {
        self.groups[g_index].add_i32(device)
    }

    fn add_f32(&mut self, g_index: usize, device: &Device) {
        self.groups[g_index].add_f32(device)
    }

    fn edit_u32(&mut self, g_index: usize, b_index: usize, value: u32, queue: &Queue) {
        self.groups[g_index].edit_u32(b_index, value, queue)
    }
    fn edit_i32(&mut self, g_index: usize, b_index: usize, value: i32, queue: &Queue) {
        self.groups[g_index].edit_i32(b_index, value, queue)
    }
    fn edit_f32(&mut self, g_index: usize, b_index: usize, value: f32, queue: &Queue) {
        self.groups[g_index].edit_f32(b_index, value, queue)
    }

    fn add_bind_group(&mut self, device: &Device) {
        self.groups.push(UniformGroup::new(device))
    }

    pub(crate) fn update_time(&self, elapsed_time: u32, queue: &Queue) {
        let (g_index, b_index) = self.time_uniform_location;
        let time_binding = &self.groups[g_index].bindings[b_index];
        assert!(time_binding.value == UniformValue::BuiltIn(BuiltinValue::Time));

        queue.write_buffer(
            &time_binding.buffer,
            0,
            &elapsed_time.to_le_bytes()
        ).unwrap();
    }

    pub(crate) fn define_binding(&mut self, group: u32, binding: u32, device: &Device) {
        while group >= self.groups.len() as u32 {
            self.add_bind_group(device)
        }

        self.groups[group as usize].define_binding(binding, device);
    }

    fn change_type(&mut self, unitype: UniformType, g_index: usize, b_index: usize, queue: &Queue) {
        self.groups[g_index].change_type(unitype, b_index, queue)
    }
}

pub struct UiState {
    pub texture_id: TextureId,
    shader_name: String,
    shader_exists: bool,
    pub inputs: Uniforms,
    errors: Vec<String>,
    show_errors: bool
}

impl UiState {
    fn new(texture_id: TextureId, device: &Device) -> UiState {
        UiState {
            texture_id,
            shader_name: "shader.wgsl".to_string(),
            shader_exists: true,
            inputs: Uniforms::new(device),
            errors: vec![],
            show_errors: false
        }
    }

    fn create_ui(&mut self, ui: &Ui, device: &Device, queue: &Queue) -> Option<Message> {
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
            let mut edit_event = None;
            for (group_index, group) in self.inputs.groups.iter_mut().enumerate() {
                if ui.collapsing_header(format!("Binding group {group_index}"), TreeNodeFlags::empty()) {
                    for (binding_index, uniform) in group.bindings.iter_mut().enumerate() {
                        if let Some(event) = uniform.show_editor(ui, group_index, binding_index) {
                            edit_event = Some(event);
                        }
                        ui.separator();
                    }
                    if ui.button(format!("Add u32##add_u32{group_index}")) {
                        edit_event = Some(UniformEditEvent::AddU32(group_index))
                    };
                    ui.same_line();
                    if ui.button(format!("Add i32##add_i32{group_index}")) {
                        edit_event = Some(UniformEditEvent::AddI32(group_index))
                    };
                    ui.same_line();
                    if ui.button(format!("Add f32##add_f32{group_index}")) {
                        edit_event = Some(UniformEditEvent::AddF32(group_index))
                    };
                }
            }

            ui.separator();
            if ui.button("Add Bind Group") {
                edit_event = Some(UniformEditEvent::AddBindGroup)
            }

            if let Some(event) = edit_event {
                match event {
                    UniformEditEvent::EditU32(g_index, b_index, value) => self.inputs.edit_u32(g_index, b_index, value, queue),
                    UniformEditEvent::EditI32(g_index, b_index, value) => self.inputs.edit_i32(g_index, b_index, value, queue),
                    UniformEditEvent::EditF32(g_index, b_index, value) => self.inputs.edit_f32(g_index, b_index, value, queue),
                    UniformEditEvent::AddU32(g_index) => self.inputs.add_u32(g_index, device),
                    UniformEditEvent::AddI32(g_index) => self.inputs.add_i32(g_index, device),
                    UniformEditEvent::AddF32(g_index) => self.inputs.add_f32(g_index, device),
                    UniformEditEvent::AddBindGroup => self.inputs.add_bind_group(device),
                    UniformEditEvent::ChangeType(unitype, g_index, b_index) => self.inputs.change_type(unitype, g_index, b_index, queue),
                };
                message = Some(Message::ReloadPipeline);
            }
        });

        ui.window("Errors").focused(self.show_errors).build(|| {
            self.show_errors = false;
            for error in self.errors.iter() {
                ui.text_wrapped(error)
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
    pub ui: UiState,
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

        let ui = UiState::new(texture_id, &gpu.device);
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

        let message = self.ui.create_ui(&ui, &gpu.device, &gpu.queue);

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

    pub(crate) fn destroy_errors(&mut self) {
        self.ui.errors = Vec::new();
        self.ui.show_errors = false;
    }

    pub(crate) fn show_crate_shader_err(&mut self, err: CreateShaderModuleError) {
        self.ui.show_errors = true;
        self.ui.errors = vec![
            err.to_string()
        ]
    }
}
