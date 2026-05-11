use core::articles::dto::{ArticleListItem, ArticlePopularListItem};

use egui::Sense;
use egui_extras::{Column, TableBuilder};
use models::Uuid;

use crate::ui::{core::{page::UIContext, tables::{TableAction, TableMode}}, utils::{date_time_ft, strong_unselectable}};

pub fn show_articles_table(ui: &mut egui::Ui, articles: &Vec<ArticleListItem>, ui_context: &UIContext, only_select: bool) -> TableAction<Uuid> {
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

    let table = TableMode::Select.add_action_column(table)
        .column(Column::exact(150.0).at_least(30.0).at_most(350.0))
        .column(Column::exact(510.0).at_least(50.0).at_most(800.0))
        .column(Column::exact(150.0).at_least(30.0).at_most(150.0))
        .min_scrolled_height(height)
        .max_scroll_height(height)
        .sense(Sense::click());

    table
        .header(20.0, |mut header| {
            header.col(|ui| {
                strong_unselectable(ui, "Actions");
            });
            header.col(|ui| {
                strong_unselectable(ui, "Title");
            });
            header.col(|ui| {
                strong_unselectable(ui, "Description");
            });
            header.col(|ui| {
                strong_unselectable(ui, "Created");
            });
        })
        .body(|body| {
            body.rows(text_height, articles.len(), |mut row| {
                let article = articles.get(row.index()).unwrap();
                let row_table_mode = if !only_select && ui_context.is_user_or_admin(article.author_id)  {
                    TableMode::EditDelete
                } else {
                    TableMode::Select
                };
                row_table_mode.add_action_rows(&mut row, article.id, &article.title, &mut table_action, None);
                row.col(|ui| {
                    ui.label(&article.title);
                });
                row.col(|ui| {
                    ui.label(&article.description);
                });
                row.col(|ui| {
                    ui.label(date_time_ft(article.created_at));
                });
            });
        });
    table_action
}

pub fn show_popular_articles_table(ui: &mut egui::Ui, articles: &Vec<ArticlePopularListItem>, ui_context: &UIContext) -> TableAction<Uuid> {
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

    let table = TableMode::Select.add_action_column(table)
        .column(Column::exact(150.0).at_least(30.0).at_most(350.0))
        .column(Column::exact(410.0).at_least(50.0).at_most(600.0))
        .column(Column::exact(120.0).at_least(30.0).at_most(120.0))
        .column(Column::exact(120.0).at_least(30.0).at_most(120.0))
        .min_scrolled_height(height)
        .max_scroll_height(height)
        .sense(Sense::click());

    table
        .header(20.0, |mut header| {
            header.col(|ui| {
                strong_unselectable(ui, "Actions");
            });
            header.col(|ui| {
                strong_unselectable(ui, "Title");
            });
            header.col(|ui| {
                strong_unselectable(ui, "Description");
            });
            header.col(|ui| {
                strong_unselectable(ui, "Favorite Count");
            });
            header.col(|ui| {
                strong_unselectable(ui, "Created");
            });
        })
        .body(|body| {
            body.rows(text_height, articles.len(), |mut row| {
                let article = articles.get(row.index()).unwrap();
                let row_table_mode = if ui_context.is_user_or_admin(article.author_id)  {
                    TableMode::EditDelete
                } else {
                    TableMode::Select
                };
                row_table_mode.add_action_rows(&mut row, article.id, &article.title, &mut table_action, None);
                row.col(|ui| {
                    ui.label(&article.title);
                });
                row.col(|ui| {
                    ui.label(&article.description);
                });
                row.col(|ui| {
                    ui.label(&article.count.to_string());
                });
                row.col(|ui| {
                    ui.label(date_time_ft(article.created_at));
                });
            });
        });
    table_action
}

