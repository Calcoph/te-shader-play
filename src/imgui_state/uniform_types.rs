use std::borrow::Cow;

use cgmath::{Deg, InnerSpace, Matrix4, Point3, Rad, SquareMatrix, Transform, Vector3};
use imgui::Ui;
use serde_json::{Map, Value as JsonValue};

use crate::imgui_state::UniformEditEvent;

pub(crate) use self::{
    matrix::MatrixType,
    scalar::{ScalarType, ScalarUniformValue},
    vec::VecType,
};
use self::{matrix::MatrixUniformValue, transform::TransformUniformValue, vec::VectorUniformValue};

use super::{
    CameraUniform, ImguiMatrix, ImguiScalar, ImguiUniformSelectable, ImguiVec, DEFAULT_U32_UNIFORM,
};

mod matrix;
mod scalar;
mod transform;
mod vec;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum BuiltinValue {
    Time,
    Camera {
        position: Point3<f32>,
        yaw: f32,
        pitch: f32,
        enabled: bool,
    },
}
impl BuiltinValue {
    fn to_le_bytes(self) -> Vec<u8> {
        match self {
            BuiltinValue::Time => 0u32.to_le_bytes().into(),
            BuiltinValue::Camera { .. } => self.calc_matrix().to_le_bytes(),
        }
    }

    fn calc_matrix(&self) -> CameraUniform {
        match self {
            BuiltinValue::Camera {
                position,
                yaw,
                pitch,
                enabled,
            } => {
                #[rustfmt::skip]
                let (view_matrix, projection_matrix, inverse_view, inverse_proj) = if *enabled {
                    let view = Matrix4::look_to_rh(
                        *position,
                        Vector3::new(
                            yaw.cos() * pitch.cos(),
                            pitch.sin(),
                            yaw.sin() * pitch.cos(),
                        )
                        .normalize(),
                        Vector3::unit_y(),
                    );

                    let projection = cgmath::perspective(Rad::from(Deg(45.0)), 1.0, 0.1, 100.0);

                    (
                        view,
                        projection,
                        view.inverse_transform().unwrap_or_else(|| {
                            println!("Uninversable matrix!");
                            Matrix4::identity()
                        }),
                        projection.inverse_transform().unwrap_or_else(|| {
                            println!("Uninversable matrix!");
                            Matrix4::identity()
                        }),
                    )
                } else {
                    (
                        Matrix4::new(
                        1.0, 0.0, 0.0, 0.0,
                        0.0, 1.0, 0.0, 0.0,
                        0.0, 0.0, 1.0, 0.0,
                        0.0, 0.0, 0.0, 1.0
                        ),
                        Matrix4::new(
                            1.0, 0.0, 0.0, 0.0,
                            0.0, 1.0, 0.0, 0.0,
                            0.0, 0.0, 1.0, 0.0,
                            0.0, 0.0, 0.0, 1.0
                        ),
                        Matrix4::new(
                            1.0, 0.0, 0.0, 0.0,
                            0.0, 1.0, 0.0, 0.0,
                            0.0, 0.0, 1.0, 0.0,
                            0.0, 0.0, 0.0, 1.0
                        ),
                        Matrix4::new(
                            1.0, 0.0, 0.0, 0.0,
                            0.0, 1.0, 0.0, 0.0,
                            0.0, 0.0, 1.0, 0.0,
                            0.0, 0.0, 0.0, 1.0
                    ))
                };

                CameraUniform {
                    position: *position,
                    view_matrix,
                    projection_matrix,
                    inverse_view_matrix: inverse_view,
                    inverse_projection_matrix: inverse_proj,
                }
            }
            _ => unreachable!(),
        }
    }

    fn from_json(uniform: &Map<String, JsonValue>) -> Option<BuiltinValue> {
        let inner_type = uniform.get("innertype")?;
        if let None = inner_type.as_str() {
            println!("Couldn't load saved data because innertype of builtin isn't a string");
        }
        match inner_type.as_str()? {
            "time" => Some(BuiltinValue::Time),
            "camera" => {
                let json_position = uniform.get("position")?.as_array()?;
                let yaw = uniform.get("yaw")?;
                let pitch = uniform.get("pitch")?;
                let enabled = uniform.get("enabled")?;

                if json_position.len() != 3 {
                    return None
                }

                let position = Point3 {
                    x: json_position.get(0)?.as_f64()? as f32,
                    y: json_position.get(1)?.as_f64()? as f32,
                    z: json_position.get(2)?.as_f64()? as f32,
                };

                let yaw = yaw.as_f64()? as f32;
                let pitch = pitch.as_f64()? as f32;
                let enabled = enabled.as_bool()?;

                Some(BuiltinValue::Camera { position, yaw, pitch, enabled })
            },
            _ => {
                println!("Couldn't load saved data because of invalid innertype of builtin");
                None
            }
        }
    }

    fn to_json(&self, json_obj: &mut serde_json::Map<String, serde_json::Value>) {
        match self {
            BuiltinValue::Time => json_obj.insert("innertype".into(), "time".into()),
            BuiltinValue::Camera { .. } => json_obj.insert("innertype".into(), "camera".into()),
        };

        match self {
            BuiltinValue::Time => (),
            BuiltinValue::Camera { position, yaw, pitch, enabled } => {
                let position: Vec<serde_json::Value> = vec![position.x.into(), position.y.into(), position.z.into()];
                json_obj.insert("position".into(), position.into());
                json_obj.insert("yaw".into(), serde_json::Value::from(*yaw));
                json_obj.insert("pitch".into(), serde_json::Value::from(*pitch));
                json_obj.insert("enabled".into(), serde_json::Value::from(*enabled));
            },
        }
    }
}

fn cast_f32_u32(v: f32) -> u32 {
    let v = (v as i32).try_into();
    v.unwrap_or(DEFAULT_U32_UNIFORM)
}

fn cast_i32_u32(v: i32) -> u32 {
    let res = v.try_into();
    res.unwrap_or(DEFAULT_U32_UNIFORM)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum UniformValue {
    BuiltIn(BuiltinValue),
    Scalar(ScalarUniformValue),
    Vector(VectorUniformValue),
    Matrix(MatrixUniformValue),
    Transform(TransformUniformValue),
}

trait ExtendedUi {
    fn matrix_combo<V, L>(
        &self,
        label: impl AsRef<str>,
        current_item: &mut usize,
        items: &[V],
        label_fn: L,
        column_amount: i32,
    ) -> bool
    where
        for<'b> L: Fn(&'b V) -> Cow<'b, str>;
}

impl ExtendedUi for Ui {
    fn matrix_combo<V, L>(
        &self,
        label: impl AsRef<str>,
        current_item: &mut usize,
        items: &[V],
        label_fn: L,
        column_amount: i32,
    ) -> bool
    where
        for<'b> L: Fn(&'b V) -> Cow<'b, str>,
    {
        let mut ret = false;
        let mut selected = label_fn(&items[*current_item]);
        if let Some(_cb) = self.begin_combo(label, selected.clone()) {
            for (i, cur) in items.iter().enumerate() {
                let cur = label_fn(cur);
                if selected == cur {
                    // Auto-scroll to selected item
                    self.set_item_default_focus();
                }
                self.columns(column_amount, "columns", true);
                // Create a "selectable"
                let clicked = self
                    .selectable_config(cur.clone())
                    .selected(selected == cur)
                    .build();
                // When item is clicked, store it
                if clicked {
                    ret = true;
                    *current_item = i;
                    selected = cur;
                }
                self.next_column()
            }
        };

        ret
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum UniformType {
    Scalar(ScalarType),
    Vec(VecType),
    Matrix(MatrixType),
    Transform,
}

impl ImguiUniformSelectable for UniformValue {
    fn cast_to(&self, casted_type: UniformType) -> UniformValue {
        match self {
            UniformValue::Scalar(s) => s.cast_to(casted_type),
            UniformValue::Vector(v) => v.cast_to(casted_type),
            UniformValue::Matrix(m) => m.cast_to(casted_type),
            UniformValue::BuiltIn(_) => unreachable!(),
            UniformValue::Transform(t) => t.cast_to(casted_type),
        }
    }

    fn show_editor(
        &mut self,
        ui: &Ui,
        group_index: usize,
        binding_index: usize,
        val_name: &mut String,
    ) -> Option<UniformEditEvent> {
        match self {
            UniformValue::BuiltIn(builtin) => match builtin {
                BuiltinValue::Time => {
                    ui.text(format!("({binding_index}) Time (u32)"));
                    None
                }
                BuiltinValue::Camera {
                    position,
                    yaw,
                    pitch,
                    enabled,
                } => {
                    let mut message = None;
                    ui.text(format!("({binding_index}) Camera (struct {{\n    vec4<f32>,\n    mat4x4<f32>,\n    mat4x4<f32>,\n    mat4x4<f32>,\n    mat4x4<f32>\n}})"));
                    if ui.checkbox("Enabled", enabled) {
                        message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                    }
                    if *enabled {
                        ui.text("Position (x,y,z):");
                        ui.indent();
                        let mut pos = [position.x, position.y, position.z];
                        if ui
                            .input_float3(
                                format!("##camera_{group_index}_{binding_index}"),
                                &mut pos,
                            )
                            .build()
                        {
                            *position = Point3 {
                                x: pos[0],
                                y: pos[1],
                                z: pos[2],
                            };
                            message =
                                Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                        };
                        ui.unindent();
                        ui.text("Yaw");
                        let mut dyaw: Deg<f32> = Rad(*yaw).into();
                        if ui.slider(
                            format!("##yaw_{group_index}_{binding_index}"),
                            -89.9,
                            89.9,
                            &mut dyaw.0,
                        ) {
                            let ryaw: Rad<f32> = dyaw.into();

                            *yaw = ryaw.0;
                            message =
                                Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                        }
                        ui.text("Pitch");
                        let mut dpitch: Deg<f32> = Rad(*pitch).into();
                        if ui.slider(
                            format!("##pitch_{group_index}_{binding_index}"),
                            -89.9,
                            89.9,
                            &mut dpitch.0,
                        ) {
                            let rpitch: Rad<f32> = dpitch.into();

                            *pitch = rpitch.0;
                            message =
                                Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                        }
                    }
                    message
                }
            },
            UniformValue::Scalar(s) => s.show_editor(ui, group_index, binding_index, val_name),
            UniformValue::Vector(v) => v.show_editor(ui, group_index, binding_index, val_name),
            UniformValue::Matrix(m) => m.show_editor(ui, group_index, binding_index, val_name),
            UniformValue::Transform(t) => t.show_editor(ui, group_index, binding_index, val_name),
        }
    }

    fn to_le_bytes(&self) -> Vec<u8> {
        match self {
            UniformValue::BuiltIn(b) => b.to_le_bytes(),
            UniformValue::Scalar(s) => s.to_le_bytes(),
            UniformValue::Vector(v) => v.to_le_bytes(),
            UniformValue::Matrix(m) => m.to_le_bytes(),
            UniformValue::Transform(t) => t.to_le_bytes(),
        }
    }
}

impl ImguiScalar for UniformValue {
    fn decrease(&mut self) {
        match self {
            UniformValue::Scalar(s) => s.decrease(),
            UniformValue::Matrix(_) => unreachable!(),
            UniformValue::BuiltIn(_) => unreachable!(),
            UniformValue::Vector(_) => unreachable!(),
            UniformValue::Transform(_) => unreachable!(),
        }
    }

    fn increase(&mut self) {
        match self {
            UniformValue::Scalar(s) => s.increase(),
            UniformValue::Matrix(_) => unreachable!(),
            UniformValue::BuiltIn(_) => unreachable!(),
            UniformValue::Vector(_) => unreachable!(),
            UniformValue::Transform(_) => unreachable!(),
        }
    }
}

impl ImguiVec for UniformValue {
    fn change_inner_type(&mut self, inner_type: ScalarType) {
        match self {
            UniformValue::Vector(v) => v.change_inner_type(inner_type),
            UniformValue::Matrix(_) => unreachable!(),
            UniformValue::BuiltIn(_) => unreachable!(),
            UniformValue::Scalar(_) => unreachable!(),
            UniformValue::Transform(_) => unreachable!(),
        }
    }
}

impl ImguiMatrix for UniformValue {
    fn change_matrix_size(&mut self, matrix_size: MatrixType) {
        match self {
            UniformValue::BuiltIn(_) => unreachable!(),
            UniformValue::Scalar(_) => unreachable!(),
            UniformValue::Vector(_) => unreachable!(),
            UniformValue::Matrix(m) => m.change_matrix_size(matrix_size),
            UniformValue::Transform(_) => unreachable!(),
        }
    }
}

impl UniformValue {
    fn show_primitive_selector(
        ui: &Ui,
        group_index: usize,
        binding_index: usize,
        message: &mut Option<UniformEditEvent>,
        type_index: usize,
        val_name: &mut String,
    ) {
        const TYPES: &[UniformType] = &[
            UniformType::Scalar(ScalarType::U32),
            UniformType::Scalar(ScalarType::I32),
            UniformType::Scalar(ScalarType::F32),
            UniformType::Vec(VecType::Vec2(ScalarType::F32)),
            UniformType::Vec(VecType::Vec3(ScalarType::F32)),
            UniformType::Vec(VecType::Vec4(ScalarType::F32)),
            UniformType::Matrix(MatrixType::M4x4),
            UniformType::Transform,
        ];
        const COMBO_WIDTH: f32 = 95.0;
        const VAR_NAME_WIDTH: f32 = 150.0;

        ui.text(format!("({binding_index})"));
        ui.same_line();
        ui.set_next_item_width(VAR_NAME_WIDTH);
        ui.input_text(
            format!("##name_edit{group_index}_{binding_index}"),
            val_name,
        )
        .build();
        ui.set_next_item_width(COMBO_WIDTH);
        let mut selection = type_index;
        if ui.combo(
            format!("##combo_g{group_index}_b{binding_index}"),
            &mut selection,
            TYPES,
            |unitype| unitype.into(),
        ) {
            let selected_type = TYPES[selection];
            if selected_type != TYPES[type_index] {
                *message = Some(UniformEditEvent::ChangeType(
                    selected_type,
                    group_index,
                    binding_index,
                ))
            }
        };
    }

    pub(crate) fn from_json(uniform: &Map<String, JsonValue>) -> Option<UniformValue> {
        let outer_type = uniform.get("outer_type")?;
        match outer_type.as_str()? {
            "builtin" => Some(UniformValue::BuiltIn(BuiltinValue::from_json(uniform)?)),
            "matrix" => Some(UniformValue::Matrix(MatrixUniformValue::from_json(uniform)?)),
            "scalar" => Some(UniformValue::Scalar(ScalarUniformValue::from_json(uniform)?)),
            "transform" => Some(UniformValue::Transform(TransformUniformValue::from_json(uniform)?)),
            "vector" => Some(UniformValue::Vector(VectorUniformValue::from_json(uniform)?)),
            _ => {
                println!("Couldn't load saved data because outer_type is not valid");
                None
            }
        }
    }

    pub(crate) fn to_json(&self) -> JsonValue {
        let mut json_o= Map::new();
        let json_obj = &mut json_o;
        match self {
            UniformValue::BuiltIn(_) => json_obj.insert("outer_type".into(), "builtin".into()),
            UniformValue::Scalar(_) => json_obj.insert("outer_type".into(), "scalar".into()),
            UniformValue::Vector(_) => json_obj.insert("outer_type".into(), "vector".into()),
            UniformValue::Matrix(_) => json_obj.insert("outer_type".into(), "matrix".into()),
            UniformValue::Transform(_) => json_obj.insert("outer_type".into(), "transform".into()),
        };

        match self {
            UniformValue::BuiltIn(b) => b.to_json(json_obj),
            UniformValue::Scalar(s) => s.to_json(json_obj),
            UniformValue::Vector(v) => v.to_json(json_obj),
            UniformValue::Matrix(m) => m.to_json(json_obj),
            UniformValue::Transform(t) => t.to_json(json_obj),
        };

        JsonValue::Object(json_o)
    }
}

impl<'a> From<&'a UniformType> for Cow<'static, str> {
    fn from(val: &'a UniformType) -> Cow<'static, str> {
        match val {
            UniformType::Scalar(s) => s.into(),
            UniformType::Vec(v) => v.into(),
            UniformType::Matrix(_) => Cow::Borrowed("matrix"),
            UniformType::Transform => Cow::Borrowed("transform"),
        }
    }
}
