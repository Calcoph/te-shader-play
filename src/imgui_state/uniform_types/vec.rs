use std::borrow::Cow;

use imgui::Ui;
use mint::{Vector3, Vector4};
use serde_json::{Map, Value as JsonValue};

use crate::imgui_state::{
    uniform_types::{scalar::ScalarPrimitive, ScalarType},
    ImguiUniformSelectable, ImguiVec, UniformEditEvent,
};

use super::{
    cast_f32_u32, cast_i32_u32,
    matrix::{Column2, Column3, Column4, MatrixUniformValue},
    scalar::ScalarUniformValue,
    transform::TransformUniformValue,
    MatrixType, UniformType, UniformValue,
};

trait VecUniformValue {
    fn show_editor(
        &mut self,
        ui: &Ui,
        group_index: usize,
        binding_index: usize,
        message: &mut Option<UniformEditEvent>,
    );
    fn change_inner_type(&mut self, inner_type: ScalarType);
    fn to_le_bytes(&self) -> Vec<u8>;
    fn cast_to_scalar(&self, s: ScalarType) -> UniformValue;
    fn cast_to_vec(&self, v: VecType) -> UniformValue;
    fn cast_to_matrix(&self, m: MatrixType) -> UniformValue;
    fn cast_to_transform(&self) -> UniformValue;
    fn from_json(json_val: &Map<String, JsonValue>) -> Option<Self> where Self: Sized;
    fn to_json(&self, json_obj: &mut Map<String, JsonValue>);
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Vec2UniformValue {
    U32(u32, u32),
    I32(i32, i32),
    F32(f32, f32),
}

impl VecUniformValue for Vec2UniformValue {
    fn show_editor(
        &mut self,
        ui: &Ui,
        group_index: usize,
        binding_index: usize,
        message: &mut Option<UniformEditEvent>,
    ) {
        match self {
            Vec2UniformValue::U32(x, y) => {
                let mut vars = [*x, *y];
                if ui
                    .input_scalar_n(format!("##v2edit_{group_index}_{binding_index}"), &mut vars)
                    .build()
                {
                    *x = vars[0];
                    *y = vars[1];
                    *message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
            }
            Vec2UniformValue::I32(x, y) => {
                let mut vars = [*x, *y];
                if ui
                    .input_scalar_n(format!("##v2edit_{group_index}_{binding_index}"), &mut vars)
                    .build()
                {
                    *x = vars[0];
                    *y = vars[1];
                    *message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
            }
            Vec2UniformValue::F32(x, y) => {
                let mut vars = [*x, *y];
                if ui
                    .input_scalar_n(format!("##v2edit_{group_index}_{binding_index}"), &mut vars)
                    .build()
                {
                    *x = vars[0];
                    *y = vars[1];
                    *message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
            }
        }
    }

    fn change_inner_type(&mut self, inner_type: ScalarType) {
        match self {
            Vec2UniformValue::U32(x, y) => match inner_type {
                ScalarType::U32 => (),
                ScalarType::I32 => *self = Vec2UniformValue::I32(*x as i32, *y as i32),
                ScalarType::F32 => *self = Vec2UniformValue::F32(*x as f32, *y as f32),
            },
            Vec2UniformValue::I32(x, y) => match inner_type {
                ScalarType::U32 => {
                    *self = Vec2UniformValue::U32(cast_i32_u32(*x), cast_i32_u32(*y))
                }
                ScalarType::I32 => (),
                ScalarType::F32 => *self = Vec2UniformValue::F32(*x as f32, *y as f32),
            },
            Vec2UniformValue::F32(x, y) => match inner_type {
                ScalarType::U32 => {
                    *self = Vec2UniformValue::U32(cast_f32_u32(*x), cast_f32_u32(*y))
                }
                ScalarType::I32 => *self = Vec2UniformValue::I32(*x as i32, *y as i32),
                ScalarType::F32 => (),
            },
        }
    }

    fn to_le_bytes(&self) -> Vec<u8> {
        match self {
            Vec2UniformValue::U32(x, y) => {
                x.to_le_bytes().into_iter().chain(y.to_le_bytes()).collect()
            }
            Vec2UniformValue::I32(x, y) => {
                x.to_le_bytes().into_iter().chain(y.to_le_bytes()).collect()
            }
            Vec2UniformValue::F32(x, y) => {
                x.to_le_bytes().into_iter().chain(y.to_le_bytes()).collect()
            }
        }
    }

    fn cast_to_scalar(&self, s: ScalarType) -> UniformValue {
        UniformValue::Scalar(match self {
            Vec2UniformValue::U32(x, _) => match s {
                ScalarType::U32 => ScalarUniformValue::U32(*x),
                ScalarType::I32 => ScalarUniformValue::I32(*x as i32),
                ScalarType::F32 => ScalarUniformValue::F32(*x as f32),
            },
            Vec2UniformValue::I32(x, _) => match s {
                ScalarType::U32 => ScalarUniformValue::U32(cast_i32_u32(*x)),
                ScalarType::I32 => ScalarUniformValue::I32(*x),
                ScalarType::F32 => ScalarUniformValue::F32(*x as f32),
            },
            Vec2UniformValue::F32(x, _) => match s {
                ScalarType::U32 => ScalarUniformValue::U32(cast_f32_u32(*x)),
                ScalarType::I32 => ScalarUniformValue::I32(*x as i32),
                ScalarType::F32 => ScalarUniformValue::F32(*x),
            },
        })
    }

    fn cast_to_vec(&self, v: VecType) -> UniformValue {
        use ScalarPrimitive as SP;
        let scalar_type = match v {
            VecType::Vec2(v) => v,
            VecType::Vec3(v) => v,
            VecType::Vec4(v) => v,
        };

        let vec4 = match self {
            Vec2UniformValue::U32(x, y) => match scalar_type {
                ScalarType::U32 => [SP { u32: *x }, SP { u32: *y }, SP { u32: 0 }, SP { u32: 0 }],
                ScalarType::I32 => [
                    SP { i32: *x as i32 },
                    SP { i32: *y as i32 },
                    SP { i32: 0 },
                    SP { i32: 0 },
                ],
                ScalarType::F32 => [
                    SP { f32: *x as f32 },
                    SP { f32: *y as f32 },
                    SP { f32: 0.0 },
                    SP { f32: 0.0 },
                ],
            },
            Vec2UniformValue::I32(x, y) => match scalar_type {
                ScalarType::U32 => [
                    SP {
                        u32: cast_i32_u32(*x),
                    },
                    SP {
                        u32: cast_i32_u32(*y),
                    },
                    SP { u32: 0 },
                    SP { u32: 0 },
                ],
                ScalarType::I32 => [SP { i32: *x }, SP { i32: *y }, SP { i32: 0 }, SP { i32: 0 }],
                ScalarType::F32 => [
                    SP { f32: *x as f32 },
                    SP { f32: *y as f32 },
                    SP { f32: 0.0 },
                    SP { f32: 0.0 },
                ],
            },
            Vec2UniformValue::F32(x, y) => match scalar_type {
                ScalarType::U32 => [
                    SP {
                        u32: cast_f32_u32(*x),
                    },
                    SP {
                        u32: cast_f32_u32(*y),
                    },
                    SP { u32: 0 },
                    SP { u32: 0 },
                ],
                ScalarType::I32 => [
                    SP { i32: *x as i32 },
                    SP { i32: *y as i32 },
                    SP { i32: 0 },
                    SP { i32: 0 },
                ],
                ScalarType::F32 => [
                    SP { f32: *x },
                    SP { f32: *y },
                    SP { f32: 0.0 },
                    SP { f32: 0.0 },
                ],
            },
        };

        unsafe {
            UniformValue::Vector(match v {
                VecType::Vec2(scalar_type) => match scalar_type {
                    ScalarType::U32 => {
                        VectorUniformValue::Vec2(Vec2UniformValue::U32(vec4[0].u32, vec4[1].u32))
                    }
                    ScalarType::I32 => {
                        VectorUniformValue::Vec2(Vec2UniformValue::I32(vec4[0].i32, vec4[1].i32))
                    }
                    ScalarType::F32 => {
                        VectorUniformValue::Vec2(Vec2UniformValue::F32(vec4[0].f32, vec4[1].f32))
                    }
                },
                VecType::Vec3(scalar_type) => match scalar_type {
                    ScalarType::U32 => VectorUniformValue::Vec3(Vec3UniformValue::U32(
                        vec4[0].u32,
                        vec4[1].u32,
                        vec4[0].u32,
                    )),
                    ScalarType::I32 => VectorUniformValue::Vec3(Vec3UniformValue::I32(
                        vec4[0].i32,
                        vec4[1].i32,
                        vec4[0].i32,
                    )),
                    ScalarType::F32 => VectorUniformValue::Vec3(Vec3UniformValue::F32(
                        vec4[0].f32,
                        vec4[1].f32,
                        vec4[0].f32,
                    )),
                },
                VecType::Vec4(scalar_type) => match scalar_type {
                    ScalarType::U32 => VectorUniformValue::Vec4(Vec4UniformValue::U32(
                        vec4[0].u32,
                        vec4[1].u32,
                        vec4[0].u32,
                        vec4[1].u32,
                    )),
                    ScalarType::I32 => VectorUniformValue::Vec4(Vec4UniformValue::I32(
                        vec4[0].i32,
                        vec4[1].i32,
                        vec4[0].i32,
                        vec4[1].i32,
                    )),
                    ScalarType::F32 => VectorUniformValue::Vec4(Vec4UniformValue::F32(
                        vec4[0].f32,
                        vec4[1].f32,
                        vec4[0].f32,
                        vec4[1].f32,
                    )),
                },
            })
        }
    }

    fn cast_to_matrix(&self, m: MatrixType) -> UniformValue {
        // TODO: Maybe keep as much information as possible, like with other types
        UniformValue::Matrix(match m {
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
        })
    }

    fn cast_to_transform(&self) -> UniformValue {
        UniformValue::Transform(TransformUniformValue::default())
    }

    fn from_json(json_val: &Map<String, JsonValue>) -> Option<Self> {
        let inner_type_2 = json_val.get("innertype2")?;
        let i0 = json_val.get("item0")?;
        let i1 = json_val.get("item1")?;
        match inner_type_2.as_str()? {
            "f32" => Some(Vec2UniformValue::F32(i0.as_f64()? as f32, i1.as_f64()? as f32)),
            "u32" => Some(Vec2UniformValue::U32(i0.as_u64()? as u32, i1.as_u64()? as u32)),
            "i32" => Some(Vec2UniformValue::I32(i0.as_i64()? as i32, i1.as_i64()? as i32)),
            _ => None
        }
    }

    fn to_json(&self, json_obj: &mut Map<String, JsonValue>) {
        match self {
            Vec2UniformValue::U32(_, _) => json_obj.insert("innertype2".into(), "u32".into()),
            Vec2UniformValue::I32(_, _) => json_obj.insert("innertype2".into(), "i32".into()),
            Vec2UniformValue::F32(_, _) => json_obj.insert("innertype2".into(), "f32".into()),
        };

        let (i0, i1): (JsonValue,JsonValue) = match self {
            Vec2UniformValue::U32(i0, i1) => ((*i0).into(),(*i1).into()),
            Vec2UniformValue::I32(i0, i1) => ((*i0).into(),(*i1).into()),
            Vec2UniformValue::F32(i0, i1) => ((*i0).into(),(*i1).into()),
        };

        json_obj.insert("item0".into(), i0);
        json_obj.insert("item1".into(), i1);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Vec3UniformValue {
    U32(u32, u32, u32),
    I32(i32, i32, i32),
    F32(f32, f32, f32),
}

impl VecUniformValue for Vec3UniformValue {
    fn show_editor(
        &mut self,
        ui: &Ui,
        group_index: usize,
        binding_index: usize,
        message: &mut Option<UniformEditEvent>,
    ) {
        match self {
            Vec3UniformValue::U32(x, y, z) => {
                let mut vars = [*x, *y, *z];
                if ui
                    .input_scalar_n(format!("##v3edit_{group_index}_{binding_index}"), &mut vars)
                    .build()
                {
                    *x = vars[0];
                    *y = vars[1];
                    *z = vars[2];
                    *message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
            }
            Vec3UniformValue::I32(x, y, z) => {
                let mut vars = [*x, *y, *z];
                if ui
                    .input_scalar_n(format!("##v3edit_{group_index}_{binding_index}"), &mut vars)
                    .build()
                {
                    *x = vars[0];
                    *y = vars[1];
                    *z = vars[2];
                    *message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
            }
            Vec3UniformValue::F32(x, y, z) => {
                let mut c_vars = Vector3 {
                    x: *x,
                    y: *y,
                    z: *z,
                };
                let mut vars = [*x, *y, *z];
                if ui
                    .input_scalar_n(format!("##v3edit_{group_index}_{binding_index}"), &mut vars)
                    .build()
                {
                    *x = vars[0];
                    *y = vars[1];
                    *z = vars[2];
                    *message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
                if ui.color_edit3(
                    format!("##v3edit_{group_index}_{binding_index}"),
                    &mut c_vars,
                ) {
                    *x = c_vars.x;
                    *y = c_vars.y;
                    *z = c_vars.z;
                    *message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
            }
        }
    }

    fn change_inner_type(&mut self, inner_type: ScalarType) {
        match self {
            Vec3UniformValue::U32(x, y, z) => match inner_type {
                ScalarType::U32 => (),
                ScalarType::I32 => *self = Vec3UniformValue::I32(*x as i32, *y as i32, *z as i32),
                ScalarType::F32 => *self = Vec3UniformValue::F32(*x as f32, *y as f32, *z as f32),
            },
            Vec3UniformValue::I32(x, y, z) => match inner_type {
                ScalarType::U32 => {
                    *self =
                        Vec3UniformValue::U32(cast_i32_u32(*x), cast_i32_u32(*y), cast_i32_u32(*z))
                }
                ScalarType::I32 => (),
                ScalarType::F32 => *self = Vec3UniformValue::F32(*x as f32, *y as f32, *z as f32),
            },
            Vec3UniformValue::F32(x, y, z) => match inner_type {
                ScalarType::U32 => {
                    *self =
                        Vec3UniformValue::U32(cast_f32_u32(*x), cast_f32_u32(*y), cast_f32_u32(*z))
                }
                ScalarType::I32 => *self = Vec3UniformValue::I32(*x as i32, *y as i32, *z as i32),
                ScalarType::F32 => (),
            },
        }
    }

    fn to_le_bytes(&self) -> Vec<u8> {
        match self {
            Vec3UniformValue::U32(x, y, z) => x
                .to_le_bytes()
                .into_iter()
                .chain(y.to_le_bytes())
                .chain(z.to_le_bytes())
                .collect(),
            Vec3UniformValue::I32(x, y, z) => x
                .to_le_bytes()
                .into_iter()
                .chain(y.to_le_bytes())
                .chain(z.to_le_bytes())
                .collect(),
            Vec3UniformValue::F32(x, y, z) => x
                .to_le_bytes()
                .into_iter()
                .chain(y.to_le_bytes())
                .chain(z.to_le_bytes())
                .collect(),
        }
    }

    fn cast_to_scalar(&self, s: ScalarType) -> UniformValue {
        UniformValue::Scalar(match self {
            Vec3UniformValue::U32(x, _, _) => match s {
                ScalarType::U32 => ScalarUniformValue::U32(*x),
                ScalarType::I32 => ScalarUniformValue::I32(*x as i32),
                ScalarType::F32 => ScalarUniformValue::F32(*x as f32),
            },
            Vec3UniformValue::I32(x, _, _) => match s {
                ScalarType::U32 => ScalarUniformValue::U32(cast_i32_u32(*x)),
                ScalarType::I32 => ScalarUniformValue::I32(*x),
                ScalarType::F32 => ScalarUniformValue::F32(*x as f32),
            },
            Vec3UniformValue::F32(x, _, _) => match s {
                ScalarType::U32 => ScalarUniformValue::U32(cast_f32_u32(*x)),
                ScalarType::I32 => ScalarUniformValue::I32(*x as i32),
                ScalarType::F32 => ScalarUniformValue::F32(*x),
            },
        })
    }

    fn cast_to_vec(&self, v: VecType) -> UniformValue {
        use ScalarPrimitive as SP;
        let scalar_type = match v {
            VecType::Vec2(v) => v,
            VecType::Vec3(v) => v,
            VecType::Vec4(v) => v,
        };

        let vec4 = match self {
            Vec3UniformValue::U32(x, y, z) => match scalar_type {
                ScalarType::U32 => [
                    SP { u32: *x },
                    SP { u32: *y },
                    SP { u32: *z },
                    SP { u32: 0 },
                ],
                ScalarType::I32 => [
                    SP { i32: *x as i32 },
                    SP { i32: *y as i32 },
                    SP { i32: *z as i32 },
                    SP { i32: 0 },
                ],
                ScalarType::F32 => [
                    SP { f32: *x as f32 },
                    SP { f32: *y as f32 },
                    SP { f32: *z as f32 },
                    SP { f32: 0.0 },
                ],
            },
            Vec3UniformValue::I32(x, y, z) => match scalar_type {
                ScalarType::U32 => [
                    SP {
                        u32: cast_i32_u32(*x),
                    },
                    SP {
                        u32: cast_i32_u32(*y),
                    },
                    SP {
                        u32: cast_i32_u32(*z),
                    },
                    SP { u32: 0 },
                ],
                ScalarType::I32 => [
                    SP { i32: *x },
                    SP { i32: *y },
                    SP { i32: *z },
                    SP { i32: *z },
                ],
                ScalarType::F32 => [
                    SP { f32: *x as f32 },
                    SP { f32: *y as f32 },
                    SP { f32: *z as f32 },
                    SP { f32: 0.0 },
                ],
            },
            Vec3UniformValue::F32(x, y, z) => match scalar_type {
                ScalarType::U32 => [
                    SP {
                        u32: cast_f32_u32(*x),
                    },
                    SP {
                        u32: cast_f32_u32(*y),
                    },
                    SP {
                        u32: cast_f32_u32(*z),
                    },
                    SP { u32: 0 },
                ],
                ScalarType::I32 => [
                    SP { i32: *x as i32 },
                    SP { i32: *y as i32 },
                    SP { i32: *z as i32 },
                    SP { i32: 0 },
                ],
                ScalarType::F32 => [
                    SP { f32: *x },
                    SP { f32: *y },
                    SP { f32: *z },
                    SP { f32: 0.0 },
                ],
            },
        };

        unsafe {
            UniformValue::Vector(match v {
                VecType::Vec2(scalar_type) => match scalar_type {
                    ScalarType::U32 => {
                        VectorUniformValue::Vec2(Vec2UniformValue::U32(vec4[0].u32, vec4[1].u32))
                    }
                    ScalarType::I32 => {
                        VectorUniformValue::Vec2(Vec2UniformValue::I32(vec4[0].i32, vec4[1].i32))
                    }
                    ScalarType::F32 => {
                        VectorUniformValue::Vec2(Vec2UniformValue::F32(vec4[0].f32, vec4[1].f32))
                    }
                },
                VecType::Vec3(scalar_type) => match scalar_type {
                    ScalarType::U32 => VectorUniformValue::Vec3(Vec3UniformValue::U32(
                        vec4[0].u32,
                        vec4[1].u32,
                        vec4[0].u32,
                    )),
                    ScalarType::I32 => VectorUniformValue::Vec3(Vec3UniformValue::I32(
                        vec4[0].i32,
                        vec4[1].i32,
                        vec4[0].i32,
                    )),
                    ScalarType::F32 => VectorUniformValue::Vec3(Vec3UniformValue::F32(
                        vec4[0].f32,
                        vec4[1].f32,
                        vec4[0].f32,
                    )),
                },
                VecType::Vec4(scalar_type) => match scalar_type {
                    ScalarType::U32 => VectorUniformValue::Vec4(Vec4UniformValue::U32(
                        vec4[0].u32,
                        vec4[1].u32,
                        vec4[0].u32,
                        vec4[1].u32,
                    )),
                    ScalarType::I32 => VectorUniformValue::Vec4(Vec4UniformValue::I32(
                        vec4[0].i32,
                        vec4[1].i32,
                        vec4[0].i32,
                        vec4[1].i32,
                    )),
                    ScalarType::F32 => VectorUniformValue::Vec4(Vec4UniformValue::F32(
                        vec4[0].f32,
                        vec4[1].f32,
                        vec4[0].f32,
                        vec4[1].f32,
                    )),
                },
            })
        }
    }

    fn cast_to_matrix(&self, m: MatrixType) -> UniformValue {
        // TODO: Maybe keep as much information as possible, like with other types
        UniformValue::Matrix(match m {
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
        })
    }

    fn cast_to_transform(&self) -> UniformValue {
        UniformValue::Transform(TransformUniformValue::default())
    }

    fn from_json(json_val: &Map<String, JsonValue>) -> Option<Self> where Self: Sized {
        let inner_type_2 = json_val.get("innertype2")?.as_str()?;
        let i0 = json_val.get("item0")?;
        let i1 = json_val.get("item1")?;
        let i2 = json_val.get("item2")?;
        match inner_type_2 {
            "f32" => Some(Vec3UniformValue::F32(i0.as_f64()? as f32, i1.as_f64()? as f32, i2.as_f64()? as f32)),
            "u32" => Some(Vec3UniformValue::U32(i0.as_u64()? as u32, i1.as_u64()? as u32, i2.as_u64()? as u32)),
            "i32" => Some(Vec3UniformValue::I32(i0.as_i64()? as i32, i1.as_i64()? as i32, i2.as_i64()? as i32)),
            _ => None
        }
    }

    fn to_json(&self, json_obj: &mut Map<String, JsonValue>) {
        match self {
            Vec3UniformValue::U32(_, _, _) => json_obj.insert("innertype2".into(), "u32".into()),
            Vec3UniformValue::I32(_, _, _) => json_obj.insert("innertype2".into(), "i32".into()),
            Vec3UniformValue::F32(_, _, _) => json_obj.insert("innertype2".into(), "f32".into()),
        };

        let (i0, i1, i2): (JsonValue,JsonValue,JsonValue) = match self {
            Vec3UniformValue::U32(i0, i1, i2) => ((*i0).into(),(*i1).into(),(*i2).into()),
            Vec3UniformValue::I32(i0, i1, i2) => ((*i0).into(),(*i1).into(),(*i2).into()),
            Vec3UniformValue::F32(i0, i1, i2) => ((*i0).into(),(*i1).into(),(*i2).into()),
        };

        json_obj.insert("item0".into(), i0);
        json_obj.insert("item1".into(), i1);
        json_obj.insert("item2".into(), i2);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Vec4UniformValue {
    U32(u32, u32, u32, u32),
    I32(i32, i32, i32, i32),
    F32(f32, f32, f32, f32),
}

impl VecUniformValue for Vec4UniformValue {
    fn show_editor(
        &mut self,
        ui: &Ui,
        group_index: usize,
        binding_index: usize,
        message: &mut Option<UniformEditEvent>,
    ) {
        match self {
            Vec4UniformValue::U32(x, y, z, w) => {
                let mut vars = [*x, *y, *z, *w];
                if ui
                    .input_scalar_n(format!("##v4edit_{group_index}_{binding_index}"), &mut vars)
                    .build()
                {
                    *x = vars[0];
                    *y = vars[1];
                    *z = vars[2];
                    *w = vars[3];
                    *message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
            }
            Vec4UniformValue::I32(x, y, z, w) => {
                let mut vars = [*x, *y, *z, *w];
                if ui
                    .input_scalar_n(format!("##v4edit_{group_index}_{binding_index}"), &mut vars)
                    .build()
                {
                    *x = vars[0];
                    *y = vars[1];
                    *z = vars[2];
                    *w = vars[3];
                    *message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
            }
            Vec4UniformValue::F32(x, y, z, w) => {
                let mut c_vars = Vector4 {
                    x: *x,
                    y: *y,
                    z: *z,
                    w: *w,
                };
                let mut vars = [*x, *y, *z, *w];
                if ui
                    .input_scalar_n(format!("##v4edit_{group_index}_{binding_index}"), &mut vars)
                    .build()
                {
                    *x = vars[0];
                    *y = vars[1];
                    *z = vars[2];
                    *w = vars[3];
                    *message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
                if ui.color_edit4(
                    format!("##v4edit_{group_index}_{binding_index}"),
                    &mut c_vars,
                ) {
                    *x = c_vars.x;
                    *y = c_vars.y;
                    *z = c_vars.z;
                    *w = c_vars.w;
                    *message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
            }
        }
    }

    fn change_inner_type(&mut self, inner_type: ScalarType) {
        match self {
            Vec4UniformValue::U32(x, y, z, w) => match inner_type {
                ScalarType::U32 => (),
                ScalarType::I32 => {
                    *self = Vec4UniformValue::I32(*x as i32, *y as i32, *z as i32, *w as i32)
                }
                ScalarType::F32 => {
                    *self = Vec4UniformValue::F32(*x as f32, *y as f32, *z as f32, *w as f32)
                }
            },
            Vec4UniformValue::I32(x, y, z, w) => match inner_type {
                ScalarType::U32 => {
                    *self = Vec4UniformValue::U32(
                        cast_i32_u32(*x),
                        cast_i32_u32(*y),
                        cast_i32_u32(*z),
                        cast_i32_u32(*w),
                    )
                }
                ScalarType::I32 => (),
                ScalarType::F32 => {
                    *self = Vec4UniformValue::F32(*x as f32, *y as f32, *z as f32, *w as f32)
                }
            },
            Vec4UniformValue::F32(x, y, z, w) => match inner_type {
                ScalarType::U32 => {
                    *self = Vec4UniformValue::U32(
                        cast_f32_u32(*x),
                        cast_f32_u32(*y),
                        cast_f32_u32(*z),
                        cast_f32_u32(*w),
                    )
                }
                ScalarType::I32 => {
                    *self = Vec4UniformValue::I32(*x as i32, *y as i32, *z as i32, *w as i32)
                }
                ScalarType::F32 => (),
            },
        }
    }

    fn to_le_bytes(&self) -> Vec<u8> {
        match self {
            Vec4UniformValue::U32(x, y, z, w) => x
                .to_le_bytes()
                .into_iter()
                .chain(y.to_le_bytes())
                .chain(z.to_le_bytes())
                .chain(w.to_le_bytes())
                .collect(),
            Vec4UniformValue::I32(x, y, z, w) => x
                .to_le_bytes()
                .into_iter()
                .chain(y.to_le_bytes())
                .chain(z.to_le_bytes())
                .chain(w.to_le_bytes())
                .collect(),
            Vec4UniformValue::F32(x, y, z, w) => x
                .to_le_bytes()
                .into_iter()
                .chain(y.to_le_bytes())
                .chain(z.to_le_bytes())
                .chain(w.to_le_bytes())
                .collect(),
        }
    }

    fn cast_to_scalar(&self, s: ScalarType) -> UniformValue {
        UniformValue::Scalar(match self {
            Vec4UniformValue::U32(x, _, _, _) => match s {
                ScalarType::U32 => ScalarUniformValue::U32(*x),
                ScalarType::I32 => ScalarUniformValue::I32(*x as i32),
                ScalarType::F32 => ScalarUniformValue::F32(*x as f32),
            },
            Vec4UniformValue::I32(x, _, _, _) => match s {
                ScalarType::U32 => ScalarUniformValue::U32(cast_i32_u32(*x)),
                ScalarType::I32 => ScalarUniformValue::I32(*x),
                ScalarType::F32 => ScalarUniformValue::F32(*x as f32),
            },
            Vec4UniformValue::F32(x, _, _, _) => match s {
                ScalarType::U32 => ScalarUniformValue::U32(cast_f32_u32(*x)),
                ScalarType::I32 => ScalarUniformValue::I32(*x as i32),
                ScalarType::F32 => ScalarUniformValue::F32(*x),
            },
        })
    }

    fn cast_to_vec(&self, v: VecType) -> UniformValue {
        use ScalarPrimitive as SP;
        let scalar_type = match v {
            VecType::Vec2(v) => v,
            VecType::Vec3(v) => v,
            VecType::Vec4(v) => v,
        };

        let vec4 = match self {
            Vec4UniformValue::U32(x, y, z, w) => match scalar_type {
                ScalarType::U32 => [
                    SP { u32: *x },
                    SP { u32: *y },
                    SP { u32: *z },
                    SP { u32: *w },
                ],
                ScalarType::I32 => [
                    SP { i32: *x as i32 },
                    SP { i32: *y as i32 },
                    SP { i32: *z as i32 },
                    SP { i32: *w as i32 },
                ],
                ScalarType::F32 => [
                    SP { f32: *x as f32 },
                    SP { f32: *y as f32 },
                    SP { f32: *z as f32 },
                    SP { f32: *w as f32 },
                ],
            },
            Vec4UniformValue::I32(x, y, z, w) => match scalar_type {
                ScalarType::U32 => [
                    SP {
                        u32: cast_i32_u32(*x),
                    },
                    SP {
                        u32: cast_i32_u32(*y),
                    },
                    SP {
                        u32: cast_i32_u32(*z),
                    },
                    SP {
                        u32: cast_i32_u32(*w),
                    },
                ],
                ScalarType::I32 => [
                    SP { i32: *x },
                    SP { i32: *y },
                    SP { i32: *z },
                    SP { i32: *w },
                ],
                ScalarType::F32 => [
                    SP { f32: *x as f32 },
                    SP { f32: *y as f32 },
                    SP { f32: *z as f32 },
                    SP { f32: *w as f32 },
                ],
            },
            Vec4UniformValue::F32(x, y, z, w) => match scalar_type {
                ScalarType::U32 => [
                    SP {
                        u32: cast_f32_u32(*x),
                    },
                    SP {
                        u32: cast_f32_u32(*y),
                    },
                    SP {
                        u32: cast_f32_u32(*z),
                    },
                    SP {
                        u32: cast_f32_u32(*w),
                    },
                ],
                ScalarType::I32 => [
                    SP { i32: *x as i32 },
                    SP { i32: *y as i32 },
                    SP { i32: *z as i32 },
                    SP { i32: *w as i32 },
                ],
                ScalarType::F32 => [
                    SP { f32: *x },
                    SP { f32: *y },
                    SP { f32: *z },
                    SP { f32: *w },
                ],
            },
        };

        unsafe {
            UniformValue::Vector(match v {
                VecType::Vec2(scalar_type) => match scalar_type {
                    ScalarType::U32 => {
                        VectorUniformValue::Vec2(Vec2UniformValue::U32(vec4[0].u32, vec4[1].u32))
                    }
                    ScalarType::I32 => {
                        VectorUniformValue::Vec2(Vec2UniformValue::I32(vec4[0].i32, vec4[1].i32))
                    }
                    ScalarType::F32 => {
                        VectorUniformValue::Vec2(Vec2UniformValue::F32(vec4[0].f32, vec4[1].f32))
                    }
                },
                VecType::Vec3(scalar_type) => match scalar_type {
                    ScalarType::U32 => VectorUniformValue::Vec3(Vec3UniformValue::U32(
                        vec4[0].u32,
                        vec4[1].u32,
                        vec4[0].u32,
                    )),
                    ScalarType::I32 => VectorUniformValue::Vec3(Vec3UniformValue::I32(
                        vec4[0].i32,
                        vec4[1].i32,
                        vec4[0].i32,
                    )),
                    ScalarType::F32 => VectorUniformValue::Vec3(Vec3UniformValue::F32(
                        vec4[0].f32,
                        vec4[1].f32,
                        vec4[0].f32,
                    )),
                },
                VecType::Vec4(scalar_type) => match scalar_type {
                    ScalarType::U32 => VectorUniformValue::Vec4(Vec4UniformValue::U32(
                        vec4[0].u32,
                        vec4[1].u32,
                        vec4[0].u32,
                        vec4[1].u32,
                    )),
                    ScalarType::I32 => VectorUniformValue::Vec4(Vec4UniformValue::I32(
                        vec4[0].i32,
                        vec4[1].i32,
                        vec4[0].i32,
                        vec4[1].i32,
                    )),
                    ScalarType::F32 => VectorUniformValue::Vec4(Vec4UniformValue::F32(
                        vec4[0].f32,
                        vec4[1].f32,
                        vec4[0].f32,
                        vec4[1].f32,
                    )),
                },
            })
        }
    }

    fn cast_to_matrix(&self, m: MatrixType) -> UniformValue {
        // TODO: Maybe keep as much information as possible, like with other types
        UniformValue::Matrix(match m {
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
        })
    }

    fn cast_to_transform(&self) -> UniformValue {
        UniformValue::Transform(TransformUniformValue::default())
    }

    fn from_json(json_val: &Map<String, JsonValue>) -> Option<Self> where Self: Sized {
        let inner_type_2 = json_val.get("innertype2")?.as_str()?;
        let i0 = json_val.get("item0")?;
        let i1 = json_val.get("item1")?;
        let i2 = json_val.get("item2")?;
        let i3 = json_val.get("item3")?;
        match inner_type_2 {
            "f32" => Some(Vec4UniformValue::F32(i0.as_f64()? as f32, i1.as_f64()? as f32, i2.as_f64()? as f32, i3.as_f64()? as f32)),
            "u32" => Some(Vec4UniformValue::U32(i0.as_u64()? as u32, i1.as_u64()? as u32, i2.as_u64()? as u32, i3.as_u64()? as u32)),
            "i32" => Some(Vec4UniformValue::I32(i0.as_i64()? as i32, i1.as_i64()? as i32, i2.as_i64()? as i32, i3.as_i64()? as i32)),
            _ => None
        }
    }

    fn to_json(&self, json_obj: &mut Map<String, JsonValue>) {
        match self {
            Vec4UniformValue::U32(_, _, _, _) => json_obj.insert("innertype2".into(), "u32".into()),
            Vec4UniformValue::I32(_, _, _, _) => json_obj.insert("innertype2".into(), "i32".into()),
            Vec4UniformValue::F32(_, _, _, _) => json_obj.insert("innertype2".into(), "f32".into()),
        };

        let (i0, i1, i2, i3): (JsonValue,JsonValue,JsonValue,JsonValue) = match self {
            Vec4UniformValue::U32(i0, i1, i2, i3) => ((*i0).into(),(*i1).into(),(*i2).into(),(*i3).into()),
            Vec4UniformValue::I32(i0, i1, i2, i3) => ((*i0).into(),(*i1).into(),(*i2).into(),(*i3).into()),
            Vec4UniformValue::F32(i0, i1, i2, i3) => ((*i0).into(),(*i1).into(),(*i2).into(),(*i3).into()),
        };

        json_obj.insert("item0".into(), i0);
        json_obj.insert("item1".into(), i1);
        json_obj.insert("item2".into(), i2);
        json_obj.insert("item3".into(), i3);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum VectorUniformValue {
    Vec2(Vec2UniformValue),
    Vec3(Vec3UniformValue),
    Vec4(Vec4UniformValue),
}

impl ImguiUniformSelectable for VectorUniformValue {
    fn cast_to(&self, casted_type: UniformType) -> UniformValue {
        match casted_type {
            UniformType::Scalar(s) => match self {
                VectorUniformValue::Vec2(v) => v.cast_to_scalar(s),
                VectorUniformValue::Vec3(v) => v.cast_to_scalar(s),
                VectorUniformValue::Vec4(v) => v.cast_to_scalar(s),
            },
            UniformType::Vec(v) => match self {
                VectorUniformValue::Vec2(vec) => vec.cast_to_vec(v),
                VectorUniformValue::Vec3(vec) => vec.cast_to_vec(v),
                VectorUniformValue::Vec4(vec) => vec.cast_to_vec(v),
            },
            UniformType::Matrix(m) => match self {
                VectorUniformValue::Vec2(v) => v.cast_to_matrix(m),
                VectorUniformValue::Vec3(v) => v.cast_to_matrix(m),
                VectorUniformValue::Vec4(v) => v.cast_to_matrix(m),
            },
            UniformType::Transform => match self {
                VectorUniformValue::Vec2(v) => v.cast_to_transform(),
                VectorUniformValue::Vec3(v) => v.cast_to_transform(),
                VectorUniformValue::Vec4(v) => v.cast_to_transform(),
            },
        }
    }

    fn show_editor(
        &mut self,
        ui: &Ui,
        group_index: usize,
        binding_index: usize,
        val_name: &mut String,
    ) -> Option<UniformEditEvent> {
        let mut message = None;
        match self {
            VectorUniformValue::Vec2(v) => {
                UniformValue::show_primitive_selector(
                    ui,
                    group_index,
                    binding_index,
                    &mut message,
                    3,
                    val_name,
                );
                let inner_type_index = match v {
                    Vec2UniformValue::U32(..) => 0,
                    Vec2UniformValue::I32(..) => 1,
                    Vec2UniformValue::F32(..) => 2,
                };
                VectorUniformValue::show_scalar_selector(
                    ui,
                    group_index,
                    binding_index,
                    &mut message,
                    inner_type_index,
                );
                v.show_editor(ui, group_index, binding_index, &mut message);
            }
            VectorUniformValue::Vec3(v) => {
                UniformValue::show_primitive_selector(
                    ui,
                    group_index,
                    binding_index,
                    &mut message,
                    4,
                    val_name,
                );
                let inner_type_index = match v {
                    Vec3UniformValue::U32(..) => 0,
                    Vec3UniformValue::I32(..) => 1,
                    Vec3UniformValue::F32(..) => 2,
                };
                VectorUniformValue::show_scalar_selector(
                    ui,
                    group_index,
                    binding_index,
                    &mut message,
                    inner_type_index,
                );
                v.show_editor(ui, group_index, binding_index, &mut message);
            }
            VectorUniformValue::Vec4(v) => {
                UniformValue::show_primitive_selector(
                    ui,
                    group_index,
                    binding_index,
                    &mut message,
                    5,
                    val_name,
                );
                let inner_type_index = match v {
                    Vec4UniformValue::U32(..) => 0,
                    Vec4UniformValue::I32(..) => 1,
                    Vec4UniformValue::F32(..) => 2,
                };
                VectorUniformValue::show_scalar_selector(
                    ui,
                    group_index,
                    binding_index,
                    &mut message,
                    inner_type_index,
                );
                v.show_editor(ui, group_index, binding_index, &mut message);
            }
        };
        message
    }

    fn to_le_bytes(&self) -> Vec<u8> {
        match self {
            VectorUniformValue::Vec2(v) => v.to_le_bytes(),
            VectorUniformValue::Vec3(v) => v.to_le_bytes(),
            VectorUniformValue::Vec4(v) => v.to_le_bytes(),
        }
    }
}

impl VectorUniformValue {
    fn show_scalar_selector(
        ui: &Ui,
        group_index: usize,
        binding_index: usize,
        message: &mut Option<UniformEditEvent>,
        type_index: usize,
    ) {
        const TYPES: &[ScalarType] = &[ScalarType::U32, ScalarType::I32, ScalarType::F32];
        const COMBO_WIDTH: f32 = 50.0;

        ui.set_next_item_width(COMBO_WIDTH);
        let mut selection = type_index;
        ui.same_line();
        if ui.combo(
            format!("##scalar_combo_{group_index}_{binding_index}"),
            &mut selection,
            TYPES,
            |unitype| unitype.into(),
        ) {
            let selected_type = TYPES[selection];
            if selected_type != TYPES[type_index] {
                *message = Some(UniformEditEvent::ChangeInnerType(
                    selected_type,
                    group_index,
                    binding_index,
                ))
            }
        };
    }

    pub(crate) fn from_json(uniform: &Map<String, JsonValue>) -> Option<VectorUniformValue> {
        let inner_type = uniform.get("innertype")?;
        match inner_type.as_str()? {
            "vec2" => Some(VectorUniformValue::Vec2(Vec2UniformValue::from_json(uniform)?)),
            "vec3" => Some(VectorUniformValue::Vec3(Vec3UniformValue::from_json(uniform)?)),
            "vec4" => Some(VectorUniformValue::Vec4(Vec4UniformValue::from_json(uniform)?)),
            _ => None
        }
    }

    pub(crate) fn to_json(&self, json_obj: &mut Map<String, JsonValue>) {
        match self {
            VectorUniformValue::Vec2(_) => json_obj.insert("innertype".into(), "vec2".into()),
            VectorUniformValue::Vec3(_) => json_obj.insert("innertype".into(), "vec3".into()),
            VectorUniformValue::Vec4(_) => json_obj.insert("innertype".into(), "vec4".into()),
        };

        match self {
            VectorUniformValue::Vec2(v) => v.to_json(json_obj),
            VectorUniformValue::Vec3(v) => v.to_json(json_obj),
            VectorUniformValue::Vec4(v) => v.to_json(json_obj),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum VecType {
    Vec2(ScalarType),
    Vec3(ScalarType),
    Vec4(ScalarType),
}

impl ImguiVec for VectorUniformValue {
    fn change_inner_type(&mut self, inner_type: ScalarType) {
        match self {
            VectorUniformValue::Vec2(v) => v.change_inner_type(inner_type),
            VectorUniformValue::Vec3(v) => v.change_inner_type(inner_type),
            VectorUniformValue::Vec4(v) => v.change_inner_type(inner_type),
        }
    }
}

impl<'a> From<&'a VecType> for Cow<'static, str> {
    fn from(val: &'a VecType) -> Cow<'static, str> {
        match val {
            VecType::Vec2(_) => Cow::Borrowed("vec2"),
            VecType::Vec3(_) => Cow::Borrowed("vec3"),
            VecType::Vec4(_) => Cow::Borrowed("vec4"),
        }
    }
}
