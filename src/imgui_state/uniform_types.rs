use std::borrow::Cow;

use imgui::Ui;
use mint::{Vector3, Vector4};

use crate::imgui_state::UniformEditEvent;

use super::{ImguiMatrix, ImguiScalar, ImguiUniformSelectable, ImguiVec, DEFAULT_U32_UNIFORM};


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BuiltinValue {
    Time
}
impl BuiltinValue {
    fn to_le_bytes(&self) -> Vec<u8> {
        match self {
            BuiltinValue::Time => 0u32.to_le_bytes().into(),
        }
    }
}

union ScalarPrimitive {
    u32: u32,
    i32: i32,
    f32: f32
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum ScalarUniformValue {
    U32(u32),
    I32(i32),
    F32(f32)
}

impl ImguiScalar for ScalarUniformValue {
    fn decrease(&mut self) {
        match self {
            ScalarUniformValue::U32(v) => *v -= 1,
            ScalarUniformValue::I32(v) => *v -= 1,
            ScalarUniformValue::F32(v) => *v -= 1.0
        }
    }

    fn increase(&mut self) {
        match self {
            ScalarUniformValue::U32(v) => *v += 1,
            ScalarUniformValue::I32(v) => *v += 1,
            ScalarUniformValue::F32(v) => *v += 1.0
        }
    }
}

impl ImguiUniformSelectable for ScalarUniformValue {
    fn cast_to(&self, casted_type: UniformType) -> UniformValue {
        match casted_type {
            UniformType::Scalar(s) => UniformValue::Scalar(self.cast_to_scalar(s)),
            UniformType::Vec(v) => UniformValue::Vector(self.cast_to_vec(v)),
            UniformType::Matrix(m) => UniformValue::Matrix(self.cast_to_matrix(m)),
        }
    }

    fn show_editor(&mut self, ui: &Ui, group_index: usize, binding_index: usize, val_name: &mut String) -> Option<UniformEditEvent> {
        const PRIMITIVE_INPUT_WIDTH: f32 = 50.0;
        let mut message = None;
        match self {
            ScalarUniformValue::U32(v) => {
                UniformValue::show_primitive_selector(ui, group_index, binding_index, &mut message, 0, val_name);
                ui.same_line();
                ui.set_next_item_width(PRIMITIVE_INPUT_WIDTH);
                if ui.input_scalar(format!("##editor{group_index}_{binding_index}"), v).build() {
                    message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index));
                }
                ui.same_line();
                Self::number_edit(ui, group_index, binding_index, &mut message)
            },
            ScalarUniformValue::I32(v) => {
                UniformValue::show_primitive_selector(ui, group_index, binding_index, &mut message, 1, val_name);
                ui.same_line();
                ui.set_next_item_width(PRIMITIVE_INPUT_WIDTH);
                if ui.input_scalar(format!("##editor{group_index}_{binding_index}"), v).build() {
                    message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
                ui.same_line();
                Self::number_edit(ui, group_index, binding_index, &mut message)
            },
            ScalarUniformValue::F32(v) => {
                UniformValue::show_primitive_selector(ui, group_index, binding_index, &mut message, 2, val_name);
                ui.same_line();
                ui.set_next_item_width(PRIMITIVE_INPUT_WIDTH);
                if ui.input_float(format!("##editor{group_index}_{binding_index}"), v).build() {
                    message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
                ui.same_line();
                Self::number_edit(ui, group_index, binding_index, &mut message)
            },
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

            (ScalarUniformValue::I32(v), ScalarType::U32) => ScalarUniformValue::U32(cast_i32_u32(v)),
            (ScalarUniformValue::I32(v), ScalarType::F32) => ScalarUniformValue::F32(v as f32),

            (ScalarUniformValue::F32(v), ScalarType::U32) => ScalarUniformValue::U32(cast_f32_u32(v)),
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
            },
            VecType::Vec3(s) => {
                let v = self.cast_to_scalar(s).to_vec3();
                VectorUniformValue::Vec3(v)
            },
            VecType::Vec4(s) => {
                let v = self.cast_to_scalar(s).to_vec4();
                VectorUniformValue::Vec4(v)
            },
        }
    }

    fn to_vec2(&self) -> Vec2UniformValue {
        match self {
            ScalarUniformValue::U32(s) => Vec2UniformValue::U32(*s, 0),
            ScalarUniformValue::I32(s) => Vec2UniformValue::I32(*s, 0),
            ScalarUniformValue::F32(s) => Vec2UniformValue::F32(*s, 0.0),
        }
    }

    fn to_vec3(&self) -> Vec3UniformValue {
        match self {
            ScalarUniformValue::U32(s) => Vec3UniformValue::U32(*s, 0, 0),
            ScalarUniformValue::I32(s) => Vec3UniformValue::I32(*s, 0, 0),
            ScalarUniformValue::F32(s) => Vec3UniformValue::F32(*s, 0.0, 0.0),
        }
    }

    fn to_vec4(&self) -> Vec4UniformValue {
        match self {
            ScalarUniformValue::U32(s) => Vec4UniformValue::U32(*s, 0, 0, 0),
            ScalarUniformValue::I32(s) => Vec4UniformValue::I32(*s, 0, 0, 0),
            ScalarUniformValue::F32(s) => Vec4UniformValue::F32(*s, 0.0, 0.0, 0.0),
        }
    }


    fn cast_to_matrix(&self, m: MatrixType) -> MatrixUniformValue {
        // TODO: Maybe keep as much information as possible, like with other type casts
        match m {
            MatrixType::M2x2 => MatrixUniformValue::M2x2(Column2(0.0, 0.0), Column2(0.0, 0.0)),
            MatrixType::M2x3 => MatrixUniformValue::M2x3(Column3(0.0, 0.0, 0.0), Column3(0.0, 0.0, 0.0)),
            MatrixType::M2x4 => MatrixUniformValue::M2x4(Column4(0.0, 0.0, 0.0, 0.0), Column4(0.0, 0.0, 0.0, 0.0)),

            MatrixType::M3x2 => MatrixUniformValue::M3x2(Column2(0.0, 0.0), Column2(0.0, 0.0), Column2(0.0, 0.0)),
            MatrixType::M3x3 => MatrixUniformValue::M3x3(Column3(0.0, 0.0, 0.0), Column3(0.0, 0.0, 0.0), Column3(0.0, 0.0, 0.0)),
            MatrixType::M3x4 => MatrixUniformValue::M3x4(Column4(0.0, 0.0, 0.0, 0.0), Column4(0.0, 0.0, 0.0, 0.0), Column4(0.0, 0.0, 0.0, 0.0)),

            MatrixType::M4x2 => MatrixUniformValue::M4x2(Column2(0.0, 0.0), Column2(0.0, 0.0), Column2(0.0, 0.0), Column2(0.0, 0.0)),
            MatrixType::M4x3 => MatrixUniformValue::M4x3(Column3(0.0, 0.0, 0.0), Column3(0.0, 0.0, 0.0), Column3(0.0, 0.0, 0.0), Column3(0.0, 0.0, 0.0)),
            MatrixType::M4x4 => MatrixUniformValue::M4x4(Column4(0.0, 0.0, 0.0, 0.0), Column4(0.0, 0.0, 0.0, 0.0), Column4(0.0, 0.0, 0.0, 0.0), Column4(0.0, 0.0, 0.0, 0.0)),
        }
    }

    fn number_edit(ui: &Ui, group_index: usize, binding_index: usize, message: &mut Option<UniformEditEvent>) {
        if ui.button(format!("+##add_{group_index}_{binding_index}")) {
            *message = Some(UniformEditEvent::Increase(group_index, binding_index))
        }
        ui.same_line();
        if ui.button(format!("-##decrease_{group_index}_{binding_index}")) {
            *message = Some(UniformEditEvent::Decrease(group_index, binding_index))
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

trait VecUniformValue {
    fn show_editor(&mut self, ui: &Ui, group_index: usize, binding_index: usize, message: &mut Option<UniformEditEvent>);
    fn change_inner_type(&mut self, inner_type: ScalarType);
    fn to_le_bytes(&self) -> Vec<u8>;
    fn cast_to_scalar(&self, s: ScalarType) -> UniformValue;
    fn cast_to_vec(&self, v: VecType) -> UniformValue;
    fn cast_to_matrix(&self, m: MatrixType) -> UniformValue;
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Vec2UniformValue {
    U32(u32, u32),
    I32(i32, i32),
    F32(f32, f32),
}

impl VecUniformValue for Vec2UniformValue {
    fn show_editor(&mut self, ui: &Ui, group_index: usize, binding_index: usize, message: &mut Option<UniformEditEvent>) {
        match self {
            Vec2UniformValue::U32(x, y) => {
                let mut vars = [*x, *y];
                if ui.input_scalar_n(format!("##v2edit_{group_index}_{binding_index}"), &mut vars).build() {
                    *x = vars[0];
                    *y = vars[1];
                    *message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
            },
            Vec2UniformValue::I32(x, y) => {
                let mut vars = [*x, *y];
                if ui.input_scalar_n(format!("##v2edit_{group_index}_{binding_index}"), &mut vars).build() {
                    *x = vars[0];
                    *y = vars[1];
                    *message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
            },
            Vec2UniformValue::F32(x, y) => {
                let mut vars = [*x, *y];
                if ui.input_scalar_n(format!("##v2edit_{group_index}_{binding_index}"), &mut vars).build() {
                    *x = vars[0];
                    *y = vars[1];
                    *message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
            },
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
                ScalarType::U32 => *self = Vec2UniformValue::U32(cast_i32_u32(*x), cast_i32_u32(*y)),
                ScalarType::I32 => (),
                ScalarType::F32 => *self = Vec2UniformValue::F32(*x as f32, *y as f32),
            },
            Vec2UniformValue::F32(x, y) => match inner_type {
                ScalarType::U32 => *self = Vec2UniformValue::U32(cast_f32_u32(*x), cast_f32_u32(*y)),
                ScalarType::I32 => *self = Vec2UniformValue::I32(*x as i32, *y as i32),
                ScalarType::F32 => (),
            },
        }
    }

    fn to_le_bytes(&self) -> Vec<u8> {
        match self {
            Vec2UniformValue::U32(x, y) => x.to_le_bytes().into_iter().chain(y.to_le_bytes().into_iter()).collect(),
            Vec2UniformValue::I32(x, y) => x.to_le_bytes().into_iter().chain(y.to_le_bytes().into_iter()).collect(),
            Vec2UniformValue::F32(x, y) => x.to_le_bytes().into_iter().chain(y.to_le_bytes().into_iter()).collect(),
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
                ScalarType::U32 => [SP{u32: *x}, SP{u32: *y}, SP{u32: 0}, SP{u32: 0}],
                ScalarType::I32 => [SP{i32: *x as i32}, SP{i32: *y as i32}, SP{i32: 0}, SP{i32: 0}],
                ScalarType::F32 => [SP{f32: *x as f32}, SP{f32: *y as f32}, SP{f32: 0.0}, SP{f32: 0.0}],
            },
            Vec2UniformValue::I32(x, y) => match scalar_type {
                ScalarType::U32 => [SP{u32: cast_i32_u32(*x)}, SP{u32: cast_i32_u32(*y)}, SP{u32: 0}, SP{u32: 0}],
                ScalarType::I32 => [SP{i32: *x}, SP{i32: *y}, SP{i32: 0}, SP{i32: 0}],
                ScalarType::F32 => [SP{f32: *x as f32}, SP{f32: *y as f32}, SP{f32: 0.0}, SP{f32: 0.0}],
            },
            Vec2UniformValue::F32(x, y) => match scalar_type {
                ScalarType::U32 => [SP{u32: cast_f32_u32(*x)}, SP{u32: cast_f32_u32(*y)}, SP{u32: 0}, SP{u32: 0}],
                ScalarType::I32 => [SP{i32: *x as i32}, SP{i32: *y as i32}, SP{i32: 0}, SP{i32: 0}],
                ScalarType::F32 => [SP{f32: *x}, SP{f32: *y}, SP{f32: 0.0}, SP{f32: 0.0}],
            },
        };

        unsafe {
            UniformValue::Vector(match v {
                VecType::Vec2(scalar_type) => match scalar_type {
                    ScalarType::U32 => VectorUniformValue::Vec2(Vec2UniformValue::U32(vec4[0].u32, vec4[1].u32)),
                    ScalarType::I32 => VectorUniformValue::Vec2(Vec2UniformValue::I32(vec4[0].i32, vec4[1].i32)),
                    ScalarType::F32 => VectorUniformValue::Vec2(Vec2UniformValue::F32(vec4[0].f32, vec4[1].f32)),
                },
                VecType::Vec3(scalar_type) => match scalar_type {
                    ScalarType::U32 => VectorUniformValue::Vec3(Vec3UniformValue::U32(vec4[0].u32, vec4[1].u32, vec4[0].u32)),
                    ScalarType::I32 => VectorUniformValue::Vec3(Vec3UniformValue::I32(vec4[0].i32, vec4[1].i32, vec4[0].i32)),
                    ScalarType::F32 => VectorUniformValue::Vec3(Vec3UniformValue::F32(vec4[0].f32, vec4[1].f32, vec4[0].f32)),
                },
                VecType::Vec4(scalar_type) => match scalar_type {
                    ScalarType::U32 => VectorUniformValue::Vec4(Vec4UniformValue::U32(vec4[0].u32, vec4[1].u32, vec4[0].u32, vec4[1].u32)),
                    ScalarType::I32 => VectorUniformValue::Vec4(Vec4UniformValue::I32(vec4[0].i32, vec4[1].i32, vec4[0].i32, vec4[1].i32)),
                    ScalarType::F32 => VectorUniformValue::Vec4(Vec4UniformValue::F32(vec4[0].f32, vec4[1].f32, vec4[0].f32, vec4[1].f32)),
                },
            })
        }
    }

    fn cast_to_matrix(&self, m: MatrixType) -> UniformValue {
        // TODO: Maybe keep as much information as possible, like with other types
        UniformValue::Matrix(match m {
            MatrixType::M2x2 => MatrixUniformValue::M2x2(Column2(0.0, 0.0), Column2(0.0, 0.0)),
            MatrixType::M2x3 => MatrixUniformValue::M2x3(Column3(0.0, 0.0, 0.0), Column3(0.0, 0.0, 0.0)),
            MatrixType::M2x4 => MatrixUniformValue::M2x4(Column4(0.0, 0.0, 0.0, 0.0), Column4(0.0, 0.0, 0.0, 0.0)),

            MatrixType::M3x2 => MatrixUniformValue::M3x2(Column2(0.0, 0.0), Column2(0.0, 0.0), Column2(0.0, 0.0)),
            MatrixType::M3x3 => MatrixUniformValue::M3x3(Column3(0.0, 0.0, 0.0), Column3(0.0, 0.0, 0.0), Column3(0.0, 0.0, 0.0)),
            MatrixType::M3x4 => MatrixUniformValue::M3x4(Column4(0.0, 0.0, 0.0, 0.0), Column4(0.0, 0.0, 0.0, 0.0), Column4(0.0, 0.0, 0.0, 0.0)),

            MatrixType::M4x2 => MatrixUniformValue::M4x2(Column2(0.0, 0.0), Column2(0.0, 0.0), Column2(0.0, 0.0), Column2(0.0, 0.0)),
            MatrixType::M4x3 => MatrixUniformValue::M4x3(Column3(0.0, 0.0, 0.0), Column3(0.0, 0.0, 0.0), Column3(0.0, 0.0, 0.0), Column3(0.0, 0.0, 0.0)),
            MatrixType::M4x4 => MatrixUniformValue::M4x4(Column4(0.0, 0.0, 0.0, 0.0), Column4(0.0, 0.0, 0.0, 0.0), Column4(0.0, 0.0, 0.0, 0.0), Column4(0.0, 0.0, 0.0, 0.0)),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Vec3UniformValue {
    U32(u32, u32, u32),
    I32(i32, i32, i32),
    F32(f32, f32, f32),
}

impl VecUniformValue for Vec3UniformValue {
    fn show_editor(&mut self, ui: &Ui, group_index: usize, binding_index: usize, message: &mut Option<UniformEditEvent>) {
        match self {
            Vec3UniformValue::U32(x, y, z) => {
                let mut vars = [*x, *y, *z];
                if ui.input_scalar_n(format!("##v3edit_{group_index}_{binding_index}"), &mut vars).build() {
                    *x = vars[0];
                    *y = vars[1];
                    *z = vars[2];
                    *message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
            },
            Vec3UniformValue::I32(x, y, z) => {
                let mut vars = [*x, *y, *z];
                if ui.input_scalar_n(format!("##v3edit_{group_index}_{binding_index}"), &mut vars).build() {
                    *x = vars[0];
                    *y = vars[1];
                    *z = vars[2];
                    *message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
            },
            Vec3UniformValue::F32(x, y, z) => {
                let mut c_vars = Vector3 {
                    x: *x,
                    y: *y,
                    z: *z,
                };
                let mut vars = [*x, *y, *z];
                if ui.input_scalar_n(format!("##v3edit_{group_index}_{binding_index}"), &mut vars).build() {
                    *x = vars[0];
                    *y = vars[1];
                    *z = vars[2];
                    *message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
                if ui.color_edit3(format!("##v3edit_{group_index}_{binding_index}"), &mut c_vars) {
                    *x = c_vars.x;
                    *y = c_vars.y;
                    *z = c_vars.z;
                    *message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
            },
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
                ScalarType::U32 => *self = Vec3UniformValue::U32(cast_i32_u32(*x), cast_i32_u32(*y), cast_i32_u32(*z)),
                ScalarType::I32 => (),
                ScalarType::F32 => *self = Vec3UniformValue::F32(*x as f32, *y as f32, *z as f32),
            },
            Vec3UniformValue::F32(x, y, z) => match inner_type {
                ScalarType::U32 => *self = Vec3UniformValue::U32(cast_f32_u32(*x), cast_f32_u32(*y), cast_f32_u32(*z)),
                ScalarType::I32 => *self = Vec3UniformValue::I32(*x as i32, *y as i32, *z as i32),
                ScalarType::F32 => (),
            },
        }
    }

    fn to_le_bytes(&self) -> Vec<u8> {
        match self {
            Vec3UniformValue::U32(x, y, z) => x.to_le_bytes().into_iter().chain(y.to_le_bytes().into_iter().chain(z.to_le_bytes().into_iter())).collect(),
            Vec3UniformValue::I32(x, y, z) => x.to_le_bytes().into_iter().chain(y.to_le_bytes().into_iter().chain(z.to_le_bytes().into_iter())).collect(),
            Vec3UniformValue::F32(x, y, z) => x.to_le_bytes().into_iter().chain(y.to_le_bytes().into_iter().chain(z.to_le_bytes().into_iter())).collect(),
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
                ScalarType::U32 => [SP{u32: *x}, SP{u32: *y}, SP{u32: *z}, SP{u32: 0}],
                ScalarType::I32 => [SP{i32: *x as i32}, SP{i32: *y as i32}, SP{i32: *z as i32}, SP{i32: 0}],
                ScalarType::F32 => [SP{f32: *x as f32}, SP{f32: *y as f32}, SP{f32: *z as f32}, SP{f32: 0.0}],
            },
            Vec3UniformValue::I32(x, y, z) => match scalar_type {
                ScalarType::U32 => [SP{u32: cast_i32_u32(*x)}, SP{u32: cast_i32_u32(*y)}, SP{u32: cast_i32_u32(*z)}, SP{u32: 0}],
                ScalarType::I32 => [SP{i32: *x}, SP{i32: *y}, SP{i32: *z}, SP{i32: *z}],
                ScalarType::F32 => [SP{f32: *x as f32}, SP{f32: *y as f32}, SP{f32: *z as f32}, SP{f32: 0.0}],
            },
            Vec3UniformValue::F32(x, y, z) => match scalar_type {
                ScalarType::U32 => [SP{u32: cast_f32_u32(*x)}, SP{u32: cast_f32_u32(*y)}, SP{u32: cast_f32_u32(*z)}, SP{u32: 0}],
                ScalarType::I32 => [SP{i32: *x as i32}, SP{i32: *y as i32}, SP{i32: *z as i32}, SP{i32: 0}],
                ScalarType::F32 => [SP{f32: *x}, SP{f32: *y}, SP{f32: *z}, SP{f32: 0.0}],
            },
        };

        unsafe {
            UniformValue::Vector(match v {
                VecType::Vec2(scalar_type) => match scalar_type {
                    ScalarType::U32 => VectorUniformValue::Vec2(Vec2UniformValue::U32(vec4[0].u32, vec4[1].u32)),
                    ScalarType::I32 => VectorUniformValue::Vec2(Vec2UniformValue::I32(vec4[0].i32, vec4[1].i32)),
                    ScalarType::F32 => VectorUniformValue::Vec2(Vec2UniformValue::F32(vec4[0].f32, vec4[1].f32)),
                },
                VecType::Vec3(scalar_type) => match scalar_type {
                    ScalarType::U32 => VectorUniformValue::Vec3(Vec3UniformValue::U32(vec4[0].u32, vec4[1].u32, vec4[0].u32)),
                    ScalarType::I32 => VectorUniformValue::Vec3(Vec3UniformValue::I32(vec4[0].i32, vec4[1].i32, vec4[0].i32)),
                    ScalarType::F32 => VectorUniformValue::Vec3(Vec3UniformValue::F32(vec4[0].f32, vec4[1].f32, vec4[0].f32)),
                },
                VecType::Vec4(scalar_type) => match scalar_type {
                    ScalarType::U32 => VectorUniformValue::Vec4(Vec4UniformValue::U32(vec4[0].u32, vec4[1].u32, vec4[0].u32, vec4[1].u32)),
                    ScalarType::I32 => VectorUniformValue::Vec4(Vec4UniformValue::I32(vec4[0].i32, vec4[1].i32, vec4[0].i32, vec4[1].i32)),
                    ScalarType::F32 => VectorUniformValue::Vec4(Vec4UniformValue::F32(vec4[0].f32, vec4[1].f32, vec4[0].f32, vec4[1].f32)),
                },
            })
        }
    }

    fn cast_to_matrix(&self, m: MatrixType) -> UniformValue {
        // TODO: Maybe keep as much information as possible, like with other types
        UniformValue::Matrix(match m {
            MatrixType::M2x2 => MatrixUniformValue::M2x2(Column2(0.0, 0.0), Column2(0.0, 0.0)),
            MatrixType::M2x3 => MatrixUniformValue::M2x3(Column3(0.0, 0.0, 0.0), Column3(0.0, 0.0, 0.0)),
            MatrixType::M2x4 => MatrixUniformValue::M2x4(Column4(0.0, 0.0, 0.0, 0.0), Column4(0.0, 0.0, 0.0, 0.0)),

            MatrixType::M3x2 => MatrixUniformValue::M3x2(Column2(0.0, 0.0), Column2(0.0, 0.0), Column2(0.0, 0.0)),
            MatrixType::M3x3 => MatrixUniformValue::M3x3(Column3(0.0, 0.0, 0.0), Column3(0.0, 0.0, 0.0), Column3(0.0, 0.0, 0.0)),
            MatrixType::M3x4 => MatrixUniformValue::M3x4(Column4(0.0, 0.0, 0.0, 0.0), Column4(0.0, 0.0, 0.0, 0.0), Column4(0.0, 0.0, 0.0, 0.0)),

            MatrixType::M4x2 => MatrixUniformValue::M4x2(Column2(0.0, 0.0), Column2(0.0, 0.0), Column2(0.0, 0.0), Column2(0.0, 0.0)),
            MatrixType::M4x3 => MatrixUniformValue::M4x3(Column3(0.0, 0.0, 0.0), Column3(0.0, 0.0, 0.0), Column3(0.0, 0.0, 0.0), Column3(0.0, 0.0, 0.0)),
            MatrixType::M4x4 => MatrixUniformValue::M4x4(Column4(0.0, 0.0, 0.0, 0.0), Column4(0.0, 0.0, 0.0, 0.0), Column4(0.0, 0.0, 0.0, 0.0), Column4(0.0, 0.0, 0.0, 0.0)),
        })
    }
}


#[derive(Debug, Clone, Copy, PartialEq)]
enum Vec4UniformValue {
    U32(u32, u32, u32, u32),
    I32(i32, i32, i32, i32),
    F32(f32, f32, f32, f32),
}

impl VecUniformValue for Vec4UniformValue {
    fn show_editor(&mut self, ui: &Ui, group_index: usize, binding_index: usize, message: &mut Option<UniformEditEvent>) {
        match self {
            Vec4UniformValue::U32(x, y, z, w) => {
                let mut vars = [*x, *y, *z, *w];
                if ui.input_scalar_n(format!("##v4edit_{group_index}_{binding_index}"), &mut vars).build() {
                    *x = vars[0];
                    *y = vars[1];
                    *z = vars[2];
                    *w = vars[3];
                    *message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
            },
            Vec4UniformValue::I32(x, y, z, w) => {
                let mut vars = [*x, *y, *z, *w];
                if ui.input_scalar_n(format!("##v4edit_{group_index}_{binding_index}"), &mut vars).build() {
                    *x = vars[0];
                    *y = vars[1];
                    *z = vars[2];
                    *w = vars[3];
                    *message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
            },
            Vec4UniformValue::F32(x, y, z, w) => {
                let mut c_vars = Vector4 {
                    x: *x,
                    y: *y,
                    z: *z,
                    w: *w
                };
                let mut vars = [*x, *y, *z, *w];
                if ui.input_scalar_n(format!("##v4edit_{group_index}_{binding_index}"), &mut vars).build() {
                    *x = vars[0];
                    *y = vars[1];
                    *z = vars[2];
                    *w = vars[3];
                    *message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
                if ui.color_edit4(format!("##v4edit_{group_index}_{binding_index}"), &mut c_vars) {
                    *x = c_vars.x;
                    *y = c_vars.y;
                    *z = c_vars.z;
                    *w = c_vars.w;
                    *message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
            },
        }
    }

    fn change_inner_type(&mut self, inner_type: ScalarType) {
        match self {
            Vec4UniformValue::U32(x, y, z, w) => match inner_type {
                ScalarType::U32 => (),
                ScalarType::I32 => *self = Vec4UniformValue::I32(*x as i32, *y as i32, *z as i32, *w as i32),
                ScalarType::F32 => *self = Vec4UniformValue::F32(*x as f32, *y as f32, *z as f32, *w as f32),
            },
            Vec4UniformValue::I32(x, y, z, w) => match inner_type {
                ScalarType::U32 => *self = Vec4UniformValue::U32(cast_i32_u32(*x), cast_i32_u32(*y), cast_i32_u32(*z), cast_i32_u32(*w)),
                ScalarType::I32 => (),
                ScalarType::F32 => *self = Vec4UniformValue::F32(*x as f32, *y as f32, *z as f32, *w as f32),
            },
            Vec4UniformValue::F32(x, y, z, w) => match inner_type {
                ScalarType::U32 => *self = Vec4UniformValue::U32(cast_f32_u32(*x), cast_f32_u32(*y), cast_f32_u32(*z), cast_f32_u32(*w)),
                ScalarType::I32 => *self = Vec4UniformValue::I32(*x as i32, *y as i32, *z as i32, *w as i32),
                ScalarType::F32 => (),
            },
        }
    }

    fn to_le_bytes(&self) -> Vec<u8> {
        match self {
            Vec4UniformValue::U32(x, y, z, w) => x.to_le_bytes().into_iter().chain(y.to_le_bytes().into_iter().chain(z.to_le_bytes().into_iter()).chain(w.to_le_bytes().into_iter())).collect(),
            Vec4UniformValue::I32(x, y, z, w) => x.to_le_bytes().into_iter().chain(y.to_le_bytes().into_iter().chain(z.to_le_bytes().into_iter()).chain(w.to_le_bytes().into_iter())).collect(),
            Vec4UniformValue::F32(x, y, z, w) => x.to_le_bytes().into_iter().chain(y.to_le_bytes().into_iter().chain(z.to_le_bytes().into_iter()).chain(w.to_le_bytes().into_iter())).collect(),
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
                ScalarType::U32 => [SP{u32: *x}, SP{u32: *y}, SP{u32: *z}, SP{u32: *w}],
                ScalarType::I32 => [SP{i32: *x as i32}, SP{i32: *y as i32}, SP{i32: *z as i32}, SP{i32: *w as i32}],
                ScalarType::F32 => [SP{f32: *x as f32}, SP{f32: *y as f32}, SP{f32: *z as f32}, SP{f32: *w as f32}],
            },
            Vec4UniformValue::I32(x, y, z, w) => match scalar_type {
                ScalarType::U32 => [SP{u32: cast_i32_u32(*x)}, SP{u32: cast_i32_u32(*y)}, SP{u32: cast_i32_u32(*z)}, SP{u32: cast_i32_u32(*w)}],
                ScalarType::I32 => [SP{i32: *x}, SP{i32: *y}, SP{i32: *z}, SP{i32: *w}],
                ScalarType::F32 => [SP{f32: *x as f32}, SP{f32: *y as f32}, SP{f32: *z as f32}, SP{f32: *w as f32}],
            },
            Vec4UniformValue::F32(x, y, z, w) => match scalar_type {
                ScalarType::U32 => [SP{u32: cast_f32_u32(*x)}, SP{u32: cast_f32_u32(*y)}, SP{u32: cast_f32_u32(*z)}, SP{u32: cast_f32_u32(*w)}],
                ScalarType::I32 => [SP{i32: *x as i32}, SP{i32: *y as i32}, SP{i32: *z as i32}, SP{i32: *w as i32}],
                ScalarType::F32 => [SP{f32: *x}, SP{f32: *y}, SP{f32: *z}, SP{f32: *w as f32}],
            },
        };

        unsafe {
            UniformValue::Vector(match v {
                VecType::Vec2(scalar_type) => match scalar_type {
                    ScalarType::U32 => VectorUniformValue::Vec2(Vec2UniformValue::U32(vec4[0].u32, vec4[1].u32)),
                    ScalarType::I32 => VectorUniformValue::Vec2(Vec2UniformValue::I32(vec4[0].i32, vec4[1].i32)),
                    ScalarType::F32 => VectorUniformValue::Vec2(Vec2UniformValue::F32(vec4[0].f32, vec4[1].f32)),
                },
                VecType::Vec3(scalar_type) => match scalar_type {
                    ScalarType::U32 => VectorUniformValue::Vec3(Vec3UniformValue::U32(vec4[0].u32, vec4[1].u32, vec4[0].u32)),
                    ScalarType::I32 => VectorUniformValue::Vec3(Vec3UniformValue::I32(vec4[0].i32, vec4[1].i32, vec4[0].i32)),
                    ScalarType::F32 => VectorUniformValue::Vec3(Vec3UniformValue::F32(vec4[0].f32, vec4[1].f32, vec4[0].f32)),
                },
                VecType::Vec4(scalar_type) => match scalar_type {
                    ScalarType::U32 => VectorUniformValue::Vec4(Vec4UniformValue::U32(vec4[0].u32, vec4[1].u32, vec4[0].u32, vec4[1].u32)),
                    ScalarType::I32 => VectorUniformValue::Vec4(Vec4UniformValue::I32(vec4[0].i32, vec4[1].i32, vec4[0].i32, vec4[1].i32)),
                    ScalarType::F32 => VectorUniformValue::Vec4(Vec4UniformValue::F32(vec4[0].f32, vec4[1].f32, vec4[0].f32, vec4[1].f32)),
                },
            })
        }
    }

    fn cast_to_matrix(&self, m: MatrixType) -> UniformValue {
        // TODO: Maybe keep as much information as possible, like with other types
        UniformValue::Matrix(match m {
            MatrixType::M2x2 => MatrixUniformValue::M2x2(Column2(0.0, 0.0), Column2(0.0, 0.0)),
            MatrixType::M2x3 => MatrixUniformValue::M2x3(Column3(0.0, 0.0, 0.0), Column3(0.0, 0.0, 0.0)),
            MatrixType::M2x4 => MatrixUniformValue::M2x4(Column4(0.0, 0.0, 0.0, 0.0), Column4(0.0, 0.0, 0.0, 0.0)),

            MatrixType::M3x2 => MatrixUniformValue::M3x2(Column2(0.0, 0.0), Column2(0.0, 0.0), Column2(0.0, 0.0)),
            MatrixType::M3x3 => MatrixUniformValue::M3x3(Column3(0.0, 0.0, 0.0), Column3(0.0, 0.0, 0.0), Column3(0.0, 0.0, 0.0)),
            MatrixType::M3x4 => MatrixUniformValue::M3x4(Column4(0.0, 0.0, 0.0, 0.0), Column4(0.0, 0.0, 0.0, 0.0), Column4(0.0, 0.0, 0.0, 0.0)),

            MatrixType::M4x2 => MatrixUniformValue::M4x2(Column2(0.0, 0.0), Column2(0.0, 0.0), Column2(0.0, 0.0), Column2(0.0, 0.0)),
            MatrixType::M4x3 => MatrixUniformValue::M4x3(Column3(0.0, 0.0, 0.0), Column3(0.0, 0.0, 0.0), Column3(0.0, 0.0, 0.0), Column3(0.0, 0.0, 0.0)),
            MatrixType::M4x4 => MatrixUniformValue::M4x4(Column4(0.0, 0.0, 0.0, 0.0), Column4(0.0, 0.0, 0.0, 0.0), Column4(0.0, 0.0, 0.0, 0.0), Column4(0.0, 0.0, 0.0, 0.0)),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum VectorUniformValue {
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
        }
    }

    fn show_editor(&mut self, ui: &Ui, group_index: usize, binding_index: usize, val_name: &mut String) -> Option<UniformEditEvent> {
        let mut message = None;
        match self {
            VectorUniformValue::Vec2(v) => {
                UniformValue::show_primitive_selector(ui, group_index, binding_index, &mut message, 3, val_name);
                let inner_type_index = match v {
                    Vec2UniformValue::U32(..) => 0,
                    Vec2UniformValue::I32(..) => 1,
                    Vec2UniformValue::F32(..) => 2,
                };
                VectorUniformValue::show_scalar_selector(ui, group_index, binding_index, &mut message, inner_type_index);
                v.show_editor(ui, group_index, binding_index, &mut message);
            },
            VectorUniformValue::Vec3(v) => {
                UniformValue::show_primitive_selector(ui, group_index, binding_index, &mut message, 4, val_name);
                let inner_type_index = match v {
                    Vec3UniformValue::U32(..) => 0,
                    Vec3UniformValue::I32(..) => 1,
                    Vec3UniformValue::F32(..) => 2,
                };
                VectorUniformValue::show_scalar_selector(ui, group_index, binding_index, &mut message, inner_type_index);
                v.show_editor(ui, group_index, binding_index, &mut message);
            },
            VectorUniformValue::Vec4(v) => {
                UniformValue::show_primitive_selector(ui, group_index, binding_index, &mut message, 5, val_name);
                let inner_type_index = match v {
                    Vec4UniformValue::U32(..) => 0,
                    Vec4UniformValue::I32(..) => 1,
                    Vec4UniformValue::F32(..) => 2,
                };
                VectorUniformValue::show_scalar_selector(ui, group_index, binding_index, &mut message, inner_type_index);
                v.show_editor(ui, group_index, binding_index, &mut message);
            },
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
    fn show_scalar_selector(ui: &Ui, group_index: usize, binding_index: usize, message: &mut Option<UniformEditEvent>, type_index: usize) {
        const TYPES: &[ScalarType] = &[
            ScalarType::U32,
            ScalarType::I32,
            ScalarType::F32,

        ];
        const COMBO_WIDTH: f32 = 50.0;

        ui.set_next_item_width(COMBO_WIDTH);
        let mut selection = type_index;
        ui.same_line();
        if ui.combo(
            format!("##scalar_combo_{group_index}_{binding_index}"),
            &mut selection,
            TYPES,
            |unitype| unitype.into()
        ) {
            let selected_type = TYPES[selection];
            if selected_type != TYPES[type_index] {
                *message = Some(UniformEditEvent::ChangeInnerType(selected_type, group_index, binding_index))
            }
        };
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum UniformValue {
    BuiltIn(BuiltinValue),
    Scalar(ScalarUniformValue),
    Vector(VectorUniformValue),
    Matrix(MatrixUniformValue),
}

trait MatrixColumn {
    fn to_le_bytes(&self) -> Vec<u8>;
    fn values(&self) -> Vec<f32>;
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Column2(f32, f32);
#[derive(Debug, Clone, Copy, PartialEq)]
struct Column3(f32, f32, f32);
#[derive(Debug, Clone, Copy, PartialEq)]
struct Column4(f32, f32, f32, f32);

impl MatrixColumn for Column2 {
    fn to_le_bytes(&self) -> Vec<u8> {
        self.0.to_le_bytes().into_iter().chain(
            self.1.to_le_bytes().into_iter()
        ).collect()
    }

    fn values(&self) -> Vec<f32> {
        vec![
            self.0,
            self.1,
        ]
    }
}

impl MatrixColumn for Column3 {
    fn to_le_bytes(&self) -> Vec<u8> {
        self.0.to_le_bytes().into_iter().chain(
            self.1.to_le_bytes().into_iter().chain(
                self.2.to_le_bytes().into_iter()
            )
        ).collect()
    }

    fn values(&self) -> Vec<f32> {
        vec![
            self.0,
            self.1,
            self.2,
        ]
    }
}

impl MatrixColumn for Column4 {
    fn to_le_bytes(&self) -> Vec<u8> {
        self.0.to_le_bytes().into_iter().chain(
            self.1.to_le_bytes().into_iter().chain(
                self.2.to_le_bytes().into_iter().chain(
                    self.3.to_le_bytes().into_iter()
                )
            )
        ).collect()
    }

    fn values(&self) -> Vec<f32> {
        vec![
            self.0,
            self.1,
            self.2,
            self.3
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum MatrixUniformValue {
    // Column x Row
    M2x2(Column2, Column2),
    M2x3(Column3, Column3),
    M2x4(Column4, Column4),

    M3x2(Column2, Column2, Column2),
    M3x3(Column3, Column3, Column3),
    M3x4(Column4, Column4, Column4),

    M4x2(Column2, Column2, Column2, Column2),
    M4x3(Column3, Column3, Column3, Column3),
    M4x4(Column4, Column4, Column4, Column4),
}
impl MatrixUniformValue {
    fn cast_to_scalar(&self, s: ScalarType) -> UniformValue {
        UniformValue::Scalar(match s {
            ScalarType::U32 => ScalarUniformValue::U32(0),
            ScalarType::I32 => ScalarUniformValue::I32(0),
            ScalarType::F32 => ScalarUniformValue::F32(0.0),
        })
    }

    fn cast_to_vec(&self, v: VecType) -> UniformValue {
        UniformValue::Vector(match v {
            VecType::Vec2(s) => match s {
                ScalarType::U32 => VectorUniformValue::Vec2(Vec2UniformValue::U32(0, 0)),
                ScalarType::I32 => VectorUniformValue::Vec2(Vec2UniformValue::I32(0, 0)),
                ScalarType::F32 => VectorUniformValue::Vec2(Vec2UniformValue::F32(0.0, 0.0)),
            },
            VecType::Vec3(s) => match s {
                ScalarType::U32 => VectorUniformValue::Vec3(Vec3UniformValue::U32(0, 0, 0)),
                ScalarType::I32 => VectorUniformValue::Vec3(Vec3UniformValue::I32(0, 0, 0)),
                ScalarType::F32 => VectorUniformValue::Vec3(Vec3UniformValue::F32(0.0, 0.0, 0.0)),
            },
            VecType::Vec4(s) => match s {
                ScalarType::U32 => VectorUniformValue::Vec4(Vec4UniformValue::U32(0, 0, 0, 0)),
                ScalarType::I32 => VectorUniformValue::Vec4(Vec4UniformValue::I32(0, 0, 0, 0)),
                ScalarType::F32 => VectorUniformValue::Vec4(Vec4UniformValue::F32(0.0, 0.0, 0.0, 0.0)),
            },
        })
    }

    fn cast_to_matrix(&self, m: MatrixType) -> UniformValue {
        UniformValue::Matrix(match m {
            MatrixType::M2x2 => MatrixUniformValue::M2x2(Column2(0.0, 0.0), Column2(0.0, 0.0)),
            MatrixType::M2x3 => MatrixUniformValue::M2x3(Column3(0.0, 0.0, 0.0), Column3(0.0, 0.0, 0.0)),
            MatrixType::M2x4 => MatrixUniformValue::M2x4(Column4(0.0, 0.0, 0.0, 0.0), Column4(0.0, 0.0, 0.0, 0.0)),

            MatrixType::M3x2 => MatrixUniformValue::M3x2(Column2(0.0, 0.0), Column2(0.0, 0.0), Column2(0.0, 0.0)),
            MatrixType::M3x3 => MatrixUniformValue::M3x3(Column3(0.0, 0.0, 0.0), Column3(0.0, 0.0, 0.0), Column3(0.0, 0.0, 0.0)),
            MatrixType::M3x4 => MatrixUniformValue::M3x4(Column4(0.0, 0.0, 0.0, 0.0), Column4(0.0, 0.0, 0.0, 0.0), Column4(0.0, 0.0, 0.0, 0.0)),

            MatrixType::M4x2 => MatrixUniformValue::M4x2(Column2(0.0, 0.0), Column2(0.0, 0.0), Column2(0.0, 0.0), Column2(0.0, 0.0)),
            MatrixType::M4x3 => MatrixUniformValue::M4x3(Column3(0.0, 0.0, 0.0), Column3(0.0, 0.0, 0.0), Column3(0.0, 0.0, 0.0), Column3(0.0, 0.0, 0.0)),
            MatrixType::M4x4 => MatrixUniformValue::M4x4(Column4(0.0, 0.0, 0.0, 0.0), Column4(0.0, 0.0, 0.0, 0.0), Column4(0.0, 0.0, 0.0, 0.0), Column4(0.0, 0.0, 0.0, 0.0)),
        })
    }

    fn show_size_selector(ui: &Ui, group_index: usize, binding_index: usize, size_index: usize, message: &mut Option<UniformEditEvent>) {
        const MATRIX_SIZES: &[MatrixType] = &[
            MatrixType::M2x2,
            MatrixType::M3x2,
            MatrixType::M4x2,
            MatrixType::M2x3,
            MatrixType::M3x3,
            MatrixType::M4x3,
            MatrixType::M2x4,
            MatrixType::M3x4,
            MatrixType::M4x4,
        ];

        const COMBO_WIDTH: f32 = 150.0;

        ui.set_next_item_width(COMBO_WIDTH);
        let mut selection = size_index;
        if ui.matrix_combo(
            format!("##matrix_combo_g{group_index}_b{binding_index}"),
            &mut selection,
            MATRIX_SIZES,
            |unitype| unitype.into(),
            3
        ) {
            let selected_type = MATRIX_SIZES[selection];
            if selected_type != MATRIX_SIZES[size_index] {
                *message = Some(UniformEditEvent::ChangeMatrixSize(selected_type, group_index, binding_index))
            }
        };
    }
}

trait ExtendedUi {
    fn matrix_combo<V, L>(&self, label: impl AsRef<str>, current_item: &mut usize, items: &[V], label_fn: L, column_amount: i32) -> bool
    where
        for<'b> L: Fn(&'b V) -> Cow<'b, str>;

}

impl ExtendedUi for Ui {
    fn matrix_combo<V, L>(&self, label: impl AsRef<str>, current_item: &mut usize, items: &[V], label_fn: L, column_amount: i32) -> bool
    where
        for<'b> L: Fn(&'b V) -> Cow<'b, str>
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
                let clicked = self.selectable_config(cur.clone())
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

impl ImguiUniformSelectable for MatrixUniformValue {
    fn cast_to(&self, casted_type: UniformType) -> UniformValue {
        // TODO: Do like other types and keep as much data as possible
        match casted_type {
            UniformType::Scalar(s) => self.cast_to_scalar(s),
            UniformType::Vec(v) => self.cast_to_vec(v),
            UniformType::Matrix(m) => self.cast_to_matrix(m),
        }
    }

    fn show_editor(&mut self, ui: &Ui, group_index: usize, binding_index: usize, val_name: &mut String) -> Option<UniformEditEvent> {
        let mut message = None;
        UniformValue::show_primitive_selector(ui, group_index, binding_index, &mut message, 6, val_name);
        ui.same_line();
        match self {
            MatrixUniformValue::M2x2(c1, c2) => {
                MatrixUniformValue::show_size_selector(ui, group_index, binding_index, 0, &mut message);
                let vc1 = c1.values();
                let vc2 = c2.values();
                let mut r1 = [vc1[0], vc2[0]];
                if ui.input_float2(format!("##m_edit_1_{group_index}_{binding_index}"), &mut r1).build() {
                    c1.0 = r1[0];
                    c2.0 = r1[1];
                    message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
                let mut r2 = [vc1[1], vc2[1]];
                if ui.input_float2(format!("##m_edit_2_{group_index}_{binding_index}"), &mut r2).build() {
                    c1.1 = r2[0];
                    c2.1 = r2[1];
                    message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
            },
            MatrixUniformValue::M3x2(c1, c2, c3) => {
                MatrixUniformValue::show_size_selector(ui, group_index, binding_index, 1, &mut message);
                let vc1 = c1.values();
                let vc2 = c2.values();
                let vc3 = c3.values();
                let mut r1 = [vc1[0], vc2[0], vc3[0]];
                if ui.input_float3(format!("##m_edit_1_{group_index}_{binding_index}"), &mut r1).build() {
                    c1.0 = r1[0];
                    c2.0 = r1[1];
                    c3.0 = r1[2];
                    message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
                let mut r2 = [vc1[1], vc2[1], vc3[1]];
                if ui.input_float3(format!("##m_edit_2_{group_index}_{binding_index}"), &mut r2).build() {
                    c1.1 = r2[0];
                    c2.1 = r2[1];
                    c3.1 = r2[2];
                    message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
            },
            MatrixUniformValue::M4x2(c1, c2, c3, c4) => {
                MatrixUniformValue::show_size_selector(ui, group_index, binding_index, 2, &mut message);
                let vc1 = c1.values();
                let vc2 = c2.values();
                let vc3 = c3.values();
                let vc4 = c4.values();
                let mut r1 = [vc1[0], vc2[0], vc3[0], vc4[0]];
                if ui.input_float4(format!("##m_edit_1_{group_index}_{binding_index}"), &mut r1).build() {
                    c1.0 = r1[0];
                    c2.0 = r1[1];
                    c3.0 = r1[2];
                    c4.0 = r1[3];
                    message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
                let mut r2 = [vc1[1], vc2[1], vc3[1], vc4[1]];
                if ui.input_float4(format!("##m_edit_2_{group_index}_{binding_index}"), &mut r2).build() {
                    c1.1 = r2[0];
                    c2.1 = r2[1];
                    c3.1 = r2[2];
                    c4.1 = r2[3];
                    message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
            },
            MatrixUniformValue::M2x3(c1, c2) => {
                MatrixUniformValue::show_size_selector(ui, group_index, binding_index, 3, &mut message);
                let vc1 = c1.values();
                let vc2 = c2.values();
                let mut r1 = [vc1[0], vc2[0]];
                if ui.input_float2(format!("##m_edit_1_{group_index}_{binding_index}"), &mut r1).build() {
                    c1.0 = r1[0];
                    c2.0 = r1[1];
                    message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
                let mut r2 = [vc1[1], vc2[1]];
                if ui.input_float2(format!("##m_edit_2_{group_index}_{binding_index}"), &mut r2).build() {
                    c1.1 = r2[0];
                    c2.1 = r2[1];
                    message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
                let mut r3 = [vc1[2], vc2[2]];
                if ui.input_float2(format!("##m_edit_3_{group_index}_{binding_index}"), &mut r3).build() {
                    c1.2 = r3[0];
                    c2.2 = r3[1];
                    message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
            },
            MatrixUniformValue::M3x3(c1, c2, c3) => {
                MatrixUniformValue::show_size_selector(ui, group_index, binding_index, 4, &mut message);
                let vc1 = c1.values();
                let vc2 = c2.values();
                let vc3 = c3.values();
                let mut r1 = [vc1[0], vc2[0], vc3[0]];
                if ui.input_float3(format!("##m_edit_1_{group_index}_{binding_index}"), &mut r1).build() {
                    c1.0 = r1[0];
                    c2.0 = r1[1];
                    c3.0 = r1[2];
                    message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
                let mut r2 = [vc1[1], vc2[1], vc3[1]];
                if ui.input_float3(format!("##m_edit_2_{group_index}_{binding_index}"), &mut r2).build() {
                    c1.1 = r2[0];
                    c2.1 = r2[1];
                    c3.1 = r2[2];
                    message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
                let mut r3 = [vc1[2], vc2[2], vc3[2]];
                if ui.input_float3(format!("##m_edit_3_{group_index}_{binding_index}"), &mut r3).build() {
                    c1.2 = r3[0];
                    c2.2 = r3[1];
                    c3.2 = r3[2];
                    message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
            },
            MatrixUniformValue::M4x3(c1, c2, c3, c4) => {
                MatrixUniformValue::show_size_selector(ui, group_index, binding_index, 5, &mut message);
                let vc1 = c1.values();
                let vc2 = c2.values();
                let vc3 = c3.values();
                let vc4 = c4.values();
                let mut r1 = [vc1[0], vc2[0], vc3[0], vc4[0]];
                if ui.input_float4(format!("##m_edit_1_{group_index}_{binding_index}"), &mut r1).build() {
                    c1.0 = r1[0];
                    c2.0 = r1[1];
                    c3.0 = r1[2];
                    c4.0 = r1[3];
                    message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
                let mut r2 = [vc1[1], vc2[1], vc3[1], vc4[1]];
                if ui.input_float4(format!("##m_edit_2_{group_index}_{binding_index}"), &mut r2).build() {
                    c1.1 = r2[0];
                    c2.1 = r2[1];
                    c3.1 = r2[2];
                    c4.1 = r2[3];
                    message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
                let mut r3 = [vc1[2], vc2[2], vc3[2], vc4[2]];
                if ui.input_float4(format!("##m_edit_3_{group_index}_{binding_index}"), &mut r3).build() {
                    c1.2 = r3[0];
                    c2.2 = r3[1];
                    c3.2 = r3[2];
                    c4.2 = r3[3];
                    message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
            },
            MatrixUniformValue::M2x4(c1, c2) => {
                MatrixUniformValue::show_size_selector(ui, group_index, binding_index, 6, &mut message);
                let vc1 = c1.values();
                let vc2 = c2.values();
                let mut r1 = [vc1[0], vc2[0]];
                if ui.input_float2(format!("##m_edit_1_{group_index}_{binding_index}"), &mut r1).build() {
                    c1.0 = r1[0];
                    c2.0 = r1[1];
                    message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
                let mut r2 = [vc1[1], vc2[1]];
                if ui.input_float2(format!("##m_edit_2_{group_index}_{binding_index}"), &mut r2).build() {
                    c1.1 = r2[0];
                    c2.1 = r2[1];
                    message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
                let mut r3 = [vc1[2], vc2[2]];
                if ui.input_float2(format!("##m_edit_3_{group_index}_{binding_index}"), &mut r3).build() {
                    c1.2 = r3[0];
                    c2.2 = r3[1];
                    message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
                let mut r4 = [vc1[3], vc2[3]];
                if ui.input_float2(format!("##m_edit_4_{group_index}_{binding_index}"), &mut r4).build() {
                    c1.3 = r4[0];
                    c2.3 = r4[1];
                    message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
            },
            MatrixUniformValue::M3x4(c1, c2, c3) => {
                MatrixUniformValue::show_size_selector(ui, group_index, binding_index, 7, &mut message);
                let vc1 = c1.values();
                let vc2 = c2.values();
                let vc3 = c3.values();
                let mut r1 = [vc1[0], vc2[0], vc3[0]];
                if ui.input_float3(format!("##m_edit_1_{group_index}_{binding_index}"), &mut r1).build() {
                    c1.0 = r1[0];
                    c2.0 = r1[1];
                    c3.0 = r1[2];
                    message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
                let mut r2 = [vc1[1], vc2[1], vc3[1]];
                if ui.input_float3(format!("##m_edit_2_{group_index}_{binding_index}"), &mut r2).build() {
                    c1.1 = r2[0];
                    c2.1 = r2[1];
                    c3.1 = r2[2];
                    message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
                let mut r3 = [vc1[2], vc2[2], vc3[2]];
                if ui.input_float3(format!("##m_edit_3_{group_index}_{binding_index}"), &mut r3).build() {
                    c1.2 = r3[0];
                    c2.2 = r3[1];
                    c3.2 = r3[2];
                    message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
                let mut r4 = [vc1[3], vc2[3], vc3[3]];
                if ui.input_float3(format!("##m_edit_4_{group_index}_{binding_index}"), &mut r4).build() {
                    c1.3 = r4[0];
                    c2.3 = r4[1];
                    c3.3 = r4[2];
                    message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
            },
            MatrixUniformValue::M4x4(c1, c2, c3, c4) => {
                MatrixUniformValue::show_size_selector(ui, group_index, binding_index, 8, &mut message);
                let vc1 = c1.values();
                let vc2 = c2.values();
                let vc3 = c3.values();
                let vc4 = c4.values();
                let mut r1 = [vc1[0], vc2[0], vc3[0], vc4[0]];
                if ui.input_float4(format!("##m_edit_1_{group_index}_{binding_index}"), &mut r1).build() {
                    c1.0 = r1[0];
                    c2.0 = r1[1];
                    c3.0 = r1[2];
                    c4.0 = r1[3];
                    message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
                let mut r2 = [vc1[1], vc2[1], vc3[1], vc4[1]];
                if ui.input_float4(format!("##m_edit_2_{group_index}_{binding_index}"), &mut r2).build() {
                    c1.1 = r2[0];
                    c2.1 = r2[1];
                    c3.1 = r2[2];
                    c4.1 = r2[3];
                    message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
                let mut r3 = [vc1[2], vc2[2], vc3[2], vc4[2]];
                if ui.input_float4(format!("##m_edit_3_{group_index}_{binding_index}"), &mut r3).build() {
                    c1.2 = r3[0];
                    c2.2 = r3[1];
                    c3.2 = r3[2];
                    c4.2 = r3[3];
                    message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
                let mut r4 = [vc1[3], vc2[3], vc3[3], vc4[3]];
                if ui.input_float4(format!("##m_edit_4_{group_index}_{binding_index}"), &mut r4).build() {
                    c1.3 = r4[0];
                    c2.3 = r4[1];
                    c3.3 = r4[2];
                    c4.3 = r4[3];
                    message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
                }
            },
        };
        message
    }

    fn to_le_bytes(&self) -> Vec<u8> {
        let columns = match self {
            MatrixUniformValue::M2x2(c1, c2) => vec![c1.to_le_bytes(), c2.to_le_bytes()],
            MatrixUniformValue::M2x3(c1, c2) => vec![c1.to_le_bytes(), c2.to_le_bytes()],
            MatrixUniformValue::M2x4(c1, c2) => vec![c1.to_le_bytes(), c2.to_le_bytes()],
            MatrixUniformValue::M3x2(c1, c2, c3) => vec![c1.to_le_bytes(), c2.to_le_bytes(), c3.to_le_bytes()],
            MatrixUniformValue::M3x3(c1, c2, c3) => vec![c1.to_le_bytes(), c2.to_le_bytes(), c3.to_le_bytes()],
            MatrixUniformValue::M3x4(c1, c2, c3) => vec![c1.to_le_bytes(), c2.to_le_bytes(), c3.to_le_bytes()],
            MatrixUniformValue::M4x2(c1, c2, c3, c4) => vec![c1.to_le_bytes(), c2.to_le_bytes(), c3.to_le_bytes(), c4.to_le_bytes()],
            MatrixUniformValue::M4x3(c1, c2, c3, c4) => vec![c1.to_le_bytes(), c2.to_le_bytes(), c3.to_le_bytes(), c4.to_le_bytes()],
            MatrixUniformValue::M4x4(c1, c2, c3, c4) => vec![c1.to_le_bytes(), c2.to_le_bytes(), c3.to_le_bytes(), c4.to_le_bytes()],
        };

        columns.into_iter().flatten().collect()
    }
}

impl ImguiMatrix for MatrixUniformValue {
    fn change_matrix_size(&mut self, matrix_size: MatrixType) {
        match self.cast_to_matrix(matrix_size) {
            UniformValue::BuiltIn(_) => unreachable!(),
            UniformValue::Scalar(_) => unreachable!(),
            UniformValue::Vector(_) => unreachable!(),
            UniformValue::Matrix(m) => *self = m,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ScalarType {
    U32,
    I32,
    F32
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum VecType {
    Vec2(ScalarType),
    Vec3(ScalarType),
    Vec4(ScalarType)
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum MatrixType {
    M2x2,
    M2x3,
    M2x4,

    M3x2,
    M3x3,
    M3x4,

    M4x2,
    M4x3,
    M4x4,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum UniformType {
    Scalar(ScalarType),
    Vec(VecType),
    Matrix(MatrixType)
}

impl ImguiUniformSelectable for UniformValue {
    fn cast_to(&self, casted_type: UniformType) -> UniformValue {
        match self {
            UniformValue::Scalar(s) => s.cast_to(casted_type),
            UniformValue::Vector(v) => v.cast_to(casted_type),
            UniformValue::Matrix(m) => m.cast_to(casted_type),
            UniformValue::BuiltIn(_) => unreachable!(),
        }
    }

    fn show_editor(&mut self, ui: &Ui, group_index: usize, binding_index: usize, val_name: &mut String) -> Option<UniformEditEvent> {
        match self {
            UniformValue::BuiltIn(builtin) => match builtin {
                BuiltinValue::Time => {
                    ui.text(format!("({binding_index}) Time (u32)"));
                    None
                },
            },
            UniformValue::Scalar(s) => s.show_editor(ui, group_index, binding_index, val_name),
            UniformValue::Vector(v) => v.show_editor(ui, group_index, binding_index, val_name),
            UniformValue::Matrix(m) => m.show_editor(ui, group_index, binding_index, val_name),
        }
    }

    fn to_le_bytes(&self) -> Vec<u8> {
        match self {
            UniformValue::BuiltIn(b) => b.to_le_bytes(),
            UniformValue::Scalar(s) => s.to_le_bytes(),
            UniformValue::Vector(v) => v.to_le_bytes(),
            UniformValue::Matrix(m) => m.to_le_bytes(),
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
        }
    }

    fn increase(&mut self) {
        match self {
            UniformValue::Scalar(s) => s.increase(),
            UniformValue::Matrix(_) => unreachable!(),
            UniformValue::BuiltIn(_) => unreachable!(),
            UniformValue::Vector(_) => unreachable!(),
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
        }
    }
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

impl UniformValue {
    fn show_primitive_selector(ui: &Ui, group_index: usize, binding_index: usize, message: &mut Option<UniformEditEvent>, type_index: usize, val_name: &mut String) {
        const TYPES: &[UniformType] = &[
            UniformType::Scalar(ScalarType::U32),
            UniformType::Scalar(ScalarType::I32),
            UniformType::Scalar(ScalarType::F32),
            UniformType::Vec(VecType::Vec2(ScalarType::F32)),
            UniformType::Vec(VecType::Vec3(ScalarType::F32)),
            UniformType::Vec(VecType::Vec4(ScalarType::F32)),
            UniformType::Matrix(MatrixType::M4x4),
        ];
        const COMBO_WIDTH: f32 = 70.0;
        const VAR_NAME_WIDTH: f32 = 150.0;

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
    }
}

impl<'a> Into<Cow<'static, str>> for &'a ScalarType {
    fn into(self) -> Cow<'static, str> {
        match self {
            ScalarType::U32 => Cow::Borrowed("u32"),
            ScalarType::I32 => Cow::Borrowed("i32"),
            ScalarType::F32 => Cow::Borrowed("f32"),
        }
    }
}

impl<'a> Into<Cow<'static, str>> for &'a VecType {
    fn into(self) -> Cow<'static, str> {
        match self {
            VecType::Vec2(_) => Cow::Borrowed("vec2"),
            VecType::Vec3(_) => Cow::Borrowed("vec3"),
            VecType::Vec4(_) => Cow::Borrowed("vec4"),
        }
    }
}

impl<'a> Into<Cow<'static, str>> for &'a MatrixType {
    fn into(self) -> Cow<'static, str> {
        match self {
            MatrixType::M2x2 => Cow::Borrowed("2x2"),
            MatrixType::M2x3 => Cow::Borrowed("2x3"),
            MatrixType::M2x4 => Cow::Borrowed("2x4"),

            MatrixType::M3x2 => Cow::Borrowed("3x2"),
            MatrixType::M3x3 => Cow::Borrowed("3x3"),
            MatrixType::M3x4 => Cow::Borrowed("3x4"),

            MatrixType::M4x2 => Cow::Borrowed("4x2"),
            MatrixType::M4x3 => Cow::Borrowed("4x3"),
            MatrixType::M4x4 => Cow::Borrowed("4x4"),
        }
    }
}

impl<'a> Into<Cow<'a, str>> for &'a UniformType {
    fn into(self) -> Cow<'static, str> {
        match self {
            UniformType::Scalar(s) => s.into(),
            UniformType::Vec(v) => v.into(),
            UniformType::Matrix(_) => Cow::Borrowed("matrix"),
        }
    }
}
