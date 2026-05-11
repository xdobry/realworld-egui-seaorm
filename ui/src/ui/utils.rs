use egui::Ui;
use models::DateTimeWithTimeZone;

pub fn strong_unselectable(ui: &mut Ui, text: impl Into<egui::RichText>) {
    let l = egui::Label::new(text.into().strong()).selectable(false);
    ui.add(l);
}

pub fn date_time_ft(dt: DateTimeWithTimeZone) -> String {
    dt.format("%d-%m-%Y %H:%M").to_string()
}

