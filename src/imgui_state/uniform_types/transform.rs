use cgmath::{Deg, Euler, Matrix4, Quaternion, Vector3, Vector4};

use crate::imgui_state::{ImguiUniformSelectable, UniformEditEvent};

use super::{matrix::{Column2, Column3, Column4, MatrixUniformValue}, vec::{Vec2UniformValue, Vec3UniformValue, Vec4UniformValue, VectorUniformValue}, MatrixType, ScalarType, ScalarUniformValue, UniformType, UniformValue, VecType};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct TransformUniformValue {
    translation: Vector3<f32>,
    x_scale: f32,
    y_scale: f32,
    z_scale: f32,
    rotation: Quaternion<f32>
}

impl TransformUniformValue {
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
}

trait Byteable {
    fn to_le_bytes(&self) -> Vec<u8>;
}

impl Byteable for Matrix4<f32> {
    fn to_le_bytes(&self) -> Vec<u8> {
        self.x.to_le_bytes()
    }
}

impl Byteable for Vector4<f32> {
    fn to_le_bytes(&self) -> Vec<u8> {
        self.x.to_le_bytes()
            .into_iter()
            .chain(
                self.y.to_le_bytes()
                .into_iter()
                .chain(
                    self.z.to_le_bytes()
                    .into_iter()
                    .chain(
                        self.w.to_le_bytes()
                        .into_iter()
                    )
                )
            ).collect()
    }
}

impl ImguiUniformSelectable for TransformUniformValue {
    fn cast_to(&self, casted_type: super::UniformType) -> super::UniformValue {
        // TODO: Do like other types and keep as much data as possible
        match casted_type {
            UniformType::Scalar(s) => self.cast_to_scalar(s),
            UniformType::Vec(v) => self.cast_to_vec(v),
            UniformType::Matrix(m) => self.cast_to_matrix(m),
            UniformType::Transform => unreachable!(),
        }
    }

    fn show_editor(&mut self, ui: &imgui::Ui, group_index: usize, binding_index: usize, val_name: &mut String) -> Option<UniformEditEvent> {
        let mut message = None;
        UniformValue::show_primitive_selector(ui, group_index, binding_index, &mut message, 7, val_name);
        ui.text("Position");
        ui.indent();
        ui.text("x, y, z");
        let mut translation = [self.translation.x, self.translation.y, self.translation.z];
        if ui.input_float3(format!("##pos_{group_index}_{binding_index}"), &mut translation).build() {
            self.translation.x = translation[0];
            self.translation.y = translation[1];
            self.translation.z = translation[2];
            message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
        };
        ui.unindent();
        ui.text("Scale");
        ui.indent();
        ui.text("x, y, z");
        let mut scale = [self.x_scale, self.y_scale, self.z_scale];
        if ui.input_float3(format!("##scale_{group_index}_{binding_index}"), &mut scale).build() {
            self.x_scale = scale[0];
            self.y_scale = scale[1];
            self.z_scale = scale[2];
            message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
        };
        ui.unindent();
        ui.text("Rotation");
        ui.indent();
        ui.text("euler (x, y, z)");
        let euler = Euler::from(self.rotation);
        let euler_x = Deg::from(euler.x).0;
        let euler_y = Deg::from(euler.y).0;
        let euler_z = Deg::from(euler.z).0;
        let mut euler = [euler_x, euler_y, euler_z];
        if ui.input_float3(format!("##euler_{group_index}_{binding_index}"), &mut euler).build() {
            let euler = Euler {
                x: Deg(euler[0]),
                y: Deg(euler[1]),
                z: Deg(euler[2]),
            };
            self.rotation = Quaternion::from(euler);
            message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
        };
        let quat_x = self.rotation.v.x;
        let quat_y = self.rotation.v.y;
        let quat_z = self.rotation.v.z;
        let quat_w = self.rotation.s;
        let mut rotation_quat = [quat_x, quat_y, quat_z, quat_w];
        ui.text("quaternion (x, y, z, w)");
        if ui.input_float4(format!("##quat_{group_index}_{binding_index}"), &mut rotation_quat).build() {
            self.rotation = Quaternion::from(rotation_quat);
            message = Some(UniformEditEvent::UpdateBuffer(group_index, binding_index))
        };
        ui.unindent();

        message
    }

    fn to_le_bytes(&self) -> Vec<u8> {
        let transfrom = Matrix4::from_translation(self.translation)
            * Matrix4::from(self.rotation)
            * Matrix4::from_nonuniform_scale(self.x_scale, self.y_scale, self.z_scale);
        transfrom.to_le_bytes()
    }
}

impl Default for TransformUniformValue {
    fn default() -> Self {
        Self {
            translation: Vector3 { x: 0.0, y: 0.0, z: 0.0 },
            x_scale: 1.0,
            y_scale: 1.0,
            z_scale: 1.0,
            rotation: Quaternion::new(0.0, 0.0, 0.0, 0.0)
        }
    }
}
