use std::{array::IntoIter, iter::Chain, path::Path};

use cgmath::{Deg, Matrix4, Point3, Rad, Vector4};
use imgui::{ConfigFlags, Context, Image, StyleVar, TextureId, TreeNodeFlags, Ui};
use imgui_wgpu::{Renderer, RendererConfig, Texture as ImTexture, TextureConfig};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use serde_json::{Map, Value as JsonValue};
use wgpu::{
    core::pipeline::CreateShaderModuleError, util::{BufferInitDescriptor, DeviceExt}, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferUsages, CommandEncoder, Device, Queue, ShaderStages, TextureView
};
use winit::{
    event::Event,
    window::{Window as WinitWindow, WindowLevel},
};

use crate::{imgui_state::uniform_types::VecType, state::Gpu};

use uniform_types::UniformType;

use self::uniform_types::{BuiltinValue, MatrixType, ScalarType, ScalarUniformValue, UniformValue};

mod uniform_types;

pub const IMAGE_HEIGHT: f32 = 512.0;
pub const IMAGE_WIDTH: f32 = 512.0;

const DEFAULT_U32_UNIFORM: u32 = 0;
const DEFAULT_UNIFORM: UniformValue = UniformValue::Scalar(ScalarUniformValue::F32(0.0));

trait ImguiScalar {
    fn increase(&mut self);
    fn decrease(&mut self);
}

trait ImguiVec {
    fn change_inner_type(&mut self, inner_type: ScalarType);
}

trait ImguiMatrix {
    fn change_matrix_size(&mut self, matrix_size: MatrixType);
}

trait ImguiUniformSelectable {
    fn cast_to(&self, casted_type: UniformType) -> UniformValue;
    fn show_editor(
        &mut self,
        ui: &Ui,
        group_index: usize,
        binding_index: usize,
        val_name: &mut String,
    ) -> Option<UniformEditEvent>;
    fn to_le_bytes(&self) -> Vec<u8>;
}

pub enum Message {
    ReloadShader,
    LoadShader(String),
    ReloadPipeline,
    ReloadMeshBuffers,
    ChangeWindowLevel(WindowLevel),
    SaveParameters,
}

enum UniformEditEvent {
    UpdateBuffer(usize, usize),
    AddUniform(usize),
    AddBindGroup,
    ChangeType(UniformType, usize, usize),
    Increase(usize, usize),
    Decrease(usize, usize),
    ChangeInnerType(ScalarType, usize, usize),
    ChangeMatrixSize(MatrixType, usize, usize),
}
struct UniformBinding {
    pub buffer: Buffer,
    value: UniformValue,
    name: String,
}
impl UniformBinding {
    fn bgl_entry(&self, index: u32) -> BindGroupLayoutEntry {
        BindGroupLayoutEntry {
            binding: index,
            visibility: ShaderStages::VERTEX_FRAGMENT,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }

    fn bg_entry(&self, index: u32) -> BindGroupEntry<'_> {
        BindGroupEntry {
            binding: index,
            resource: self.buffer.as_entire_binding(),
        }
    }

    fn new(device: &Device, value: UniformValue) -> UniformBinding {
        let contents = value.to_le_bytes();

        let buffer = device
            .create_buffer_init(&BufferInitDescriptor {
                label: Some("new uniform buffer"),
                contents: &contents,
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            })
            .unwrap();

        UniformBinding {
            buffer,
            value,
            name: "unnamed".to_string(),
        }
    }

    fn change_type(&mut self, new_type: UniformType, queue: &Queue, device: &Device) {
        let old_value = self.value;
        let old_size = old_value.to_le_bytes().len();
        let new_value = old_value.cast_to(new_type);

        self.value = new_value;
        let new_bytes = self.value.to_le_bytes();
        if new_bytes.len() != old_size {
            self.buffer = device
                .create_buffer_init(&BufferInitDescriptor {
                    label: Some("Resized buffer"),
                    contents: &new_bytes,
                    usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
                })
                .unwrap();
        } else {
            queue.write_buffer(&self.buffer, 0, &new_bytes).unwrap();
        }
    }

    fn show_editor(
        &mut self,
        ui: &Ui,
        group_index: usize,
        binding_index: usize,
    ) -> Option<UniformEditEvent> {
        self.value
            .show_editor(ui, group_index, binding_index, &mut self.name)
    }

    fn decrease(&mut self, queue: &Queue) {
        self.value.decrease();

        let new_value = self.value.to_le_bytes();
        queue.write_buffer(&self.buffer, 0, &new_value).unwrap();
    }

    fn increase(&mut self, queue: &Queue) {
        self.value.increase();

        let new_value = self.value.to_le_bytes();
        queue.write_buffer(&self.buffer, 0, &new_value).unwrap();
    }

    fn change_inner_type(&mut self, inner_type: ScalarType, queue: &Queue) {
        self.value.change_inner_type(inner_type);
        let new_value = self.value.to_le_bytes();
        queue.write_buffer(&self.buffer, 0, &new_value).unwrap();
    }

    fn change_binding_size(&mut self, new_size: u64, device: &Device, queue: &Queue) {
        /*
        Matrix sizes:
        2x2: 4*f32 = 4*4 = 16
        2x3: 6*f32 = 6*4 = 24
        3x2: 6*f32 = 6*4 = 24
        2x4: 8*f32 = 8*4 = 32
        4x2: 8*f32 = 8*4 = 32
        3x3: 9*f32 = 9*4 = 36
        3x4: 12*f32 = 12*4 = 48
        4x3: 12*f32 = 12*4 = 48
        4x4: 16*f32 = 16*4 = 64
        */
        #[rustfmt::skip]
        const DEFAULT_SIZEN_TYPE: &[Option<UniformType>] = &[
            None,None,None, None, // sizes 0..=3 don't have any default value
            Some(UniformType::Scalar(ScalarType::F32)), // Size 4
            None,None,None, // sizes 5..=7 don't have any default value
            Some(UniformType::Vec(VecType::Vec2(ScalarType::F32))), // Size 8
            None,None,None, // sizes 9..=11 don't have any default value
            Some(UniformType::Vec(VecType::Vec3(ScalarType::F32))), // Size 12
            None,None,None, // sizes 13..=15 don't have any default value
            Some(UniformType::Vec(VecType::Vec4(ScalarType::F32))), // Size 16
            None,None,None,None,None,None,None, // sizes 17..=23 don't have any default value
            Some(UniformType::Matrix(MatrixType::M2x3)), // Size 24
            None,None,None,None,None,None,None, // sizes 25..=31 don't have any default value
            Some(UniformType::Matrix(MatrixType::M2x4)), // Size 32
            None,None,None, // sizes 33..=35 don't have any default value
            Some(UniformType::Matrix(MatrixType::M3x3)), // Size 36
            None,None,None,None,None,None,None,None,None,None,None, // sizes 37..=37 don't have any default value
            Some(UniformType::Matrix(MatrixType::M3x4)), // Size 48
            None,None,None,None,None,None,None,None,None,None,None,None,None,None,None, // sizes 49..=63 don't have any default value
            Some(UniformType::Matrix(MatrixType::M4x4)), // Size 64
            // Sizes 65..infinity don't have any default value
        ];
        // Make sure that I've coutned correctly
        // TODO: Make this into a test
        assert_eq!(
            DEFAULT_SIZEN_TYPE[4],
            Some(UniformType::Scalar(ScalarType::F32))
        );
        assert_eq!(
            DEFAULT_SIZEN_TYPE[8],
            Some(UniformType::Vec(VecType::Vec2(ScalarType::F32)))
        );
        assert_eq!(
            DEFAULT_SIZEN_TYPE[12],
            Some(UniformType::Vec(VecType::Vec3(ScalarType::F32)))
        );
        assert_eq!(
            DEFAULT_SIZEN_TYPE[16],
            Some(UniformType::Vec(VecType::Vec4(ScalarType::F32)))
        );
        assert_eq!(
            DEFAULT_SIZEN_TYPE[24],
            Some(UniformType::Matrix(MatrixType::M2x3))
        );
        assert_eq!(
            DEFAULT_SIZEN_TYPE[32],
            Some(UniformType::Matrix(MatrixType::M2x4))
        );
        assert_eq!(
            DEFAULT_SIZEN_TYPE[36],
            Some(UniformType::Matrix(MatrixType::M3x3))
        );
        assert_eq!(
            DEFAULT_SIZEN_TYPE[48],
            Some(UniformType::Matrix(MatrixType::M3x4))
        );
        assert_eq!(
            DEFAULT_SIZEN_TYPE[64],
            Some(UniformType::Matrix(MatrixType::M4x4))
        );
        assert_eq!(DEFAULT_SIZEN_TYPE.len(), 65);
        self.change_type(
            DEFAULT_SIZEN_TYPE.get(new_size as usize).unwrap().unwrap(),
            queue,
            device,
        )
    }

    fn change_matrix_size(&mut self, matrix_size: MatrixType, queue: &Queue) {
        self.value.change_matrix_size(matrix_size);
        let new_value = self.value.to_le_bytes();
        queue.write_buffer(&self.buffer, 0, &new_value).unwrap();
    }

    fn to_json(&self) -> serde_json::Value {
        let mut val = serde_json::Map::new();
        val.insert("name".into(), self.name.clone().into());
        val.insert("value".into(), self.value.to_json());
        serde_json::Value::Object(val)
    }
}

pub struct UniformGroup {
    bindings: Vec<UniformBinding>,
    pub bind_group: BindGroup,
}

impl UniformGroup {
    fn new(device: &Device) -> UniformGroup {
        let bg = device
            .create_bind_group(&BindGroupDescriptor {
                label: None,
                layout: &device
                    .create_bind_group_layout(&BindGroupLayoutDescriptor {
                        label: Some("Automaticall created layout in new"),
                        entries: &[],
                    })
                    .unwrap(),
                entries: &[],
            })
            .unwrap();

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

        device
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Automatically created layout in bg_layout"),
                entries: &entries,
            })
            .unwrap()
    }

    fn add_f32(&mut self, device: &Device) {
        self.bindings
            .push(UniformBinding::new(device, DEFAULT_UNIFORM));
        self.refresh_bind_group(device)
    }

    fn add_custom(&mut self, device: &Device, uniform: UniformValue) {
        self.bindings.push(UniformBinding::new(device, uniform));
        self.refresh_bind_group(device)
    }

    fn update_buffer(&mut self, b_index: usize, queue: &Queue) {
        let binding = &mut self.bindings[b_index];
        queue
            .write_buffer(&binding.buffer, 0, &binding.value.to_le_bytes())
            .unwrap();
    }

    fn refresh_bind_group(&mut self, device: &Device) {
        let mut layout_entries = Vec::new();
        let mut bindgroup_entries = Vec::new();
        for (index, binding) in self.bindings.iter().enumerate() {
            layout_entries.push(binding.bgl_entry(index as u32));
            bindgroup_entries.push(binding.bg_entry(index as u32));
        }

        let layout = device
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Autogenerated bind group layout in refresh_bind_group"),
                entries: &layout_entries,
            })
            .unwrap();
        let bg = device
            .create_bind_group(&BindGroupDescriptor {
                label: Some("Autogenerated bind group in refresh_bind_group"),
                layout: &layout,
                entries: &bindgroup_entries,
            })
            .unwrap();

        self.bind_group = bg;
    }

    fn define_binding(&mut self, binding: u32, device: &Device) {
        while binding >= self.bindings.len() as u32 {
            self.add_f32(device)
        }
    }

    fn change_type(
        &mut self,
        unitype: UniformType,
        b_index: usize,
        queue: &Queue,
        device: &Device,
    ) {
        self.bindings[b_index].change_type(unitype, queue, device);
        self.refresh_bind_group(device);
    }

    fn increase(&mut self, b_index: usize, queue: &Queue) {
        self.bindings[b_index].increase(queue)
    }

    fn decrease(&mut self, b_index: usize, queue: &Queue) {
        self.bindings[b_index].decrease(queue)
    }

    fn change_inner_type(
        &mut self,
        inner_type: ScalarType,
        b_index: usize,
        device: &Device,
        queue: &Queue,
    ) {
        self.bindings[b_index].change_inner_type(inner_type, queue);
        self.refresh_bind_group(device);
    }

    fn change_binding_size(
        &mut self,
        b_index: usize,
        new_size: u64,
        device: &Device,
        queue: &Queue,
    ) {
        self.bindings[b_index].change_binding_size(new_size, device, queue);
        self.refresh_bind_group(device);
    }

    fn change_matrix_size(
        &mut self,
        matrix_size: MatrixType,
        b_index: usize,
        device: &Device,
        queue: &Queue,
    ) {
        self.bindings[b_index].change_matrix_size(matrix_size, queue);
        self.refresh_bind_group(device);
    }

    fn to_json(&self) -> serde_json::Value {
        let mut bindings = Vec::new();
        for binding in self.bindings.iter() {
            bindings.push(binding.to_json())
        }

        serde_json::Value::Array(bindings)
    }

    fn set_name(&mut self, b_index: usize, name: String) {
        self.bindings[b_index].name = name
    }
}

pub(crate) struct CameraUniform {
    position: Point3<f32>,
    view_matrix: Matrix4<f32>,
    projection_matrix: Matrix4<f32>,
    inverse_view_matrix: Matrix4<f32>,
    inverse_projection_matrix: Matrix4<f32>,
}

type V4Iter =
    Chain<Chain<Chain<IntoIter<u8, 4>, IntoIter<u8, 4>>, IntoIter<u8, 4>>, IntoIter<u8, 4>>;
fn get_vec4_bytes(vec4: Vector4<f32>) -> V4Iter {
    vec4.x
        .to_le_bytes()
        .into_iter()
        .chain(vec4.y.to_le_bytes())
        .chain(vec4.z.to_le_bytes())
        .chain(vec4.w.to_le_bytes())
}

type M4Iter = Chain<Chain<Chain<V4Iter, V4Iter>, V4Iter>, V4Iter>;
fn get_matrix4_bytes(mat4: Matrix4<f32>) -> M4Iter {
    get_vec4_bytes(mat4.x)
        .chain(get_vec4_bytes(mat4.y))
        .chain(get_vec4_bytes(mat4.z))
        .chain(get_vec4_bytes(mat4.w))
}

impl CameraUniform {
    pub(crate) fn to_le_bytes(&self) -> Vec<u8> {
        let position = self
            .position
            .x
            .to_le_bytes()
            .into_iter()
            .chain(self.position.y.to_le_bytes())
            .chain(self.position.z.to_le_bytes())
            .chain([0u8, 0u8, 0u8, 0u8]); // 4 extra bytes because alignment of vec3 = alignment of vec4 = 16 (4 extra bytes)

        let projection = get_matrix4_bytes(self.projection_matrix);
        let view = get_matrix4_bytes(self.view_matrix);
        let inverse_view = get_matrix4_bytes(self.inverse_view_matrix);
        let inverse_proj = get_matrix4_bytes(self.inverse_projection_matrix);

        position
            .chain(projection)
            .chain(view)
            .chain(inverse_view)
            .chain(inverse_proj)
            .collect()
    }
}

pub struct Uniforms {
    pub groups: Vec<UniformGroup>,
    time_uniform_location: (usize, usize),
    camera_uniform_location: (usize, usize),
}

impl Uniforms {
    fn new(device: &Device) -> Uniforms {
        let mut group0 = UniformGroup::new(device);
        group0.add_custom(device, UniformValue::BuiltIn(BuiltinValue::Time));
        let time_uniform_location = (0, 0);
        let mut group1 = UniformGroup::new(device);
        let yaw: Rad<f32> = Deg(-45.0).into();
        let pitch: Rad<f32> = Deg(-45.0).into();
        group1.add_custom(
            device,
            UniformValue::BuiltIn(BuiltinValue::Camera {
                position: Point3 {
                    x: -1.5,
                    y: 1.2,
                    z: 0.5,
                },
                yaw: yaw.0,
                pitch: pitch.0,
                enabled: false,
            }),
        );
        let camera_uniform_location = (1, 0);
        Uniforms {
            groups: vec![group0, group1],
            time_uniform_location,
            camera_uniform_location,
        }
    }

    fn add_f32(&mut self, g_index: usize, device: &Device) {
        self.groups[g_index].add_f32(device)
    }

    fn update_buffer(&mut self, g_index: usize, b_index: usize, queue: &Queue) {
        self.groups[g_index].update_buffer(b_index, queue)
    }

    fn add_bind_group(&mut self, device: &Device) {
        self.groups.push(UniformGroup::new(device))
    }

    pub(crate) fn update_time(&self, elapsed_time: u32, queue: &Queue) {
        let (g_index, b_index) = self.time_uniform_location;
        let time_binding = &self.groups[g_index].bindings[b_index];
        assert!(time_binding.value == UniformValue::BuiltIn(BuiltinValue::Time));

        queue
            .write_buffer(&time_binding.buffer, 0, &elapsed_time.to_le_bytes())
            .unwrap();
    }

    pub(crate) fn enable_camera(&mut self, enable: bool, queue: &Queue) {
        let (g_index, b_index) = self.camera_uniform_location;
        let camera_binding = &mut self.groups[g_index].bindings[b_index];

        match &mut camera_binding.value {
            UniformValue::BuiltIn(BuiltinValue::Camera { enabled, .. }) => *enabled = enable,
            _ => unreachable!(),
        };

        self.update_buffer(g_index, b_index, queue)
    }

    pub(crate) fn define_binding(&mut self, group: u32, binding: u32, device: &Device) {
        while group >= self.groups.len() as u32 {
            self.add_bind_group(device)
        }

        self.groups[group as usize].define_binding(binding, device);
    }

    fn change_type(
        &mut self,
        unitype: UniformType,
        g_index: usize,
        b_index: usize,
        queue: &Queue,
        device: &Device,
    ) {
        self.groups[g_index].change_type(unitype, b_index, queue, device)
    }

    fn increase(&mut self, g_index: usize, b_index: usize, queue: &Queue) {
        self.groups[g_index].increase(b_index, queue)
    }

    fn decrease(&mut self, g_index: usize, b_index: usize, queue: &Queue) {
        self.groups[g_index].decrease(b_index, queue)
    }

    fn change_inner_type(
        &mut self,
        inner_type: ScalarType,
        g_index: usize,
        b_index: usize,
        device: &Device,
        queue: &Queue,
    ) {
        self.groups[g_index].change_inner_type(inner_type, b_index, device, queue)
    }

    pub(crate) fn change_binding_size(
        &mut self,
        g_index: usize,
        b_index: usize,
        new_size: u64,
        device: &Device,
        queue: &Queue,
    ) {
        self.groups[g_index].change_binding_size(b_index, new_size, device, queue);
    }

    fn change_matrix_size(
        &mut self,
        matrix_size: MatrixType,
        g_index: usize,
        b_index: usize,
        device: &Device,
        queue: &Queue,
    ) {
        self.groups[g_index].change_matrix_size(matrix_size, b_index, device, queue)
    }

    pub(crate) fn save(&self, shader_name: &str) {
        let config = std::fs::read_to_string("save.json").unwrap_or(String::from("{}"));
        let config = serde_json::from_str(&config).unwrap_or(JsonValue::Object(Map::new()));

        let mut config = if let JsonValue::Object(config) = config {
            config
        } else {
            serde_json::Map::new()
        };

        config.get(shader_name);

        let tul = self.time_uniform_location;
        let time_uniform_location = JsonValue::Array(vec![JsonValue::Number(serde_json::Number::from(tul.0)), JsonValue::Number(serde_json::Number::from(tul.1))]);
        let cul = self.camera_uniform_location;
        let camera_uniform_location = JsonValue::Array(vec![JsonValue::Number(serde_json::Number::from(cul.0)), JsonValue::Number(serde_json::Number::from(cul.1))]);

        let mut shader_conf = Map::new();
        shader_conf.insert("time_uniform_location".into(), time_uniform_location);
        shader_conf.insert("camera_uniform_location".into(), camera_uniform_location);

        let mut json_groups = Vec::new();

        for group in self.groups.iter() {
            json_groups.push(group.to_json());
        }

        let json_groups = JsonValue::Array(json_groups);
        shader_conf.insert("groups".into(), json_groups);

        config.insert(shader_name.into(), JsonValue::Object(shader_conf));
        let file = std::fs::OpenOptions::new().create(true).write(true).open("save.json").unwrap();
        serde_json::to_writer(file, &config).unwrap();
    }

    pub(crate) fn load(device: &Device, shader_name: &str) -> Option<Uniforms> {
        let config = std::fs::read_to_string("save.json").ok()?;
        let config: JsonValue = serde_json::from_str(&config).ok()?;

        let config = config.as_object()?
            .get(shader_name)?
            .as_object()?;

        let time_uniform_location = config.get("time_uniform_location")?.as_array()?;
        let camera_uniform_location = config.get("camera_uniform_location")?.as_array()?;

        let tul_0 = time_uniform_location.get(0)?;
        let tul_1 = time_uniform_location.get(1)?;
        let cul_0 = camera_uniform_location.get(0)?;
        let cul_1 = camera_uniform_location.get(1)?;

        let (tul, cul) = if let (Some(tul_0),Some(tul_1),Some(cul_0),Some(cul_1)) = (tul_0.as_u64(), tul_1.as_u64(), cul_0.as_u64(), cul_1.as_u64()) {
            ((tul_0 as usize, tul_1 as usize), (cul_0 as usize, cul_1 as usize))
        } else {
            if let None = cul_0.as_u64() {
                println!("cul_0 is not a number")
            }
            if let None = cul_1.as_u64() {
                println!("cul_1 is not a number")
            }
            if let None = tul_0.as_u64() {
                println!("tul_0 is not a number")
            }
            if let None = tul_1.as_u64() {
                println!("tul_1 is not a number")
            }
            println!("Couldn't load saved data because uniform locations items aren't numbers");
            return None
        };

        let json_groups = config.get("groups")?.as_array()?;

        let mut groups = Vec::new();
        let mut time_count = 0;
        let mut camera_count = 0;
        for group in json_groups {
            let mut uniform_group = UniformGroup::new(device);
            let group = group.as_array()?;
            for (i, uniform) in group.iter().enumerate() {
                let name = uniform.get("name")?.as_str()?.into();
                let uniform = uniform.get("value")?.as_object()?;
                let uniform = UniformValue::from_json(uniform)?;
                uniform_group.add_custom(device, uniform);
                uniform_group.set_name(i, name);
                match uniform {
                    UniformValue::BuiltIn(BuiltinValue::Time) => time_count += 1,
                    UniformValue::BuiltIn(BuiltinValue::Camera { .. }) => camera_count += 1,
                    _ => ()
                }
            }
            groups.push(uniform_group)
        }

        if time_count != 1 || camera_count != 1 {
            println!("Couldn't load saved data because there is not exactl 1 time and 1 camera");
            return None
        }

        // TODO: Check that time and camera are in correct positions

        Some(Uniforms {
            groups,
            time_uniform_location: tul,
            camera_uniform_location: cul
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MeshType {
    Screen2D,
    Plane,
    Sphere,
    Cube,
    Cylinder,
    Cone,
    Torus,
}

pub enum MeshConfig {
    Screen2D,
    Plane((f32, f32), (u32, u32)),
    Sphere,
    Cube,
    Cylinder,
    Cone,
    Torus,
}

pub struct UiState {
    pub texture_id: TextureId,
    shader_name: String,
    shader_exists: bool,
    pub inputs: Uniforms,
    errors: Vec<String>,
    show_errors: bool,
    mesh_type: MeshType,
    pub mesh_config: MeshConfig,
    pub show_mesh: bool,
    always_on_top: bool,
    pub background_color: [f32; 4],
    pub draw_grid: bool,
}

impl UiState {
    fn new(texture_id: TextureId, device: &Device) -> UiState {
        UiState {
            texture_id,
            shader_name: "shader.wgsl".to_string(),
            shader_exists: true,
            inputs: Uniforms::new(device),
            errors: vec![],
            show_errors: false,
            mesh_type: MeshType::Screen2D,
            mesh_config: MeshConfig::Screen2D,
            show_mesh: false,
            always_on_top: false,
            background_color: [1.0, 0.5, 0.5, 1.0],
            draw_grid: true,
        }
    }

    fn create_ui(&mut self, ui: &Ui, device: &Device, queue: &Queue) -> Option<Message> {
        let mut message = None;
        ui.dockspace_over_main_viewport();
        ui.window("Render").build(|| {
            let a = ui.push_style_var(StyleVar::FrameBorderSize(50.0));
            Image::new(self.texture_id, mint::Vector2{ x: IMAGE_WIDTH, y: IMAGE_HEIGHT }).border_col([1.0;4]).build(ui);
            a.pop();
            if self.show_mesh {
                ui.text_colored([1.0, 0.0, 0.0, 1.0], "Mesh rendering is enabled, turn it off\nin the \"Mesh configuration\" window to see\nthe expected output")
            }
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
            ui.separator();
            if ui.checkbox("Show always on top", &mut self.always_on_top) {
                if self.always_on_top {
                    message = Some(Message::ChangeWindowLevel(WindowLevel::AlwaysOnTop))
                } else {
                    message = Some(Message::ChangeWindowLevel(WindowLevel::Normal))
                }
            }
        });

        ui.window("Shader parameters").build(|| {
            ui.color_edit4("Background color", &mut self.background_color);
            let mut edit_event = None;
            for (group_index, group) in self.inputs.groups.iter_mut().enumerate() {
                if ui.collapsing_header(
                    format!("Binding group {group_index}"),
                    TreeNodeFlags::empty(),
                ) {
                    for (binding_index, uniform) in group.bindings.iter_mut().enumerate() {
                        if let Some(event) = uniform.show_editor(ui, group_index, binding_index) {
                            edit_event = Some(event);
                        }
                        ui.separator();
                    }
                    if ui.button(format!("Add parameter to this group##add_f32{group_index}")) {
                        edit_event = Some(UniformEditEvent::AddUniform(group_index))
                    };
                }
            }

            ui.separator();
            if ui.button("Add Bind Group") {
                edit_event = Some(UniformEditEvent::AddBindGroup)
            }

            if ui.button("Save parameters") {
                message = Some(Message::SaveParameters)
            }

            if let Some(event) = edit_event {
                match event {
                    UniformEditEvent::UpdateBuffer(g_index, b_index) => {
                        self.inputs.update_buffer(g_index, b_index, queue)
                    }
                    UniformEditEvent::AddUniform(g_index) => self.inputs.add_f32(g_index, device),
                    UniformEditEvent::AddBindGroup => self.inputs.add_bind_group(device),
                    UniformEditEvent::ChangeType(unitype, g_index, b_index) => self
                        .inputs
                        .change_type(unitype, g_index, b_index, queue, device),
                    UniformEditEvent::Increase(g_index, b_index) => {
                        self.inputs.increase(g_index, b_index, queue)
                    }
                    UniformEditEvent::Decrease(g_index, b_index) => {
                        self.inputs.decrease(g_index, b_index, queue)
                    }
                    UniformEditEvent::ChangeInnerType(inner_type, g_index, b_index) => self
                        .inputs
                        .change_inner_type(inner_type, g_index, b_index, device, queue),
                    UniformEditEvent::ChangeMatrixSize(matrix_size, g_index, b_index) => self
                        .inputs
                        .change_matrix_size(matrix_size, g_index, b_index, device, queue),
                };
                message = Some(Message::ReloadPipeline);
            }
        });

        ui.window("Mesh configuration").build(|| {
            if ui.checkbox("Show mesh", &mut self.show_mesh) {
                message = Some(Message::ReloadPipeline)
            };
            ui.checkbox("Show grid", &mut self.draw_grid);
            ui.separator();

            if ui.radio_button("2D whole screen", &mut self.mesh_type, MeshType::Screen2D) {
                self.mesh_config = MeshConfig::Screen2D;
                message = Some(Message::ReloadMeshBuffers);
            };
            if ui.radio_button("Plane", &mut self.mesh_type, MeshType::Plane) {
                self.mesh_config = MeshConfig::Plane((1.0, 1.0), (1, 1));
                message = Some(Message::ReloadMeshBuffers);
            };
            let dis = ui.begin_disabled(true);
            if ui.radio_button("Cube", &mut self.mesh_type, MeshType::Cube) {
                self.mesh_config = MeshConfig::Cube;
                message = Some(Message::ReloadMeshBuffers);
            };
            if ui.radio_button("Sphere", &mut self.mesh_type, MeshType::Sphere) {
                self.mesh_config = MeshConfig::Sphere;
                message = Some(Message::ReloadMeshBuffers);
            };
            if ui.radio_button("Cone", &mut self.mesh_type, MeshType::Cone) {
                self.mesh_config = MeshConfig::Cone;
                message = Some(Message::ReloadMeshBuffers);
            };
            if ui.radio_button("Cylinder", &mut self.mesh_type, MeshType::Cylinder) {
                self.mesh_config = MeshConfig::Cylinder;
                message = Some(Message::ReloadMeshBuffers);
            };
            if ui.radio_button("Torus", &mut self.mesh_type, MeshType::Torus) {
                self.mesh_config = MeshConfig::Torus;
                message = Some(Message::ReloadMeshBuffers);
            };
            dis.end();
            ui.separator();

            match &mut self.mesh_config {
                MeshConfig::Screen2D => (),
                MeshConfig::Plane((x_size, y_size), (rows, columns)) => {
                    let mut size = [*x_size, *y_size];
                    if ui.input_float2("Size", &mut size).build() {
                        *x_size = size[0];
                        *y_size = size[1];
                        message = Some(Message::ReloadMeshBuffers)
                    };
                    ui.text("Triangle resolution:");
                    if ui.slider("Rows", 1, 1_000, rows) {
                        message = Some(Message::ReloadMeshBuffers)
                    };
                    if ui.slider("Columns", 1, 1_000, columns) {
                        message = Some(Message::ReloadMeshBuffers)
                    };
                }
                MeshConfig::Sphere => {
                    ui.input_float("Radius", &mut 0.0).build();
                    ui.slider("Triangle count", 4, 1_000_000, &mut 0);
                }
                MeshConfig::Cube => {
                    ui.input_float("Side length", &mut 0.0).build();
                    ui.slider("Triangles per side", 2, 1_000_000, &mut 0);
                }
                MeshConfig::Cylinder => {
                    ui.slider("Radius", 0.1, 1000.0, &mut 1.0);
                    ui.slider("height", 0.1, 1000.0, &mut 3.0);
                }
                MeshConfig::Cone => {
                    ui.slider("Radius", 0.1, 1000.0, &mut 1.0);
                    ui.slider("height", 0.1, 1000.0, &mut 3.0);
                }
                MeshConfig::Torus => {
                    ui.slider("Inner radius", 0.1, 1000.0, &mut 1.0);
                    ui.slider("Outer radius", 0.1, 1000.0, &mut 0.0);
                }
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

    pub(crate) fn load_uniforms(&mut self, shader_name: &str, device: &Device) {
        self.inputs = match Uniforms::load(device, shader_name) {
            Some(inputs) => inputs,
            None => Uniforms::new(device)
        }
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

        let texture = ImTexture::new(
            &gpu.device,
            &renderer,
            TextureConfig {
                size: wgpu::Extent3d {
                    width: IMAGE_WIDTH as u32,
                    height: IMAGE_HEIGHT as u32,
                    ..Default::default()
                },
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                ..Default::default()
            },
        );
        let texture_id = renderer.textures.insert(texture);

        let ui = UiState::new(texture_id, &gpu.device);
        ImState {
            context,
            platform,
            renderer,
            ui,
        }
    }

    pub fn render(
        &mut self,
        window: &WinitWindow,
        gpu: &Gpu,
        view: &TextureView,
    ) -> (CommandEncoder, Option<Message>) {
        self.platform
            .prepare_frame(self.context.io_mut(), window)
            .expect("Failed to prepare frame");
        let ui = self.context.frame();

        let message = self.ui.create_ui(ui, &gpu.device, &gpu.queue);

        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("ImGui Render Encoder"),
            })
            .unwrap();
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
                timestamp_writes: None,
            });
            self.renderer
                .render(
                    self.context.render(),
                    &gpu.queue,
                    &gpu.device,
                    &mut render_pass,
                )
                .expect("Rendering failed");
        }
        (encoder, message)
    }

    pub fn handle_event(&mut self, event: &Event<()>, window: &WinitWindow) {
        self.platform
            .handle_event(self.context.io_mut(), window, event);
    }

    pub fn get_texture_view(&self) -> &TextureView {
        self.renderer
            .textures
            .get(self.ui.texture_id)
            .unwrap()
            .view()
    }

    pub(crate) fn destroy_errors(&mut self) {
        self.ui.errors = Vec::new();
        self.ui.show_errors = false;
    }

    pub(crate) fn show_crate_shader_err(&mut self, err: CreateShaderModuleError) {
        self.ui.show_errors = true;
        self.ui.errors = vec![err.to_string()]
    }
}
