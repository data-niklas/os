use std::rc::Rc;

use crate::ui::UI;
use crate::APPLICATION_ID;
use gtk::prelude::*;
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use relm4::{factory::FactoryVecDeque, prelude::*, typed_view::list::{RelmListItem, TypedListView}};
// use gio::prelude::*;
use crate::os::Os;
use gtk::{Application, ApplicationWindow, Button, Entry, SearchEntry};

pub struct GtkUI {
    os: Rc<Os>,
    prompt: String,
}

impl UI for GtkUI {
    fn run(&self) {
        let app = RelmApp::new(APPLICATION_ID);
        app.run::<GtkApp>((self.os.clone(), self.prompt.clone()));
    }
}

impl GtkUI {
    pub fn new(os: Os, prompt: &str) -> Self {
        GtkUI {
            os: Rc::new(os),
            prompt: prompt.to_string(),
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd)]
struct SearchItem {
    score: i32
}

impl SearchItem {
    pub fn new(score: i32) -> Self {
        SearchItem { score }
    }
}

impl Eq for SearchItem{}

impl Ord for SearchItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.score.cmp(&other.score)
    }
}

impl RelmListItem for SearchItem {
    type Root = gtk::Box;
    type Widgets = ();

    fn setup(list_item: &gtk::ListItem) -> (Self::Root, Self::Widgets) {
        relm4::view!{
            my_box = gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                gtk::Label {
                    set_label: "HI",
                    set_width_chars: 3,
                },
            }
        }

        let widgets = ();
        (my_box, widgets)
    }
}


struct GtkApp {
    os: Rc<Os>,
    search_items: TypedListView<SearchItem, gtk::SingleSelection>,
}

#[derive(Debug)]
pub enum Msg {
    Search,
}

#[relm4::component]
impl SimpleComponent for GtkApp {
    type Init = (Rc<Os>, String);
    type Input = Msg;
    type Output = ();

    view! {
        gtk::Window {
            init_layer_shell: (),
            set_keyboard_mode: gtk4_layer_shell::KeyboardMode::OnDemand,
            set_layer: Layer::Overlay,
            auto_exclusive_zone_enable: (),
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,
                set_margin_all: 5,

                gtk::SearchEntry {
                    set_placeholder_text: Some("Search"),
                    connect_search_changed => Msg::Search,
                },
                #[local_ref]
                search_items_box -> gtk::ListView{
                    set_orientation: gtk::Orientation::Vertical,
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let (os, prompt) = init;

        let search_items: TypedListView<SearchItem, gtk::SingleSelection> = TypedListView::with_sorting();
        let model = GtkApp { os, search_items };

        let search_items_box = &model.search_items.view;
        let widgets = view_output!();


        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            Msg::Search => {
                println!("Search");
            }
        }
    }
}

// impl GtkUI {
//     fn build_ui(app: &gtk::Application, prompt: String) {
//         // Create a button
//         let search_entry = SearchEntry::builder()
//             .placeholder_text("Search")
//             .editable(true)
//             .focus_on_click(true)
//             .build();
//         search_entry.connect_search_changed(||);
//
//         // Create a window
//         let window = ApplicationWindow::builder()
//             .application(app)
//             .title("My GTK App")
//             .child(&search_entry)
//             // .child(&button)
//             .build();
//         window.init_layer_shell();
//         window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::Exclusive);
//         // Display above normal windows
//         window.set_layer(Layer::Overlay);
//
//         // Push other windows out of the way
//         window.auto_exclusive_zone_enable();
//         window.set_margin(Edge::Left, 40);
//         window.set_margin(Edge::Right, 40);
//         window.set_margin(Edge::Top, 20);
//
//         // Present window
//         window.show();
//         search_entry.grab_focus();
//     }
//
//     fn create_app() -> gtk::Application {
//         let application = gtk::Application::new(Some(APPLICATION_ID), Default::default());
//         application
//     }
//
//
// }
