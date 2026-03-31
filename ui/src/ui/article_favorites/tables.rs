use egui::Sense;
use egui_extras::{Column, TableBuilder};
use uuid::Uuid;

use crate::ui::{core::tables::{TableAction, TableMode}, utils::strong_unselectable};
use core::article_favorites::dto::{ArticleFavoriteUI, UserFavoriteUI};

pub fn show_article_favorites_table(ui: &mut egui::Ui, favorites: &Vec<ArticleFavoriteUI>, table_mode: TableMode) -> TableAction<(Uuid,Uuid)> {
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
                strong_unselectable(ui, "Name");
            });
            header.col(|ui| {
                strong_unselectable(ui, "Created at");
            });
        })
        .body(|body| {
            body.rows(text_height, favorites.len(), |mut row| {
                let favorite = favorites.get(row.index()).unwrap();
                table_mode.add_action_rows(&mut row, (favorite.article_id,favorite.user_id), "", &mut table_action);
                row.col(|ui| {
                    ui.label(&favorite.user_name);
                });
                row.col(|ui| {
                    ui.label(favorite.created_at.to_string().as_str());
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
                table_mode.add_action_rows(&mut row, (article.article_id,article.user_id), "", &mut table_action);
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
