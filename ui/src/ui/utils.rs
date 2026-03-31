use egui::Ui;

pub fn strong_unselectable(ui: &mut Ui, text: impl Into<egui::RichText>) {
    let l = egui::Label::new(text.into().strong()).selectable(false);
    ui.add(l);
}

