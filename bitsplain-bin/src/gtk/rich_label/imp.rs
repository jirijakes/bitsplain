use std::cell::RefCell;

use gtk::glib::clone;
use gtk::prelude::WidgetExt;
use gtk::subclass::prelude::*;
use gtk::{gdk, glib};
use relm4::gtk::traits::BoxExt;
use relm4::{gtk, RelmRemoveAllExt};

use crate::tag::Tag;

#[derive(Debug)]
pub struct RichLabelImpl {
    wrap: gtk::Box,
    name: gtk::Label,
    tags_wrap: gtk::Box,
}

impl Default for RichLabelImpl {
    fn default() -> Self {
        let name = gtk::Label::new(None);
        let wrap = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(10)
            .build();
        let tags_wrap = gtk::Box::new(gtk::Orientation::Horizontal, 3);

        Self {
            name,
            wrap,
            tags_wrap,
        }
    }
}

impl RichLabelImpl {
    pub fn set_label(&self, text: &str) {
        self.name.set_label(text);
    }

    pub fn set_attributes(&self, attrs: &gtk::pango::AttrList) {
        self.name.set_attributes(Some(&attrs));
    }

    pub fn set_tags(&self, tags: &[bitsplain::tree::Tag]) {
        self.tags_wrap.remove_all();
        tags.iter().for_each(|t| {
            let tag = Tag::new(&t.label);
            self.tags_wrap.append(&tag);
        });
    }
}

#[glib::object_subclass]
impl ObjectSubclass for RichLabelImpl {
    const NAME: &'static str = "RichLabel";
    type Type = super::RichLabel;
    type ParentType = gtk::Widget;

    fn class_init(klass: &mut Self::Class) {
        klass.set_layout_manager_type::<gtk::BoxLayout>();
        klass.set_css_name("richlabel");
    }
}

impl ObjectImpl for RichLabelImpl {
    fn constructed(&self) {
        self.parent_constructed();

        let obj = self.obj();

        let provider = gtk::CssProvider::new();
        provider.load_from_data(include_bytes!("richlabel.css"));
        gtk::StyleContext::add_provider_for_display(
            &gdk::Display::default().unwrap(),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        self.wrap.append(&self.name);
        self.wrap.append(&self.tags_wrap);
        self.wrap.set_parent(&*obj);
    }

    fn dispose(&self) {
        self.wrap.unparent();
    }
}

impl WidgetImpl for RichLabelImpl {}
