
use core::tags::dto::TagUI;

use crate::ui::utils::date_time_ft;

pub fn ui_tag(ui: &mut egui::Ui, tag: &mut TagUI) {
    ui.label("uuid");
    ui.label(tag.id.to_string());
    ui.label("name");
    ui.text_edit_singleline(&mut tag.name);
    ui.label("created at");
    ui.label(date_time_ft(tag.created_at));
}