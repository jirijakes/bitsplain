use std::rc::Rc;

use bitsplain::tree::Information;
use bitsplain::value::Value;
use gtk::prelude::*;
use relm4::prelude::*;

pub struct DocModel {
    title: String,
    subtitle: Option<String>,
    data_type: Option<String>,
    doc: Option<String>,
    value: Option<Value>,
    bytes: Rc<Vec<u8>>,
    range: Option<(usize, usize)>,
}

#[derive(Debug)]
pub enum DocMsg {
    T(Option<(usize, usize)>, Information),
}

#[relm4::component(pub)]
impl SimpleComponent for DocModel {
    type Init = ();
    type Input = DocMsg;
    type Widgets = DocWidgets;
    type Output = ();

    fn init(
        parent_model: Self::Init,
        root: &Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = DocModel {
            title: "Serialized Bitcoin transaction".to_owned(),
            subtitle: None,
            data_type: None,
            doc: None,
            value: None,
            bytes: Rc::new(vec![]),
            range: None,
        };
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            DocMsg::T(range, s) => {
                self.subtitle = Some(s.label.to_string());
                self.data_type = s.data.get("datatype").cloned();
                self.doc = s.doc;
                self.value = Some(s.value);
                self.range = range;
            }
        }
    }

    #[rustfmt::skip]
    view! {
	gtk::Box {
	    set_orientation: gtk::Orientation::Vertical,
	    add_css_class: "doc",
            append = &gtk::Label::new(Some(&model.title)) {
		add_css_class: "title"
	    },
	    append: subtitle = &gtk::Label {
		set_label: "",
		add_css_class: "subtitle"
	    },
	    append = &gtk::Grid {
		set_vexpand: true,
		set_hexpand: true,
		set_row_spacing: 10,
		set_column_spacing: 10,
		set_column_homogeneous: false,
		attach[1, 1, 1, 1] = &gtk::Label {
		    set_label: "Name",
		    set_width_request: 50,
		    set_xalign: 1.0,
		    set_yalign: 0.8,
		    add_css_class: "label"
		},
		attach[2, 1, 1, 1] = &gtk::Label {
		    #[watch] set_label: model.subtitle.as_deref().unwrap_or("") ,
		    set_hexpand: true,
		    set_xalign: 0.0,
		    add_css_class: "value"
		},
		attach[1, 2, 1, 1] = &gtk::Label {
		    set_label: "Data type",
		    set_width_request: 50,
		    set_xalign: 1.0,
		    set_yalign: 0.8,
		    add_css_class: "label"
		},
		attach[2, 2, 1, 1] = &gtk::Label {
		    #[watch] set_label: model.data_type.as_deref().unwrap_or("") ,
		    set_hexpand: true,
		    set_xalign: 0.0,
		    add_css_class: "value"
		},
		attach[1, 3, 1, 1] = &gtk::Label {
		    set_label: "Value",
		    set_width_request: 50,
		    set_xalign: 1.0,
		    set_yalign: 0.8,
		    add_css_class: "label"
		},
		attach[2, 3, 1, 1] = &gtk::Label {
		    #[watch] set_label: &model.value.as_ref().map(|v| v.preview()).unwrap_or_else(String::new) ,
		    set_hexpand: true,
		    set_xalign: 0.0,
		    set_wrap_mode: gtk::pango::WrapMode::Char,
		    set_wrap: true,
		    add_css_class: "value"
		},
		attach[1, 4, 1, 1] = &gtk::Label {
		    set_label: "Length",
		    set_width_request: 50,
		    set_xalign: 1.0,
		    set_yalign: 0.8,
		    add_css_class: "label"
		},
		attach[2, 4, 1, 1] = &gtk::Label {
		    #[watch] set_label: &model.range.as_ref().map(|(f, t)| format!("{}", t - f)).unwrap_or_else(String::new) ,
		    set_hexpand: true,
		    set_xalign: 0.0,
		    add_css_class: "value"
		},
		attach[1, 5, 1, 1] = &gtk::Label {
		    set_label: "Doc",
		    set_width_request: 50,
		    set_xalign: 1.0,
		    set_yalign: 0.8,
		    add_css_class: "label"
		},
		attach[2, 5, 1, 1] = &gtk::Label {
		    #[watch] set_label: model.doc.as_deref().unwrap_or("") ,
		    set_hexpand: true,
		    set_xalign: 0.0,
		    add_css_class: "value"
		},
	    }
	}
    }
}
