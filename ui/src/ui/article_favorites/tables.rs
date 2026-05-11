use egui::Sense;
use egui_extras::{Column, TableBuilder};
use models::Uuid;

use crate::ui::{core::{page::UIContext, tables::{TableAction, TableMode}}, utils::{date_time_ft, strong_unselectable}};
use core::article_favorites::dto::{ArticleFavoriteUI, UserFavoriteUI};

pub fn show_article_favorites_table(ui: &mut egui::Ui, favorites: &Vec<ArticleFavoriteUI>, ui_context: &UIContext) -> TableAction<(Uuid,Uuid)> {
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
        .column(Column::exact(200.0).at_least(30.0).at_most(300.0))
        .column(Column::exact(200.0).at_least(30.0).at_most(300.0))
        .min_scrolled_height(height)
        .max_scroll_height(height)
        .sense(Sense::click());

    table
        .header(20.0, |mut header| {
            header.col(|ui| {
                strong_unselectable(ui, "Action");
            });
            header.col(|ui| {
                strong_unselectable(ui, "User");
            });
            header.col(|ui| {
                strong_unselectable(ui, "Created at");
            });
        })
        .body(|body| {
            body.rows(text_height, favorites.len(), |mut row| {
                let favorite = favorites.get(row.index()).unwrap();
                let row_table_mode = if ui_context.is_user_or_admin(favorite.user_id) {
                    TableMode::Delete
                } else {
                    TableMode::Nothing
                };
                row_table_mode.add_action_rows(&mut row, (favorite.user_id,favorite.article_id), "", &mut table_action, Some(favorite.user_id));
                row.col(|ui| {
                    ui.label(&favorite.user_name);
                });
                row.col(|ui| {
                    ui.label(date_time_ft(favorite.created_at));
                });
            });
        });
    table_action
}


pub fn show_user_favorites_table(ui: &mut egui::Ui, articles: &Vec<UserFavoriteUI>, table_mode: TableMode) -> TableAction<(Uuid,Uuid)> {
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
        .column(Column::exact(200.0).at_least(30.0).at_most(300.0))
        .column(Column::exact(200.0).at_least(30.0).at_most(300.0))
        .min_scrolled_height(height)
        .max_scroll_height(height)
        .sense(Sense::click());

    table
        .header(20.0, |mut header| {
            header.col(|ui| {
                strong_unselectable(ui, "Action");
            });
            header.col(|ui| {
                strong_unselectable(ui, "Title");
            });
            header.col(|ui| {
                strong_unselectable(ui, "Created at");
            });
        })
        .body(|body| {
            body.rows(text_height, articles.len(), |mut row| {
                let article = articles.get(row.index()).unwrap();
                table_mode.add_action_rows(&mut row, (article.user_id,article.article_id), "", &mut table_action, Some(article.article_id));
                row.col(|ui| {
                    ui.label(&article.article_title);
                });
                row.col(|ui| {
                    ui.label(article.created_at.to_string().as_str());
                });
            });
        });
    table_action
}
