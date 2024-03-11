use std::borrow::Cow;

use imgui::Ui;
use serde_json::{Map, Value as JsonValue};

use crate::imgui_state::{ImguiScalar, ImguiUniformSelectable, UniformEditEvent};

use super::{
    cast_f32_u32, cast_i32_u32,
    matrix::{Column2, Column3, Column4, MatrixUniformValue},
    transform::TransformUniformValue,
    vec::{Vec2UniformValue, Vec3UniformValue, Vec4UniformValue},
    MatrixType, UniformType, UniformValue, VecType, VectorUniformValue,
};

pub(crate) union ScalarPrimitive {
    pub(crate) u32: u32,
    pub(crate) i32: i32,
    pub(crate) f32: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum ScalarUniformValue {
    U32(u32),
    I32(i32),
    F32(f32),
}

impl ImguiScalar for ScalarUniformValue {
    fn decrease(&mut self) {
        match self {
            ScalarUniformValue::U32(v) => *v -= 1,
            ScalarUniformValue::I32(v) => *v -= 1,
            ScalarUniformValue::F32(v) => *v -= 1.0,
        }
    }

    fn increase(&mut self) {
        match self {
            ScalarUniformValue::U32(v) => *v += 1,
            ScalarUniformValue::I32(v) => *v += 1,
            ScalarUniformValue::F32(v) => *v += 1.0,
        }
    }
}

impl ImguiUniformSelectable for ScalarUniformValue {
    fn cast_to(&self, casted_type: UniformType) -> UniformValue {
        match casted_type {
            UniformType::Scalar(s) => UniformValue::Scalar(self.cast_to_scalar(s)),
            UniformType::Vec(v) => UniformValue::Vector(self.cast_to_vec(v)),
            UniformType::Matrix(m) => UniformValue::Matrix(self.cast_to_matrix(m)),
            UniformType::Transform => UniformValue::Transform(self.cast_to_transform()),
        }
    }

    fn show_editor(
        &mut self,
        ui: &Ui,
        group_index: usize,
        binding_index: usize,
        val_name: &mut String,
    ) -> Option<UniformEditEvent> {
        const PRIMITIVE_INPUT_WIDTH: f32 = 50.0;
        let mut message = None;
        match self {
            ScalarUniformValue::U32(v) => {
                UniformValue::show_primitive_selector(
                    ui,
                    group_index,
                    binding_index,
                    &mut message,
                    0,
                    val_name,
                );
                ui.same_line();
                ui.set_next_item_width(PRIMITIVE_INPUT_WIDTH);
                if ui
                    .input_scalar(format!("##editor{group_index}_{binding_index}"), v)
                    .build()
                {
                    message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index));
                }
                ui.same_line();
                Self::number_edit(ui, group_index, binding_index, &mut message)
            }
            ScalarUniformValue::I32(v) => {
                UniformValue::show_primitive_selector(
                    ui,
                    group_index,
                    binding_index,
                    &mut message,
                    1,
                    val_name,
                );
                ui.same_line();
                ui.set_next_item_width(PRIMITIVE_INPUT_WIDTH);
                if ui
                    .input_scalar(format!("##editor{group_index}_{binding_index}"), v)
                    .build()
                {
                    message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
                ui.same_line();
                Self::number_edit(ui, group_index, binding_index, &mut message)
            }
            ScalarUniformValue::F32(v) => {
                UniformValue::show_primitive_selector(
                    ui,
                    group_index,
                    binding_index,
                    &mut message,
                    2,
                    val_name,
                );
                ui.same_line();
                ui.set_next_item_width(PRIMITIVE_INPUT_WIDTH);
                if ui
                    .input_float(format!("##editor{group_index}_{binding_index}"), v)
                    .build()
                {
                    message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
                ui.same_line();
                Self::number_edit(ui, group_index, binding_index, &mut message)
            }
        };
        message
    }

    fn to_le_bytes(&self) -> Vec<u8> {
        match self {
            ScalarUniformValue::U32(s) => s.to_le_bytes().into(),
            ScalarUniformValue::I32(s) => s.to_le_bytes().into(),
            ScalarUniformValue::F32(s) => s.to_le_bytes().into(),
        }
    }
}

impl ScalarUniformValue {
    fn cast_to_scalar(self, s: ScalarType) -> ScalarUniformValue {
        match (self, s) {
            (ScalarUniformValue::U32(v), ScalarType::I32) => ScalarUniformValue::I32(v as i32),
            (ScalarUniformValue::U32(v), ScalarType::F32) => ScalarUniformValue::F32(v as f32),

            (ScalarUniformValue::I32(v), ScalarType::U32) => {
                ScalarUniformValue::U32(cast_i32_u32(v))
            }
            (ScalarUniformValue::I32(v), ScalarType::F32) => ScalarUniformValue::F32(v as f32),

            (ScalarUniformValue::F32(v), ScalarType::U32) => {
                ScalarUniformValue::U32(cast_f32_u32(v))
            }
            (ScalarUniformValue::F32(v), ScalarType::I32) => ScalarUniformValue::I32(v as i32),

            (ScalarUniformValue::U32(_), ScalarType::U32) => self,
            (ScalarUniformValue::I32(_), ScalarType::I32) => self,
            (ScalarUniformValue::F32(_), ScalarType::F32) => self,
        }
    }

    fn cast_to_vec(&self, v: VecType) -> VectorUniformValue {
        match v {
            VecType::Vec2(s) => {
                let v = self.cast_to_scalar(s).to_vec2();
                VectorUniformValue::Vec2(v)
            }
            VecType::Vec3(s) => {
                let v = self.cast_to_scalar(s).to_vec3();
                VectorUniformValue::Vec3(v)
            }
            VecType::Vec4(s) => {
                let v = self.cast_to_scalar(s).to_vec4();
                VectorUniformValue::Vec4(v)
            }
        }
    }

    fn to_vec2(self) -> Vec2UniformValue {
        match self {
            ScalarUniformValue::U32(s) => Vec2UniformValue::U32(s, 0),
            ScalarUniformValue::I32(s) => Vec2UniformValue::I32(s, 0),
            ScalarUniformValue::F32(s) => Vec2UniformValue::F32(s, 0.0),
        }
    }

    fn to_vec3(self) -> Vec3UniformValue {
        match self {
            ScalarUniformValue::U32(s) => Vec3UniformValue::U32(s, 0, 0),
            ScalarUniformValue::I32(s) => Vec3UniformValue::I32(s, 0, 0),
            ScalarUniformValue::F32(s) => Vec3UniformValue::F32(s, 0.0, 0.0),
        }
    }

    fn to_vec4(self) -> Vec4UniformValue {
        match self {
            ScalarUniformValue::U32(s) => Vec4UniformValue::U32(s, 0, 0, 0),
            ScalarUniformValue::I32(s) => Vec4UniformValue::I32(s, 0, 0, 0),
            ScalarUniformValue::F32(s) => Vec4UniformValue::F32(s, 0.0, 0.0, 0.0),
        }
    }

    fn cast_to_matrix(&self, m: MatrixType) -> MatrixUniformValue {
        // TODO: Maybe keep as much information as possible, like with other type casts
        match m {
            MatrixType::M2x2 => MatrixUniformValue::M2x2(Column2(0.0, 0.0), Column2(0.0, 0.0)),
            MatrixType::M2x3 => {
                MatrixUniformValue::M2x3(Column3(0.0, 0.0, 0.0), Column3(0.0, 0.0, 0.0))
            }
            MatrixType::M2x4 => {
                MatrixUniformValue::M2x4(Column4(0.0, 0.0, 0.0, 0.0), Column4(0.0, 0.0, 0.0, 0.0))
            }

            MatrixType::M3x2 => {
                MatrixUniformValue::M3x2(Column2(0.0, 0.0), Column2(0.0, 0.0), Column2(0.0, 0.0))
            }
            MatrixType::M3x3 => MatrixUniformValue::M3x3(
                Column3(0.0, 0.0, 0.0),
                Column3(0.0, 0.0, 0.0),
                Column3(0.0, 0.0, 0.0),
            ),
            MatrixType::M3x4 => MatrixUniformValue::M3x4(
                Column4(0.0, 0.0, 0.0, 0.0),
                Column4(0.0, 0.0, 0.0, 0.0),
                Column4(0.0, 0.0, 0.0, 0.0),
            ),

            MatrixType::M4x2 => MatrixUniformValue::M4x2(
                Column2(0.0, 0.0),
                Column2(0.0, 0.0),
                Column2(0.0, 0.0),
                Column2(0.0, 0.0),
            ),
            MatrixType::M4x3 => MatrixUniformValue::M4x3(
                Column3(0.0, 0.0, 0.0),
                Column3(0.0, 0.0, 0.0),
                Column3(0.0, 0.0, 0.0),
                Column3(0.0, 0.0, 0.0),
            ),
            MatrixType::M4x4 => MatrixUniformValue::M4x4(
                Column4(0.0, 0.0, 0.0, 0.0),
                Column4(0.0, 0.0, 0.0, 0.0),
                Column4(0.0, 0.0, 0.0, 0.0),
                Column4(0.0, 0.0, 0.0, 0.0),
            ),
        }
    }

    fn number_edit(
        ui: &Ui,
        group_index: usize,
        binding_index: usize,
        message: &mut Option<UniformEditEvent>,
    ) {
        if ui.button(format!("+##add_{group_index}_{binding_index}")) {
            *message = Some(UniformEditEvent::Increase(group_index, binding_index))
        }
        ui.same_line();
        if ui.button(format!("-##decrease_{group_index}_{binding_index}")) {
            *message = Some(UniformEditEvent::Decrease(group_index, binding_index))
        }
    }

    fn cast_to_transform(&self) -> TransformUniformValue {
        TransformUniformValue::default()
    }

    pub(crate) fn from_json(uniform: &Map<String, JsonValue>) -> Option<ScalarUniformValue> {
        let value = uniform.get("value")?;
        let inner_type = uniform.get("innertype")?.as_str()?;
        match inner_type {
            "f32" => {
                let val = value.as_f64()? as f32;
                Some(ScalarUniformValue::F32(val))
            },
            "u32" => {
                let val = value.as_u64()? as u32;
                Some(ScalarUniformValue::U32(val))
            },
            "i32" => {
                let val = value.as_i64()? as i32;
                Some(ScalarUniformValue::I32(val))
            },
            _ => None
        }
    }

    pub(crate) fn to_json(&self, json_obj: &mut Map<String, JsonValue>) {
        match self {
            ScalarUniformValue::U32(_) => json_obj.insert("innertype".into(), "u32".into()),
            ScalarUniformValue::I32(_) => json_obj.insert("innertype".into(), "i32".into()),
            ScalarUniformValue::F32(_) => json_obj.insert("innertype".into(), "f32".into()),
        };
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScalarType {
    U32,
    I32,
    F32,
}

impl<'a> From<&'a ScalarType> for Cow<'static, str> {
    fn from(val: &'a ScalarType) -> Cow<'static, str> {
        match val {
            ScalarType::U32 => Cow::Borrowed("u32"),
            ScalarType::I32 => Cow::Borrowed("i32"),
            ScalarType::F32 => Cow::Borrowed("f32"),
        }
    }
}
