mod helpers;
mod history;
mod model;
mod opts;
mod os;
mod source;
mod ui;

pub const APP_NAME: &str = env!("CARGO_PKG_NAME");
pub const APPLICATION_ID: &str = "sh.data-niklas.os";

fn main() {
    env_logger::init();
    let config = opts::Args::read_config();
    let prompt = config.prompt.clone();
    let ui_type = config.ui.clone();
    let app = os::Os::new(config);
    let mut ui = ui::load_ui(ui_type, app, &prompt);
    ui.run();
}
