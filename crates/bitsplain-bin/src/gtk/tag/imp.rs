use std::cell::RefCell;

use gtk::glib::clone;
use gtk::prelude::WidgetExt;
use gtk::subclass::prelude::*;
use gtk::{gdk, glib};
use relm4::gtk;
use relm4::gtk::traits::BoxExt;

#[derive(Debug)]
pub struct TagImpl {
    tag: gtk::Label,
}

impl Default for TagImpl {
    fn default() -> Self {
        Self {
            tag: gtk::Label::new(None),
        }
    }
}

impl TagImpl {
    pub fn set_tag(&self, text: &str) {
        self.tag.set_label(text);
    }
}

#[glib::object_subclass]
impl ObjectSubclass for TagImpl {
    const NAME: &'static str = "Tag";
    type Type = super::Tag;
    type ParentType = gtk::Widget;

    fn class_init(klass: &mut Self::Class) {
        klass.set_layout_manager_type::<gtk::BoxLayout>();
        klass.set_css_name("tag");
    }
}

impl ObjectImpl for TagImpl {
    fn constructed(&self) {
        self.parent_constructed();

        let obj = self.obj();

        // obj.style_context().add_class("hex");
        let provider = gtk::CssProvider::new();
        provider.load_from_data(include_str!("tag.css"));
        if let Some(display) = &gdk::Display::default() {
            gtk::style_context_add_provider_for_display(
                display,
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            )
        };
        self.tag.add_css_class("tag");
        self.tag.set_parent(&*obj);
    }

    fn dispose(&self) {
        self.tag.unparent();
    }
}

impl WidgetImpl for TagImpl {}
