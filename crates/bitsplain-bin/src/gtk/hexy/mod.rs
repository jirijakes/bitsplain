mod imp;

use gtk::glib;
use gtk::subclass::prelude::*;
use relm4::gtk;

glib::wrapper! {
    pub struct HexyLook(ObjectSubclass<imp::HexyLookImpl>)
    @extends gtk::Widget,
    @implements gtk::Accessible;
}

impl HexyLook {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn add_group(&self, index: usize, bytes: &[u8]) {
        self.imp().add_group(index, bytes);
    }

    pub fn clear(&self) {
        self.imp().clear();
    }

    fn moving(&self, x: f64, y: f64) {
        self.imp().moving(x, y)
    }

    pub fn highlight(&self, from: u32, to: u32) {
        self.imp().highlight(from, to);
    }

    pub fn no_highlight(&self) {
        self.imp().no_highlight();
    }
}

impl Default for HexyLook {
    fn default() -> Self {
        HexyLook::new()
    }
}
