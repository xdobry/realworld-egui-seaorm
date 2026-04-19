use egui::Sense;
use egui_extras::{Column, TableBuilder};
use models::Uuid;

use crate::{ui::{core::tables::{TableAction, TableMode}, utils::strong_unselectable}};
use models::entity::users;

pub fn show_users_table(ui: &mut egui::Ui, items: &Vec<users::Model>, table_mode: TableMode) -> TableAction<Uuid> {
    let mut table_action = TableAction::None;
    let height = ui.available_height();
    let text_height = egui::TextStyle::Body
        .resolve(ui.style())
        .size
        .max(ui.spacing().interact_size.y);

    let table: TableBuilder<'_> = TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center));

    let table = table_mode.add_action_column(table)
        .column(Column::exact(500.0).at_least(30.0).at_most(300.0))
        .column(Column::exact(100.0).at_least(30.0).at_most(300.0))
        .min_scrolled_height(height)
        .max_scroll_height(height)
        .sense(Sense::click());

    table
        .header(20.0, |mut header| {
            header.col(|ui| {
                strong_unselectable(ui, "Actions");
            });
            header.col(|ui| {
                strong_unselectable(ui, "Username");
            });
            header.col(|ui| {
                strong_unselectable(ui, "Email");
            });
        })
        .body(|body| {
            body.rows(text_height, items.len(), |mut row| {
                let item = items.get(row.index()).unwrap();
                table_mode.add_action_rows(&mut row, item.id, &item.username, &mut table_action, None);
                row.col(|ui| {
                    ui.label(&item.username);
                });
                row.col(|ui| {
                    ui.label(&item.email);
                });
            });
        });
    table_action
}
