
use core::users::dto::UserUI;
use std::io::Cursor;

use egui::{TextEdit, TextureHandle};
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use image::{ImageFormat, imageops::FilterType};

use crate::ui::{core::page::UIContext, utils::date_time_ft};

pub fn ui_user(ui: &mut egui::Ui, user: &mut UserUI, ui_context: &UIContext, cache: &mut CommonMarkCache, user_image: &Option<TextureHandle>, new_image: &mut bool) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        if let Some(user_image) = user_image.as_ref() {
            ui.add(egui::Image::new(user_image).max_height(200.0));
        }
        if ui_context.is_edit() {
            if user.image.is_some() {
                ui.label("drop new image to replace");
            } else {
                ui.label("drop image here");
            }
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
            TextEdit::multiline(&mut user.bio).desired_width(f32::INFINITY).desired_rows(30).show(ui);
            let dropped_files = ui.ctx().input(|i| i.raw.dropped_files.clone());
            if dropped_files.len() > 0 {
                if let Some(file) = dropped_files.first() {
                    if let Some(path) = &file.path {
                        match std::fs::read(path) {
                            Ok(data) => {
                                let data = resize_png(&data, 200, 200);
                                if let Ok(data) = data {
                                    user.image = Some(data);
                                    *new_image = true;
                                } else {
                                    eprintln!("Failed to resize image {}: {data:?}", path.display());
                                }
                            }
                            Err(err) => {
                                eprintln!("Failed to read {}: {err}", path.display());
                            }
                        }
                    } else if let Some(data) = &file.bytes {
                        user.image = Some(data.to_vec());
                    }
                }
            }
            if ui_context.is_admin() {
                ui.checkbox(&mut user.is_admin, "is admin");
            }
        } else {
            if user.is_admin {
                ui.strong("admin");
            }
            ui.horizontal(|ui| {
                ui.strong("name:");
                ui.label(&user.username);

            });
            ui.horizontal(|ui| {
                ui.strong("email:");
                ui.label(&user.email);
            });
            CommonMarkViewer::new().show(ui, cache, &user.bio);
        }
        ui.horizontal(|ui| {
            ui.strong("created at");
            ui.label(date_time_ft(user.created_at));
            ui.strong("updated at");
            ui.label(date_time_ft(user.updated_at));
        });
    });

}

// TODO this can take a while, should be done in a separate thread
fn resize_png(
    input_png: &[u8],
    max_height: u32,
    max_width: u32,
) -> image::ImageResult<Vec<u8>> {
    // Decode
    let img = image::load_from_memory(input_png)?;
    if img.width() <= max_width && img.height() <= max_height {
        let mut out = Vec::new();
        img.write_to(
            &mut Cursor::new(&mut out),
            ImageFormat::Png,
        )?;
        return Ok(out);
    }
    // Resize while preserving aspect ratio
    let resized = img.resize(max_width, max_height, FilterType::Lanczos3);

    // Encode back to PNG
    let mut out = Vec::new();
    resized.write_to(
        &mut Cursor::new(&mut out),
        ImageFormat::Png,
    )?;

    Ok(out)
}