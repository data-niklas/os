use std::rc::Rc;

use crate::model::SearchItem;
use crate::os::Os;
use crate::ui::UI;
use crate::APPLICATION_ID;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Button, Entry, SearchEntry};
use gtk4_layer_shell::{Edge, Layer, LayerShell};
use relm4::gtk::cairo::FontOptions;
use relm4::gtk::ffi::{GtkBox, GtkImage};
use relm4::gtk::gdk::Key;
use relm4::gtk::glib::Propagation;
use relm4::gtk::pango::ffi::{PangoAttrFontDesc, PangoFontDescription};
use relm4::gtk::pango::FontScale;
use relm4::gtk::{
    Align, EventControllerKey, Justification, Label, ListView, Overflow, PolicyType,
    ScrollablePolicy, ScrolledWindow,
};
use relm4::{
    factory::FactoryVecDeque,
    gtk::{gdk::KeyEvent, gdk_pixbuf::Pixbuf, glib::RustClosure, Window},
    prelude::*,
    typed_view::list::{RelmListItem, TypedListView},
};
use std::cell::RefCell;
pub struct GtkUI {
    os: Rc<RefCell<Os>>,
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
            os: Rc::new(RefCell::new(os)),
            prompt: prompt.to_string(),
        }
    }
}

pub struct SearchItemWidgets {
    icon: gtk::Image,
    title: gtk::Label,
    image: gtk::Image,
    subtitle: gtk::Label,
}

impl RelmListItem for SearchItem {
    type Root = gtk::Box;
    type Widgets = SearchItemWidgets;

    fn bind(&mut self, widgets: &mut Self::Widgets, root: &mut Self::Root) {
        // widgets.icon.set_visible(self.icon.is_some());
        widgets.title.set_visible(self.title.is_some());
        widgets.image.set_visible(self.image.is_some());
        widgets.subtitle.set_visible(self.subtitle.is_some());

        if let Some(title) = &self.title {
            widgets.title.set_label(title);
        }
        if let Some(subtitle) = &self.subtitle {
            widgets.subtitle.set_label(subtitle);
        }

        if self.icon.is_some() {
            let image = self.icon.as_ref().unwrap();
            let rgb8_image = image.to_rgba8();
            let (width, height) = rgb8_image.dimensions();
            // Convert DynamicImage to Pixbuf
            let pixbuf = Pixbuf::from_mut_slice(
                rgb8_image.into_raw(),
                gtk::gdk_pixbuf::Colorspace::Rgb,
                true, // Has alpha channel
                8,    // Bits per sample
                width as i32,
                height as i32,
                width as i32 * 4, // Row stride (4 bytes per pixel)
            );
            widgets.icon.set_from_pixbuf(Some(&pixbuf));
        }
    }

    fn setup(list_item: &gtk::ListItem) -> (Self::Root, Self::Widgets) {
        let attr_list = relm4::gtk::pango::AttrList::new();
        let mut attr = relm4::gtk::pango::AttrFloat::new_scale(0.8);
        attr_list.insert(attr);
        relm4::view! {
            my_box = gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                    set_hexpand: true,
                set_halign: Align::Start,
                #[name="icon"]
                gtk::Image {
                    set_vexpand: true,
                    set_icon_size: gtk::IconSize::Large,
                    set_margin_end: 6,
                },
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_halign: Align::Start,
                    set_valign: Align::Start,
                    // set_height_request: 50,
                    #[name="title"]
                    gtk::Label {
                        set_justify: Justification::Left,
                        set_hexpand: true,
                        set_halign: Align::Start,
                    },
                    #[name="image"]
                    gtk::Image {
                    },
                    #[name="subtitle"]
                    gtk::Label {
                        set_justify: Justification::Left,
                        set_hexpand: true,
                        set_halign: Align::Start,
                        set_attributes: Some({


                            &attr_list
                        }),
                    },
                },
            }
        }

        let widgets = SearchItemWidgets {
            icon,
            title,
            image,
            subtitle,
        };
        (my_box, widgets)
    }
}

struct GtkApp {
    os: Rc<RefCell<Os>>,
    search_items: TypedListView<SearchItem, gtk::SingleSelection>,
    search_entry: SearchEntry,
}

impl GtkApp {
    pub fn search(&mut self, query: &str) {
        let search_items = self.os.borrow().search(query);
        self.search_items.clear();
        self.search_items.extend_from_iter(search_items);
    }
}

#[derive(Debug)]
pub enum Msg {
    Search(String),
    Up,
    Down,
    Select,
}

#[relm4::component]
impl SimpleComponent for GtkApp {
    type Init = (Rc<RefCell<Os>>, String);
    type Input = Msg;
    type Output = ();

    view! {
        gtk::Window {
            init_layer_shell: (),
            set_keyboard_mode: gtk4_layer_shell::KeyboardMode::OnDemand,
            set_layer: Layer::Overlay,
            auto_exclusive_zone_enable: (),

            set_default_size: (400, 400),
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,
                set_margin_all: 5,

                #[name="search_entry"]
                gtk::SearchEntry {
                    set_placeholder_text: Some("Search"),
                    connect_search_changed[sender] =>
                        move |it: &SearchEntry|{
                            let message = Msg::Search(it.text().to_string());
                            sender.input(message);

                    },
                    connect_activate => Msg::Select,
                    add_controller: {
                        let keys = EventControllerKey::new();
                        keys.connect_key_pressed(move |_, keyval, keycode, state| {
                            match keyval {
                                Key::Escape => {
                                    os_for_key_pressed.borrow_mut().run_select_action(crate::model::SelectAction::Exit);
                                    Propagation::Stop
                                },
                                Key::Up => {
                                    sender.input(Msg::Up);
                                    Propagation::Stop
                                },
                                Key::Down => {
                                    sender.input(Msg::Down);
                                    Propagation::Stop
                                },
                                _ => Propagation::Proceed,
                            }
                        });
                        keys
                    }
                },

                #[name="scroll_items"]
                gtk::ScrolledWindow {
                    set_policy: (PolicyType::Automatic, PolicyType::Automatic),
                    set_size_request: (350, 650),
                    #[local_ref]
                    search_items_box -> gtk::ListView{
                        set_vexpand: false,
                        set_orientation: gtk::Orientation::Vertical,
                        set_vscroll_policy: gtk::ScrollablePolicy::Natural,
                    }
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
        let os_for_key_pressed = os.clone();
        let search_items: TypedListView<SearchItem, gtk::SingleSelection> =
            TypedListView::new();
        let search_items_box = &search_items.view;
        let widgets = view_output!();
        let search_entry = widgets.search_entry.clone();
        let mut model = GtkApp {
            os,
            search_items,
            search_entry,
        };
        model.search("");
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            Msg::Search(query) => {
                self.search(&query);
            }
            Msg::Up => {
                self.search_items.view.grab_focus();
                let selection = &self.search_items.selection_model;
                let current = selection.selected();
                if current > 0 {
                    selection.select_item(current - 1, true);
                }
                self.search_items
                    .view
                    .emit_move_focus(gtk::DirectionType::Up);
                self.search_entry.grab_focus();
            }
            Msg::Down => {
                self.search_items.view.grab_focus();
                let selection = &self.search_items.selection_model;
                let current = selection.selected();
                if current < selection.n_items() - 1 {
                    selection.select_item(current + 1, true);
                }
                self.search_items
                    .view
                    .emit_move_focus(gtk::DirectionType::Down);
                self.search_entry.grab_focus();
            }
            Msg::Select => {
                let selection = &self.search_items.selection_model;
                let selected = selection.selected();
                if selected >= selection.n_items() {
                    return;
                }
                let selected = self.search_items.get(selected).unwrap();
                let item = selected.borrow();
                self.os.borrow_mut().select(&item);
            }
        }
    }
}
