use std::borrow::Cow;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use crate::ui::UI;
use crate::{model::SearchItem, os::Os};

use eframe::egui;
use egui::*;

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
                res.request_focus();

                if res.changed() {
                    self.search();
                }

                if ui.input(|i| i.key_pressed(Key::ArrowDown))
                    && self.selected_index < self.items.len() - 1
                {
                    self.selected_index = self.selected_index + 1;
                }
                if ui.input(|i| i.key_pressed(Key::ArrowUp)) && self.selected_index > 0 {
                    self.selected_index = self.selected_index - 1;
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
                        }
                    }
                }
                if ui.input(|i| i.key_pressed(Key::Escape)) {
                    self.os.borrow_mut().deinit();
                    std::process::exit(0);
                }

                let scroll_area = ScrollArea::vertical().show(ui, |ui| {
                    for (i, item) in self.items.iter().enumerate() {
                        let mut group = Frame::group(&ui.style());
                        if i == self.selected_index {
                            group = group
                                .fill(Color32::from_rgb(84, 78, 78))
                                .stroke(Stroke::new(0.0, Color32::from_rgb(200, 208, 236)));
                        }
                        let response = group.show(ui, |ui| {
                            // if item.icon.is_some() {
                            //     let pixbuf = item.icon.as_ref().unwrap().borrow();
                            //     let bytes = pixbuf.read_pixel_bytes().to_vec().into_boxed_slice();
                            //     let bytes = load::Bytes::Shared(Arc::from(bytes));
                            //     let uri = format!("{}.jpeg", i);
                            //     let uri = Cow::Owned(uri);
                            //     ui.image(ImageSource::Bytes { uri, bytes });
                            // }
                            ui.heading(item.title.clone().unwrap_or_default());
                            ui.label(item.subtitle.clone().unwrap_or_default());
                        });
                        if i == self.selected_index {
                            response.response.scroll_to_me(None);
                        }
                    }
                });
            });
        });
    }
}
