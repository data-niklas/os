mod opts;
mod os;
mod plugin;
mod ui;
mod model;
mod source;
mod history;

pub const APP_NAME: &str = env!("CARGO_PKG_NAME");
pub const APPLICATION_ID: &str = "sh.data-niklas.os";

fn main() {
    let config = opts::Args::read_config();
    let prompt = config.prompt.clone();
    let ui_type = config.ui.clone();
    let app = os::Os::new(config);
    let ui = ui::load_ui(ui_type, app, &prompt);
    ui.run();
}
