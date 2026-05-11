
use core::users::dto::UserUI;

use crate::ui::{core::page::UIContext, utils::date_time_ft};

pub fn ui_user(ui: &mut egui::Ui, user: &mut UserUI, ui_context: &UIContext) {
    ui.label("uuid");
    ui.label(user.id.to_string());
    ui.label("name");
    ui.text_edit_singleline(&mut user.username);
    ui.label("email");
    ui.text_edit_singleline(&mut user.email);
    ui.label("password_hash");
    ui.add(
 egui::TextEdit::singleline(&mut user.password_hash)
            .password(true)
    );
    ui.label("bio");
    ui.text_edit_multiline(&mut user.bio);
    ui.label("image");
    ui.text_edit_multiline(&mut user.image);
    if ui_context.is_admin() {
        ui.checkbox(&mut user.is_admin, "is admin");
    }
    ui.horizontal(|ui| {
        ui.strong("created at");
        ui.label(date_time_ft(user.created_at));
        ui.strong("updated at");
        ui.label(date_time_ft(user.updated_at));
    });

}