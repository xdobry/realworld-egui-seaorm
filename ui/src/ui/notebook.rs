use egui::{Color32, Rect, Shape, TextStyle, Ui, Vec2, WidgetText, epaint::{PathShape, PathStroke}, pos2, style::WidgetVisuals, text::LayoutJob};

pub struct NoteBook<'a, F>
where
    F: Fn(usize) -> WidgetText,
{
    len: usize,
    selected: &'a mut usize,
    label_fn: F,
    max_width: f32,
    closable: bool,
}

impl<'a, F> NoteBook<'a, F>
where
    F: Fn(usize) -> WidgetText,
{
    pub fn new(len: usize, selected: &'a mut usize, label_fn: F, max_width: f32, closable: bool) -> Self {
        Self {
            len,
            selected,
            label_fn,
            max_width,
            closable,
        }
    }


    pub fn show(self, ui: &mut Ui) -> Option<usize> {
        if self.len == 0 {
            return None;
        }
        let mut close_index: Option<usize> = None;
        let mut selected_rect: Option<Rect> = None; 
        let arect = ui.available_rect_before_wrap();
        let font_id = TextStyle::Body.resolve(ui.style());
        let r = ui.visuals().widgets.active.corner_radius.average();
        let mut points = Vec::with_capacity(10);
        let gap = ui.spacing().item_spacing.x * 0.5;

        ui.horizontal(|ui| {
            for i in 0..self.len {
                // let label_str = (self.label_fn)(i).text();
                let galley = ui.fonts_mut(|f| {                   
                    let mut job = LayoutJob::default();
                    job.append(
                        (self.label_fn)(i).text(),
                        0.0,
                        egui::TextFormat {
                            font_id: font_id.clone(),
                            color: Color32::PLACEHOLDER,
                            ..Default::default()
                        },
                    );
                    if self.max_width>0.0 {
                        job.wrap = egui::text::TextWrapping {
                            max_width: self.max_width,
                            max_rows: 1,
                            break_anywhere: true,
                            ..Default::default()
                        };
                    }
                    f.layout_job(job)
                });
                if i == 0 {
                    // draw background
                    let frect = arect.with_max_y(arect.top()+galley.size().y+ui.spacing().item_spacing.y);
                    ui.painter().rect_filled(frect, 0.0, ui.visuals().faint_bg_color);        
                }
                let mut hover_color = false;
                if ui.available_width()<galley.size().x + 20.0 {
                    egui::ComboBox::from_id_salt(format!("cb{}",self.len-i))
                        .width(0.0)
                        .show_ui(ui, |ui| {
                            for ci in 0..self.len {
                                ui.selectable_value(  self.selected, ci, (self.label_fn)(ci).text());
                            }
                        });
                    break;
                } else {
                    let (rect, response) = ui.allocate_exact_size(galley.size(), egui::Sense::click());
                    let cross_data: Option<(Rect, &WidgetVisuals)> = if self.closable {
                        let c_size = Vec2::splat(ui.spacing().icon_width*0.5);
                        let (c_rect, c_response) = ui.allocate_exact_size(c_size, egui::Sense::click());
                        if c_response.clicked() {
                            close_index = Some(i);
                        }
                        if response.clicked_by(egui::PointerButton::Middle) {
                            close_index = Some(i);
                        }
                        let visuals = ui.style().interact(&c_response);
                        Some((c_rect,visuals))
                    } else {
                        None
                    };
                    if i == *self.selected {
                        let mut rect = if let Some((c_rect, _visuals)) = cross_data {
                            rect.union(c_rect).with_max_y(rect.bottom()+ui.spacing().item_spacing.y)
                        } else {
                            rect
                        };
                        rect.max.x = rect.max.x + gap;
                        if i>0 {
                            rect.min.x = rect.min.x - gap;
                        }
                        
                        selected_rect = Some(rect);

                        // Start bottom-left (flat bottom)
                        points.push(pos2(rect.left(), rect.bottom()));

                        // left edge up
                        points.push(pos2(rect.left(), rect.top() + r));

                        // top-left corner (approx arc)
                        // 1-sin(45) = 1-cos(45)
                        points.push(pos2(rect.left() + r * 0.293, rect.top() + r * 0.293));
                        points.push(pos2(rect.left() + r, rect.top()));

                        // top edge
                        points.push(pos2(rect.right() - r, rect.top()));

                        // top-right corner
                        points.push(pos2(rect.right() - r * 0.293, rect.top()+ r * 0.293));
                        points.push(pos2(rect.right(), rect.top() + r));

                        // right edge down
                        points.push(pos2(rect.right(), rect.bottom()));

                        // fill the area
                        ui.painter().add(Shape::Path(PathShape {
                            // we clone because we want to use it also for stroke painting
                            points: points.clone(), 
                            closed: true,
                            fill:ui.visuals().noninteractive().bg_fill, 
                            stroke: PathStroke::NONE,
                        }));
                    } else {
                        if i>0 && i-1 != *self.selected {
                            ui.painter().vline(rect.left()-gap,rect.top()..=rect.bottom(), 
                                ui.visuals().noninteractive().bg_stroke);

                        }
                        if response.hovered() {
                            hover_color = true
                        }
                    }
                    ui.painter().galley(rect.min, galley,
                        if hover_color { ui.visuals().widgets.hovered.text_color() } else { ui.visuals().widgets.noninteractive.text_color()});

                    if let Some((c_rect,visuals)) = cross_data {
                        let stroke = visuals.fg_stroke;
                        ui.painter() // paints \
                            .line_segment([c_rect.left_top(), c_rect.right_bottom()], stroke);
                        ui.painter() // paints /
                            .line_segment([c_rect.right_top(), c_rect.left_bottom()], stroke);
                    }

                    if response.clicked() {
                        *self.selected = i;
                    }

                }
            }
        });
        if let Some(rect) = selected_rect {
            // Start bottom-left (flat bottom)
            points.insert(0,pos2(arect.left(), rect.bottom()));          
            points.push(pos2(arect.right(), rect.bottom()));

            let painter = ui.painter();
            painter.add(Shape::Path(PathShape {
                points, 
                closed: false,
                fill: Color32::TRANSPARENT, 
                stroke: PathStroke::new(ui.visuals().noninteractive().bg_stroke.width, ui.visuals().noninteractive().bg_stroke.color)
            }));
        }
        ui.add_space(ui.spacing().item_spacing.y);
        close_index
    }
}