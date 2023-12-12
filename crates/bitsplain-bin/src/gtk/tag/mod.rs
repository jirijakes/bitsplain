mod imp;

use gtk::glib;
use gtk::subclass::prelude::*;
use relm4::gtk;

glib::wrapper! {
    pub struct Tag(ObjectSubclass<imp::TagImpl>)
    @extends gtk::Widget,
    @implements gtk::Accessible;
}

impl Tag {
    pub fn new(tag: &str) -> Self {
        let obj: Self = glib::Object::new();
        obj.set_tag(tag);
        obj
    }

    pub fn set_tag(&self, text: &str) {
        self.imp().set_tag(text);
    }
}
