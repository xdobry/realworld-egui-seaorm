
use core::users::dto::UserUI;

pub fn ui_user(ui: &mut egui::Ui, user: &mut UserUI) {
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
    ui.label("created at");
    ui.label(user.created_at.to_string().as_str());
    ui.label("updated at");
    ui.label(user.updated_at.to_string().as_str());

}