mod imp;

use gtk::glib;
use gtk::subclass::prelude::*;
use relm4::gtk;

glib::wrapper! {
    pub struct RichLabel(ObjectSubclass<imp::RichLabelImpl>)
    @extends gtk::Widget,
    @implements gtk::Accessible;
}

impl RichLabel {
    pub fn new() -> Self {
        glib::Object::new(&[])
    }

    pub fn set_label(&self, text: &str) {
        self.imp().set_label(text);
    }

    pub fn set_attributes(&self, attrs: &gtk::pango::AttrList) {
        self.imp().set_attributes(attrs);
    }

    pub fn set_tags(&self, tags: &[bitsplain::tree::Tag]) {
        self.imp().set_tags(tags);
    }
}
