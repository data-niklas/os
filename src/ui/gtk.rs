use std::rc::Rc;

use crate::model::SearchItem;
use crate::os::Os;
use crate::ui::UI;
use crate::APPLICATION_ID;
use gtk::prelude::*;
use gtk::SearchEntry;
#[cfg(feature = "wayland")]
use gtk4_layer_shell::{Edge, Layer, LayerShell};

use relm4::gtk::gdk::Key;

use relm4::gtk::glib::Propagation;

use relm4::gtk::{Align, EventControllerKey, Justification, PolicyType};
use relm4::{
    prelude::*,
    typed_view::list::{RelmListItem, TypedListView},
};
use std::cell::RefCell;
pub struct GtkUI {
    os: Rc<RefCell<Os>>,
}

impl UI for GtkUI {
    fn run(&mut self) {
        let app = RelmApp::new(APPLICATION_ID).with_args(vec![]);
        app.run::<GtkApp>(self.os.clone());
    }
}

impl GtkUI {
    pub fn new(os: Os) -> Self {
        GtkUI {
            os: Rc::new(RefCell::new(os)),
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

    fn bind(&mut self, widgets: &mut Self::Widgets, _root: &mut Self::Root) {
        widgets.title.set_visible(self.title.is_some());
        widgets.image.set_visible(self.image.is_some());
        widgets.subtitle.set_visible(self.subtitle.is_some());

        if let Some(title) = &self.title {
            // TODO: use variable from ui.gtk.char width
            let title: String = title.chars().take(50).collect();
            widgets.title.set_label(&title);
        }
        if let Some(subtitle) = &self.subtitle {
            let subtitle: String = subtitle.chars().take(50).collect();
            widgets.subtitle.set_label(&subtitle);
        }

        if self.icon.is_some() {
            let image = self.icon.as_ref().unwrap();
            let raw_bytes = image.as_raw().as_ref();
            let image_bytes =
                relm4::gtk::glib::Bytes::from_static(unsafe { std::mem::transmute(raw_bytes) });
            let width: i32 = image.width().try_into().unwrap();
            let height: i32 = image.height().try_into().unwrap();
            // 4 due to RGBA times width
            let stride: i32 = width * 4;
            let pixbuf = relm4::gtk::gdk_pixbuf::Pixbuf::from_bytes(
                &image_bytes,
                gtk::gdk_pixbuf::Colorspace::Rgb,
                true,
                8,
                width,
                height,
                stride,
            );
            widgets.icon.set_from_pixbuf(Some(&pixbuf));
        }
        if self.image.is_some() {
            let image = self.image.as_ref().unwrap();
            let raw_bytes = image.as_raw().as_ref();
            let image_bytes =
                relm4::gtk::glib::Bytes::from_static(unsafe { std::mem::transmute(raw_bytes) });
            let width: i32 = image.width().try_into().unwrap();
            let height: i32 = image.height().try_into().unwrap();
            // 4 due to RGBA times width
            let stride: i32 = width * 4;
            let pixbuf = relm4::gtk::gdk_pixbuf::Pixbuf::from_bytes(
                &image_bytes,
                gtk::gdk_pixbuf::Colorspace::Rgb,
                true,
                8,
                width,
                height,
                stride,
            );
            widgets.image.set_from_pixbuf(Some(&pixbuf));
        }
    }

    fn setup(_list_item: &gtk::ListItem) -> (Self::Root, Self::Widgets) {
        let attr_list = relm4::gtk::pango::AttrList::new();
        let attr = relm4::gtk::pango::AttrFloat::new_scale(0.8);
        attr_list.insert(attr);
        relm4::view! {
            my_box = gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_hexpand: true,
                set_halign: Align::Start,
                set_margin_all: 5,
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
                        set_wrap: false,
                        set_lines: 1
                    },
                    #[name="image"]
                    gtk::Image {
                        set_size_request: (250, 150),
                    },
                    #[name="subtitle"]
                    gtk::Label {
                        set_justify: Justification::Left,
                        set_hexpand: true,
                        set_halign: Align::Start,
                        set_wrap: false,
                        set_attributes: Some({
                            &attr_list
                        }),
                        set_lines: 1
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
        let search_items: Vec<_> = self
            .os
            .borrow()
            .search(query)
            .into_iter()
            .take(self.os.borrow().config.maximum_list_item_count)
            .collect();
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
    type Init = Rc<RefCell<Os>>;
    type Input = Msg;
    type Output = ();

    view! {
        #[name="window"]
        gtk::Window {

            set_default_size: (400, 400),
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,
                set_margin_all: 5,

                #[name="search_entry"]
                gtk::SearchEntry {
                    connect_search_changed[sender] =>
                        move |it: &SearchEntry|{
                            let message = Msg::Search(it.text().to_string());
                            sender.input(message);

                    },
                    connect_activate => Msg::Select,
                    add_controller: {
                        let sender2 = sender.clone();
                        let keys = EventControllerKey::new();
                        keys.connect_key_pressed(move |_, keyval, _keycode, _state| {
                            match keyval {
                                Key::Escape => {
                                    os_for_key_pressed.borrow_mut().deinit();
                                    std::process::exit(0);
                                },
                                Key::Up => {
                                    sender2.input(Msg::Up);
                                    Propagation::Stop
                                },
                                Key::Down => {
                                    sender2.input(Msg::Down);
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
                    set_vexpand: true,
                    #[local_ref]
                    search_items_box -> gtk::ListView{
                        connect_activate[sender] => move|_a,_b|{
                            sender.input(Msg::Select);
                        },
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
        let os = init;
        let prompt = os.borrow().config.prompt.clone();
        let initial_search: bool = os.borrow().config.initial_search;
        let os_for_key_pressed = os.clone();
        let search_items: TypedListView<SearchItem, gtk::SingleSelection> = TypedListView::new();
        let search_items_box = &search_items.view;
        let widgets = view_output!();
        #[cfg(feature = "wayland")]
        {
            if os.borrow().config.wayland_layer {
                widgets.window.init_layer_shell();
                widgets
                    .window
                    .set_keyboard_mode(gtk4_layer_shell::KeyboardMode::OnDemand);
                widgets.window.set_layer(Layer::Overlay);
                widgets.window.auto_exclusive_zone_enable();
            }
        };
        let search_entry = widgets.search_entry.clone();
        search_entry.set_placeholder_text(Some(&prompt));
        let mut model = GtkApp {
            os,
            search_items,
            search_entry,
        };
        if initial_search {
            model.search("");
        }
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
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
                let item = {
                    let selection = &self.search_items.selection_model;
                    let selected = selection.selected();
                    if selected >= selection.n_items() {
                        return;
                    }
                    self.search_items.get(selected).unwrap()
                };
                let mut os_borrow = self.os.borrow_mut();
                if os_borrow.select(&item.borrow()) {
                    os_borrow.deinit();
                    std::process::exit(0);
                } else {
                    self.search_entry.set_text("");
                    self.search_items.clear();
                }
            }
        }
    }
}
