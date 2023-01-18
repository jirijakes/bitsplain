use std::rc::Rc;

use bitsplain::annotations::Annotations;
use bitsplain::*;
use gtk::prelude::*;

use relm4::*;

use crate::hexy::HexyLook;

pub enum HexyModel {
    Empty,
    Full {
        annotations: Rc<Annotations>,
        bytes: Rc<Vec<u8>>,
        selection: Option<(u32, u32)>,
    },
}

#[derive(Debug)]
pub enum HexyMsg {
    Open {
        annotations: Rc<Annotations>,
        bytes: Rc<Vec<u8>>,
    },
    Select(u32, u32),
    Unselect,
}

#[relm4::component(pub)]
impl Component for HexyModel {
    type CommandOutput = ();
    type Input = HexyMsg;
    type Output = ();
    type Init = ();

    fn init(
        _parent_model: Self::Init,
        root: &Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = HexyModel::Empty;
        let widgets = view_output!();

        ComponentParts { widgets, model }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>, hexy: &Self::Root) {
        match self {
            HexyModel::Full { selection, .. } => {
                match msg {
                    HexyMsg::Select(from, to) => {
                        *selection = Some((from, to));
                    }
                    HexyMsg::Unselect => {
                        *selection = None;
                    }
                    HexyMsg::Open { annotations, bytes } => {
                        hexy.clear();
                        let h = bytes.as_ref();

                        annotations
                            .as_ref()
                            .leaves()
                            .iter()
                            .enumerate()
                            .for_each(|(idx, leaf)| {
                                hexy.add_group(idx, &h[leaf.location.from..leaf.location.to]);
                                // send!(sender, "");
                            });
                        *self = HexyModel::Full {
                            annotations,
                            bytes,
                            selection: None,
                        }
                    }
                };
            }
            HexyModel::Empty => {
                if let HexyMsg::Open { annotations, bytes } = msg {
                    hexy.clear();
                    let h = bytes.as_ref();

                    annotations
                        .as_ref()
                        .leaves()
                        .iter()
                        .enumerate()
                        .for_each(|(idx, leaf)| {
                            hexy.add_group(idx, &h[leaf.location.from..leaf.location.to]);
                            // send!(sender, "");
                        });
                    *self = HexyModel::Full {
                        annotations,
                        bytes,
                        selection: None,
                    };
                }
            }
        }
    }

    #[rustfmt::skip]
    view! {
	#[name = "hexy"]
	HexyLook {
            set_hexpand: true,
            set_vexpand: true
	}
    }

    fn post_view() {
        match model {
            HexyModel::Full {
                selection: Some((from, to)),
                ..
            } => {
                widgets.hexy.highlight(*from, *to);
            }
            _ => {
                widgets.hexy.no_highlight();
            }
        }
    }
}
