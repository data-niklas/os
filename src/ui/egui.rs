use std::cell::RefCell;
use std::rc::Rc;

use crate::ui::UI;
use crate::{model::SearchItem, os::Os};

use eframe::egui;

use egui::*;
use image::EncodableLayout;

pub struct EguiUI {
    os: Rc<RefCell<Os>>,
    prompt: String,
}

impl EguiUI {
    pub fn new(os: Os, prompt: &str) -> Self {
        Self {
            os: Rc::new(RefCell::new(os)),
            prompt: prompt.to_string(),
        }
    }
}

impl UI for EguiUI {
    fn run(&mut self) {
        let mut options = eframe::NativeOptions::default();
        options.viewport = options.viewport.with_inner_size(Vec2::new(400.0, 640.0));
        let os = self.os.clone();
        let prompt = self.prompt.clone();
        let _ = eframe::run_native(
            "os",
            options,
            Box::new(move |cc| {
                egui_extras::install_image_loaders(&cc.egui_ctx);
                let mut app = App::new(os, prompt);
                app.search();
                Box::new(app)
            }),
        );
    }
}

struct App {
    prompt: String,
    os: Rc<RefCell<Os>>,
    items: Vec<SearchItem>,
    text: String,
    selected_index: usize,
}

impl App {
    pub fn new(os: Rc<RefCell<Os>>, prompt: String) -> Self {
        Self {
            prompt,
            os,
            items: vec![],
            text: String::new(),
            selected_index: 0,
        }
    }
    pub fn search(&mut self) {
        // self.list.select(Some(0));
        let os = self.os.borrow_mut();
        self.items = os
            .search(&self.text)
            .into_iter()
            .take(os.config.maximum_list_item_count)
            .collect();
        self.selected_index = 0;
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(Layout::top_down_justified(Align::Min), |ui| {
                let res = TextEdit::singleline(&mut self.text)
                    .hint_text(self.prompt.clone())
                    .font(TextStyle::Heading)
                    .lock_focus(true)
                    .margin(Margin::symmetric(4.0, 4.0))
                    .ui(ui);
                ui.add_space(6.0);
                if !res.has_focus() {
                    res.request_focus();
                }

                let mut items_changed = false;
                if res.changed() {
                    self.search();
                    items_changed = true;
                }

                if ui.input(|i| i.key_pressed(Key::ArrowDown))
                    && self.selected_index < self.items.len() - 1
                {
                    self.selected_index = self.selected_index + 1;
                    items_changed = true;
                }
                if ui.input(|i| i.key_pressed(Key::ArrowUp)) && self.selected_index > 0 {
                    self.selected_index = self.selected_index - 1;
                    items_changed = true;
                }
                if ui.input(|i| i.key_pressed(Key::Enter)) {
                    if let Some(item) = self.items.get(self.selected_index) {
                        let mut os = self.os.borrow_mut();
                        if os.select(&item) {
                            os.deinit();
                            std::process::exit(0);
                        } else {
                            self.items = vec![];
                            self.text = String::new();
                            items_changed = true;
                        }
                    }
                }
                if ui.input(|i| i.key_pressed(Key::Escape)) {
                    self.os.borrow_mut().deinit();
                    std::process::exit(0);
                }

                let _scroll_area = ScrollArea::vertical().show(ui, |ui| {
                    for (i, item) in self.items.iter().enumerate() {
                        let mut group = Frame::group(&ui.style());
                        if i == self.selected_index {
                            group = group
                                .fill(Color32::from_rgb(84, 78, 78))
                                .stroke(Stroke::new(0.0, Color32::from_rgb(200, 208, 236)));
                        }
                        let response = group.show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let title_text: RichText =
                                    item.title.clone().unwrap_or_default().into();
                                let title_text_height = ui.fonts(|fonts| {
                                    title_text.font_height(fonts, ui.style().as_ref())
                                });
                                let title = Label::new(title_text.heading());
                                let subtitle_text: RichText =
                                    item.subtitle.clone().unwrap_or_default().into();
                                let subtitle_text_height = ui.fonts(|fonts| {
                                    subtitle_text.font_height(fonts, ui.style().as_ref())
                                });
                                let subtitle = Label::new(subtitle_text);
                                // Padding between title and subtitle needs to be considered
                                let max_icon_size = title_text_height
                                    + subtitle_text_height
                                    + ui.spacing().item_spacing.x;
                                if let Some(icon) = &item.icon {
                                    let color_image = ColorImage::from_rgba_unmultiplied(
                                        [icon.width() as usize, icon.height() as usize],
                                        icon.as_bytes(),
                                    );
                                    let size = egui::vec2(
                                        color_image.size[0] as f32,
                                        color_image.size[1] as f32,
                                    );
                                    let handle = ctx.load_texture(
                                        "bytes://",
                                        color_image,
                                        TextureOptions::LINEAR,
                                    );
                                    let sized_image =
                                        egui::load::SizedTexture::new(handle.id(), size);
                                    let image = egui::Image::from_texture(sized_image)
                                        .max_width(max_icon_size)
                                        .max_height(max_icon_size);
                                    ui.add(image);
                                } else {
                                    ui.add_space(max_icon_size);
                                }
                                let _height =
                                    ui.with_layout(Layout::top_down_justified(Align::Min), |ui| {
                                        ui.add(title);
                                        ui.add(subtitle);
                                    });
                            });
                        });
                        if i == self.selected_index && items_changed {
                            response.response.scroll_to_me(None);
                        }
                    }
                });
            });
        });
    }
}
