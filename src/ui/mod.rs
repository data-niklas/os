mod gtk;
pub use gtk::*;

use crate::os::Os;

pub trait UI {
    // fn show(&self);
    // fn hide(&self);
    fn run(&self);
}

pub fn load_ui(ui: String, os: Os, prompt: &str) -> Box<dyn UI> {
    match ui.as_str() {
        "gtk" => Box::new(GtkUI::new(os, prompt)),
        _ => unimplemented!(),
    }
}
