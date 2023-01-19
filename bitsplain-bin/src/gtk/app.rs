use std::convert::identity;
use std::rc::Rc;

use bitsplain::decode::{decode_input, Input};
use bitsplain::tree::*;
use gtk::glib::clone;
use gtk::prelude::*;
use gtk::{gdk, gio};
use relm4::actions::{AccelsPlus, RelmAction, RelmActionGroup};
use relm4::prelude::*;

use crate::comp::doc::*;
use crate::comp::hexy::*;
use crate::comp::tree::*;

pub enum AppModel {
    Empty {
        doc: Rc<Controller<DocModel>>,
        tree: Rc<Controller<TreeModel>>,
        hexy: Rc<Controller<HexyModel>>,
    },
    Full {
        annotations: Rc<Annotations>,
        doc: Rc<Controller<DocModel>>,
        tree: Rc<Controller<TreeModel>>,
        hexy: Rc<Controller<HexyModel>>,
    },
}

impl AppModel {
    fn hexy(&self) -> &Rc<Controller<HexyModel>> {
        match self {
            AppModel::Empty { hexy, .. } => hexy,
            AppModel::Full { hexy, .. } => hexy,
        }
    }
    fn doc(&self) -> &Rc<Controller<DocModel>> {
        match self {
            AppModel::Empty { doc, .. } => doc,
            AppModel::Full { doc, .. } => doc,
        }
    }
    fn tree(&self) -> &Rc<Controller<TreeModel>> {
        match self {
            AppModel::Empty { tree, .. } => tree,
            AppModel::Full { tree, .. } => tree,
        }
    }
}

#[derive(Debug)]
pub enum AppMsg {
    Select(Vec<String>),
    Open(String),
    Unselect,
    Paste,
    Quit,
}

#[relm4::component(pub)]
impl SimpleComponent for AppModel {
    type Init = Option<String>;
    type Input = AppMsg;
    type Output = ();

    #[rustfmt::skip]
    view! {
	main_window = gtk::ApplicationWindow {
            set_title: Some("Bitsplain"),
	    #[wrap(Some)]
            set_child = &gtk::Paned::new(gtk::Orientation::Horizontal) {
		#[wrap(Some)]
		set_start_child = &gtk::Paned::new(gtk::Orientation::Vertical) {
		    set_start_child: Some(model.hexy().widget()),
		    set_end_child: Some(model.doc().widget())
		},
		set_end_child: Some(model.tree().widget())
            }
	}
    }

    fn init(
        candidate: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let doc: Controller<DocModel> = DocModel::builder().launch(()).detach();
        let tree: Controller<TreeModel> = TreeModel::builder()
            .launch(())
            .forward(sender.input_sender(), identity);
        let hexy: Controller<HexyModel> = HexyModel::builder().launch(()).detach();

        let model = AppModel::Empty {
            doc: Rc::new(doc),
            tree: Rc::new(tree),
            hexy: Rc::new(hexy),
        };
        let widgets = view_output!();

        let settings = gtk::Settings::default().unwrap();
        settings.set_gtk_application_prefer_dark_theme(true);

        let app = relm4::main_application();

        app.set_accelerators_for_action::<crate::QuitAction>(&["<primary>Q"]);
        app.set_accelerators_for_action::<crate::PasteAction>(&["<primary>V"]);

        let win = RelmActionGroup::<crate::WindowActionGroup>::new();
        let quit: RelmAction<crate::QuitAction> = RelmAction::new_stateless(
            clone!(@strong sender => move |_| sender.input(AppMsg::Quit)),
        );
        let paste: RelmAction<crate::PasteAction> = RelmAction::new_stateless(
            clone!(@strong sender => move |_| sender.input(AppMsg::Paste)),
        );

        win.add_action(&quit);
        win.add_action(&paste);

        widgets
            .main_window
            .insert_action_group("win", Some(&win.into_action_group()));

        widgets.main_window.set_show_menubar(true);

        if let Some(s) = candidate {
            sender.input(AppMsg::Open(s));
        }

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            AppMsg::Open(s) => {
                let candidates = decode_input(Input::String(s));

                if let Some(c) = candidates.into_iter().next() {
                    let annotations = Rc::new(c.annotations);
                    let bytes = Rc::new(c.data.to_vec());

                    *self = AppModel::Full {
                        annotations: annotations.clone(),
                        doc: self.doc().clone(),
                        hexy: self.hexy().clone(),
                        tree: self.tree().clone(),
                    };

                    self.tree().emit(TreeMsg::Open {
                        annotations: annotations.clone(),
                    });
                    self.hexy().emit(HexyMsg::Open { annotations, bytes });
                }
            }
            AppMsg::Select(path) => {
                if let AppModel::Full {
                    ref annotations, ..
                } = self
                {
                    match annotations.select(&path) {
                        Some(Tree::Group { location, .. }) => self.hexy().emit(HexyMsg::Select(
                            location.index_from as u32,
                            location.index_to as u32,
                        )),
                        Some(Tree::Leaf(Leaf::Real(RealLeaf {
                            location,
                            information,
                            ..
                        }))) => {
                            self.doc().emit(DocMsg::T(
                                Some((location.from, location.to)),
                                information.clone(),
                            ));

                            self.hexy().emit(HexyMsg::Select(
                                location.index as u32,
                                location.index as u32,
                            ));
                        }
                        Some(Tree::Leaf(Leaf::Virtual(VirtualLeaf { information, .. }))) => {
                            self.doc().emit(DocMsg::T(None, information.clone()));
                            self.hexy().emit(HexyMsg::Unselect);
                        }
                        _ => self.hexy().emit(HexyMsg::Unselect),
                    };
                }
            }
            AppMsg::Unselect => {
                self.hexy().emit(HexyMsg::Unselect);
            }
            AppMsg::Paste => {
                let clipboard = gdk::Display::default().unwrap().clipboard();
                clipboard.read_text_async(
                    gio::Cancellable::NONE,
                    clone!(@strong sender => move |res|
                    if let Ok(Some(s)) = res {
                                    sender.input(AppMsg::Open(s.to_string()));
                    }),
                );
            }
            AppMsg::Quit => {
                relm4::main_application().quit();
            }
        }
    }
}
