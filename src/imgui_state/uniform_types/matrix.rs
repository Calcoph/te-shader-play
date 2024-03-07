use std::borrow::Cow;

use imgui::Ui;

use crate::imgui_state::{uniform_types::ExtendedUi, ImguiMatrix, ImguiUniformSelectable, UniformEditEvent};

use super::{scalar::ScalarUniformValue, transform::TransformUniformValue, vec::{Vec2UniformValue, Vec3UniformValue, Vec4UniformValue, VectorUniformValue}, ScalarType, UniformType, UniformValue, VecType};


trait MatrixColumn {
    fn to_le_bytes(&self) -> Vec<u8>;
    fn values(&self) -> Vec<f32>;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Column2(pub f32, pub f32);
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Column3(pub f32, pub f32, pub f32);
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Column4(pub f32, pub f32, pub f32, pub f32);

impl MatrixColumn for Column2 {
    fn to_le_bytes(&self) -> Vec<u8> {
        self.0.to_le_bytes()
            .into_iter()
            .chain(self.1.to_le_bytes())
            .collect()
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
        self.0.to_le_bytes()
            .into_iter()
            .chain(self.1.to_le_bytes())
            .chain(self.2.to_le_bytes())
            .collect()
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
        self.0.to_le_bytes()
            .into_iter()
            .chain(self.1.to_le_bytes())
            .chain(self.2.to_le_bytes())
            .chain(self.3.to_le_bytes())
            .collect()
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
pub(crate) enum MatrixUniformValue {
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

    fn cast_to_transform(&self) -> UniformValue {
        UniformValue::Transform(TransformUniformValue::default())
    }
}


impl ImguiUniformSelectable for MatrixUniformValue {
    fn cast_to(&self, casted_type: UniformType) -> UniformValue {
        // TODO: Do like other types and keep as much data as possible
        match casted_type {
            UniformType::Scalar(s) => self.cast_to_scalar(s),
            UniformType::Vec(v) => self.cast_to_vec(v),
            UniformType::Matrix(m) => self.cast_to_matrix(m),
            UniformType::Transform => self.cast_to_transform(),
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
            UniformValue::Transform(_) => unreachable!(),
        }
    }
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
