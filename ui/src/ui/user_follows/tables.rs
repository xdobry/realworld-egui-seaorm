use egui::Sense;
use egui_extras::{Column, TableBuilder};
use models::Uuid;

use crate::ui::{core::{page::UIContext, tables::{TableAction, TableMode}}, utils::{date_time_ft, strong_unselectable}};
use core::user_follows::dto::UserFollowerName;

pub fn show_user_followers_table(ui: &mut egui::Ui, followers: &Vec<UserFollowerName>, ui_context: &UIContext, show_followers: bool) -> TableAction<(Uuid,Uuid)> {
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

    let follow_name = if show_followers { "Follower Name"} else { "Followee Name" };

    table
        .header(20.0, |mut header| {
            header.col(|ui| {
                strong_unselectable(ui, "Action");
            });
            header.col(|ui| {
                strong_unselectable(ui, follow_name);
            });
            header.col(|ui| {
                strong_unselectable(ui, "Created at");
            });
        })
        .body(|body| {
            body.rows(text_height, followers.len(), |mut row| {
                let follower = followers.get(row.index()).unwrap();
                let row_table_mode = if ui_context.is_user_or_admin(follower.follower_id) {
                    TableMode::Delete
                } else {
                    TableMode::Nothing
                };
                row_table_mode.add_action_rows(&mut row, (follower.follower_id,follower.followee_id), "", &mut table_action, None);
                row.col(|ui| {
                    ui.label(&follower.follower_name);
                });
                row.col(|ui| {
                    ui.label(date_time_ft(follower.created_at));
                });
            });
        });
    table_action
}
