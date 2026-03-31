use egui::Sense;
use egui_extras::{Column, TableBuilder};
use uuid::Uuid;

use crate::{ui::{core::tables::{TableAction, TableMode}, utils::strong_unselectable}};

use models::entity::articles;

pub fn show_articles_table(ui: &mut egui::Ui, articles: &Vec<articles::Model>, table_mode: TableMode) -> TableAction<Uuid> {
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
        .column(Column::exact(100.0).at_least(30.0).at_most(150.0))
        .column(Column::exact(510.0).at_least(50.0).at_most(600.0))
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
                strong_unselectable(ui, "Slug");
            });
            header.col(|ui| {
                strong_unselectable(ui, "Title");
            });
            header.col(|ui| {
                strong_unselectable(ui, "Description");
            });
        })
        .body(|body| {
            body.rows(text_height, articles.len(), |mut row| {
                let article = articles.get(row.index()).unwrap();
                table_mode.add_action_rows(&mut row, article.id, &article.title, &mut table_action);
                row.col(|ui| {
                    ui.label(&article.slug);
                });
                row.col(|ui| {
                    ui.label(&article.title);
                });
                row.col(|ui| {
                    ui.label(&article.description);
                });
            });
        });
    table_action
}

