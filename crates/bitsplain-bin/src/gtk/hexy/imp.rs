use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use gtk::glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, glib};
use lazy_static::lazy_static;
use relm4::gtk;

#[derive(Debug, PartialEq, Eq)]
struct Highlighted(u32, u32);

#[derive(Debug)]
pub struct HexyLookImpl {
    offsets: RefCell<gtk::TextView>,
    hexview: RefCell<gtk::TextView>,
    scroll: gtk::ScrolledWindow,
    bx: gtk::Box,
    /// Bytes already inserted.
    size: Cell<usize>,
    /// Bytes per line.
    width: usize,
    highlighted: RefCell<Option<Highlighted>>,
    tags: RefCell<HashMap<u32, gtk::TextTag>>,
}

impl Default for HexyLookImpl {
    fn default() -> Self {
        Self {
            offsets: Default::default(),
            hexview: Default::default(),
            scroll: gtk::ScrolledWindow::new(),
            bx: gtk::Box::new(gtk::Orientation::Horizontal, 0),
            size: Default::default(),
            width: 24,
            highlighted: Default::default(),
            tags: Default::default(),
        }
    }
}

lazy_static! {
    static ref THEME: Vec<gdk::RGBA> = vec![
        gdk::RGBA::parse("#8be9fd").unwrap(),
        gdk::RGBA::parse("#ffb86c").unwrap(),
        gdk::RGBA::parse("#50fa7b").unwrap(),
        gdk::RGBA::parse("#ff79c6").unwrap(),
        gdk::RGBA::parse("#bd93f9").unwrap(),
        gdk::RGBA::parse("#ff5555").unwrap(),
        gdk::RGBA::parse("#f1fa8c").unwrap(),
    ];
    static ref THEME_SIZE: usize = THEME.len();
}

impl HexyLookImpl {
    pub(super) fn clear(&self) {
        self.hexview.borrow().buffer().set_text("");
        self.offsets.borrow().buffer().set_text("");
        self.size.set(0);
        self.tags.borrow_mut().clear();
        *self.highlighted.borrow_mut() = None;
    }

    pub(super) fn add_group(&self, index: usize, bytes: &[u8]) {
        let view_buffer = self.hexview.borrow().buffer();
        let start = view_buffer.end_iter().offset();

        let mut buf = bytes;

        // Insert `bytes` and make sure to also insert newlines
        // if the bytes need more lines.
        while !buf.is_empty() {
            let size = self.size.get();

            // Amount of bytes that can still be inserted on the line.
            let available = self.width - size % self.width;

            let (current, rest) = buf.split_at(available.min(buf.len()));
            buf = rest;

            view_buffer.insert_at_cursor(&hex::encode(current));
            self.update_offsets();

            // Update amount of already-inserted bytes.
            self.size.set(size + current.len());

            // If the data do not fit or if just-inserted data
            // ended right at the end of line, we have to insert a new line.
            if !rest.is_empty() || self.size.get() % self.width == 0 {
                view_buffer.insert_at_cursor("\n");
            }
        }

        let color = unsafe { THEME.get_unchecked(index % *THEME_SIZE) };

        let tag_group = view_buffer
            .create_tag(None, &[("foreground-rgba", color)])
            .unwrap();
        unsafe {
            tag_group.set_data("index", index);
        }

        let start = view_buffer.iter_at_offset(start);
        let end = view_buffer.end_iter();
        view_buffer.apply_tag(&tag_group, &start, &end);

        self.tags.borrow_mut().insert(index as u32, tag_group);
    }

    fn update_offsets(&self) {
        let offsets_buffer = self.offsets.borrow().buffer();
        let offsets_lines = offsets_buffer.line_count();
        let view_buffer = self.hexview.borrow().buffer();
        let view_lines = view_buffer.line_count();

        let width = self.width as i32;

        if view_lines >= offsets_lines {
            for i in offsets_lines - 1..view_lines {
                offsets_buffer.insert_at_cursor(&format!("{:06}\n", i * width));
            }
        }
    }

    fn index_at_location(&self, x: i32, y: i32) -> Option<u32> {
        let view = self.hexview.borrow();
        let iter = view.iter_at_location(x, y)?;
        let index = iter
            .tags()
            .iter()
            .find_map(|tag| unsafe { tag.data::<u32>("index") })?;
        let index = unsafe { *index.as_ref() };

        Some(index)
    }

    fn reset_tag(&self, index: u32) {
        if let Some(tag) = self.tags.borrow().get(&index) {
            let color = unsafe { THEME.get_unchecked((index as usize) % *THEME_SIZE) };
            tag.set_background(None);
            tag.set_foreground_rgba(Some(color));
        }
    }

    fn highlight2(&self, index: u32) {
        if let Some(tag) = self.tags.borrow().get(&index) {
            let color = unsafe { THEME.get_unchecked((index as usize) % *THEME_SIZE) };
            tag.set_background_rgba(Some(color));
            tag.set_foreground(Some("black"));
        }
    }

    pub(super) fn moving(&self, x: f64, y: f64) {
        let view = self.hexview.borrow();
        let (x, y) = view.window_to_buffer_coords(gtk::TextWindowType::Widget, x as i32, y as i32);

        if let Some(index) = self.index_at_location(x, y) {
            self.highlight(index, index);
        } else {
            self.no_highlight();
        }
    }

    pub(super) fn no_highlight(&self) {
        let mut hl = self.highlighted.borrow_mut();
        if let Some(Highlighted(min, max)) = hl.as_ref() {
            for i in *min..*max + 1 {
                self.reset_tag(i);
            }
            let _ = hl.take();
        }
    }

    pub(super) fn highlight(&self, from: u32, to: u32) {
        let mut hl = self.highlighted.borrow_mut();
        match hl.as_ref() {
            Some(Highlighted(min, max)) if *min == from && *max == to => {}
            Some(Highlighted(min, max)) => {
                for i in *min..*max + 1 {
                    self.reset_tag(i);
                }
                for i in from..to + 1 {
                    self.highlight2(i);
                }
                let _ = hl.insert(Highlighted(from, to));
            }
            _ => {
                for i in from..to + 1 {
                    self.highlight2(i);
                }
                let _ = hl.insert(Highlighted(from, to));
            }
        }
    }
}

#[glib::object_subclass]
impl ObjectSubclass for HexyLookImpl {
    const NAME: &'static str = "HexyLook";
    type Type = super::HexyLook;
    type ParentType = gtk::Widget;

    fn class_init(klass: &mut Self::Class) {
        klass.set_layout_manager_type::<gtk::BoxLayout>();
        klass.set_css_name("hexy");
    }
}

impl ObjectImpl for HexyLookImpl {
    fn constructed(&self) {
        self.parent_constructed();

        let obj = self.obj();

        obj.style_context().add_class("hex");
        let provider = gtk::CssProvider::new();
        provider.load_from_data(include_str!("hexy.css"));
        if let Some(display) = &gdk::Display::default() {
            gtk::style_context_add_provider_for_display(
                display,
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            )
        };
        let offsets = gtk::TextView::new();
        offsets.set_widget_name("offsets");
        offsets.set_width_request(110);
        offsets.set_vexpand(true);
        offsets.set_editable(false);
        offsets.set_cursor_visible(false);

        offsets.set_parent(&self.bx);
        *self.offsets.borrow_mut() = offsets;

        let view = gtk::TextView::new();
        view.set_widget_name("view");
        view.set_vexpand(true);
        view.set_hexpand(true);
        view.set_editable(false);
        view.set_cursor_visible(false);

        let motion_controller = gtk::EventControllerMotion::new();
        motion_controller.connect_motion(clone!(@weak obj => move |_, x, y| obj.moving(x,y)));
        // motion_controller.connect_motion(
        // clone!(@weak view => move |_, x, y| println!("{x} {y} {:?}", view.buffer())),
        // );

        view.add_controller(motion_controller);
        view.set_parent(&self.bx);
        *self.hexview.borrow_mut() = view;

        self.scroll.set_child(Some(&self.bx));
        self.scroll.set_parent(&*obj);
    }

    fn dispose(&self) {
        self.hexview.borrow_mut().unparent();
        self.offsets.borrow_mut().unparent();
        self.bx.unparent();
        self.scroll.unparent();
    }
}

impl WidgetImpl for HexyLookImpl {}
