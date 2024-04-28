mod gtk;
pub use gtk::*;

mod ratatui;
pub use ratatui::*;

mod egui;
pub use egui::*;

use crate::os::Os;

pub trait UI {
    // fn show(&self);
    // fn hide(&self);
    fn run(&mut self);
}

pub fn load_ui(ui: String, os: Os, prompt: &str, wayland_layer: bool) -> Box<dyn UI> {
    match ui.as_str() {
        "gtk" => Box::new(GtkUI::new(os, prompt, wayland_layer)),
        "ratatui" => Box::new(RatatuiUI::new(os, prompt)),
        "egui" => Box::new(EguiUI::new(os, prompt)),
        _ => unimplemented!(),
    }
}
