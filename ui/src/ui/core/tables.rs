use egui_extras::{Column, TableBuilder, TableRow};

use crate::ui::core::symbols::{ICON_DELETE, ICON_EDIT, ICON_INFO};

pub enum TableAction<T> {
    SelectItem(T, String),
    DeleteItem(T),
    NewReference,
    None
}

pub enum TableMode {
    EditDelete,
    Select,
    Delete,
}

impl TableMode {
    pub fn add_action_column<'a>(&self, table_builder: TableBuilder<'a>) -> TableBuilder<'a> {
        table_builder.column(Column::exact(60.0))
    }
    pub fn add_action_rows<T: Copy>(&self, row: &mut TableRow, primary_key: T, label: &str, table_action: &mut TableAction<T>) {
        match self {
            Self::EditDelete => {
                row.col(|ui| {
                    if ui.button(ICON_EDIT).clicked() {
                        *table_action = TableAction::SelectItem(primary_key, label.to_string())
                    }
                    if ui.button(ICON_DELETE).clicked() {
                        *table_action = TableAction::DeleteItem(primary_key)
                    }
                });
            },
            Self::Select => {
                row.col(|ui| {
                    if ui.button(ICON_INFO).clicked() {
                        *table_action = TableAction::SelectItem(primary_key, label.to_string())
                    }
                });
                if row.response().clicked() {
                    *table_action = TableAction::SelectItem(primary_key, label.to_string())
                }
            },
            Self::Delete => {
                row.col(|ui| {
                    if ui.button(ICON_DELETE).clicked() {
                        *table_action = TableAction::DeleteItem(primary_key)
                    }
                });
            }
        }
    }
}