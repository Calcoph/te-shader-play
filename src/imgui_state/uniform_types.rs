use std::borrow::Cow;

use imgui::Ui;

use crate::imgui_state::UniformEditEvent;

pub(crate) use self::{matrix::MatrixType, scalar::{ScalarType, ScalarUniformValue}, vec::VecType};
use self::{matrix::MatrixUniformValue, transform::TransformUniformValue, vec::VectorUniformValue};

use super::{ImguiMatrix, ImguiScalar, ImguiUniformSelectable, ImguiVec, DEFAULT_U32_UNIFORM};

mod scalar;
mod vec;
mod matrix;
mod transform;

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
    Transform(TransformUniformValue)
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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub(crate) enum UniformType {
    Scalar(ScalarType),
    Vec(VecType),
    Matrix(MatrixType),
    Transform
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
    fn show_primitive_selector(ui: &Ui, group_index: usize, binding_index: usize, message: &mut Option<UniformEditEvent>, type_index: usize, val_name: &mut String) {
        const TYPES: &[UniformType] = &[
            UniformType::Scalar(ScalarType::U32),
            UniformType::Scalar(ScalarType::I32),
            UniformType::Scalar(ScalarType::F32),
            UniformType::Vec(VecType::Vec2(ScalarType::F32)),
            UniformType::Vec(VecType::Vec3(ScalarType::F32)),
            UniformType::Vec(VecType::Vec4(ScalarType::F32)),
            UniformType::Matrix(MatrixType::M4x4),
            UniformType::Transform
        ];
        const COMBO_WIDTH: f32 = 95.0;
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

impl<'a> Into<Cow<'a, str>> for &'a UniformType {
    fn into(self) -> Cow<'static, str> {
        match self {
            UniformType::Scalar(s) => s.into(),
            UniformType::Vec(v) => v.into(),
            UniformType::Matrix(_) => Cow::Borrowed("matrix"),
            UniformType::Transform => Cow::Borrowed("transform"),
        }
    }
}
