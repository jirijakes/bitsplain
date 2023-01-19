use std::rc::Rc;

use bitsplain::tree::*;
use gtk::prelude::*;
use lazy_static::lazy_static;
use relm4::prelude::*;

use crate::app::AppMsg;
use crate::rich_label::RichLabel;

#[derive(Clone, Debug)]
struct Row {
    annotation: String,
    length: Option<usize>,
    data_type: Option<String>,
    value: String,
    attrs: gtk::pango::AttrList,
    path: Vec<String>,
    tags: Vec<bitsplain::tree::Tag>,
    children: Vec<Row>,
}

pub enum TreeModel {
    Empty,
    Full {
        annotations: Rc<Tree>,
        store: gtk::TreeListModel,
    },
}

// impl TreeModel {
//     fn store(&self) -> &gtk::TreeListModel {
//         match self {
//             TreeModel::Empty { store } => store,
//             TreeModel::Full { store, .. } => store,
//         }
//     }
// }

lazy_static! {
    static ref THEME: Vec<String> = vec![
        "#8be9fd".to_string(),
        "#ffb86c".to_string(),
        "#50fa7b".to_string(),
        "#ff79c6".to_string(),
        "#bd93f9".to_string(),
        "#ff5555".to_string(),
        "#f1fa8c".to_string(),
    ];
    static ref THEME_SIZE: usize = THEME.len();
}

#[derive(Debug)]
pub enum TreeMsg {
    Open { annotations: Rc<Tree> },
    Select(Option<Vec<String>>),
}

#[relm4::component(pub)]
impl Component for TreeModel {
    type CommandOutput = ();
    type Init = ();
    type Input = TreeMsg;
    type Output = AppMsg;

    #[rustfmt::skip]
    view! {
	tree = &gtk::ColumnView::new(gtk::SelectionModel::NONE) {
            set_hexpand: true,
            set_vexpand: true,
	    set_show_column_separators: true,
	    set_show_row_separators: true,
            set_reorderable: false,
	    set_single_click_activate: true,
	    add_css_class: "data-table",
	    append_column: &gtk::ColumnViewColumn::new(Some("Name"), Some(&col_name_factory)),
	    append_column: &gtk::ColumnViewColumn::new(Some("Data type"), Some(&col_datatype_factory)),
	    append_column: &gtk::ColumnViewColumn::new(Some("Length"), Some(&col_length_factory)),
	    append_column: &gtk::ColumnViewColumn::new(Some("Value"), Some(&col_value_factory)),
	}
    }

    fn init(
        _annotations: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = TreeModel::Empty;

        let col_name_factory = gtk::SignalListItemFactory::new();
        col_name_factory.connect_setup(on_setup_name);
        col_name_factory.connect_bind(on_bind_name);

        let col_datatype_factory = gtk::SignalListItemFactory::new();
        col_datatype_factory.connect_setup(on_setup_datatype);
        col_datatype_factory.connect_bind(on_bind_datatype);

        let col_length_factory = gtk::SignalListItemFactory::new();
        col_length_factory.connect_setup(on_setup_length);
        col_length_factory.connect_bind(on_bind_length);

        let col_value_factory = gtk::SignalListItemFactory::new();
        col_value_factory.connect_setup(on_setup_value);
        col_value_factory.connect_bind(on_bind_value);

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, tree: &Self::Root) {
        match msg {
            TreeMsg::Open { annotations } => {
                let root = tree_to_model(&annotations);
                let tree_list_model = gtk::TreeListModel::new(&root, false, true, get_children);
                let selection = gtk::SingleSelection::builder()
                    .model(&tree_list_model)
                    .build();
                selection.connect_selected_item_notify(move |sel| {
                    let object = sel
                        .selected_item()
                        .and_downcast::<gtk::TreeListRow>()
                        .and_then(|r| r.item())
                        .and_downcast::<gtk::glib::BoxedAnyObject>();

                    if let Some(object) = object {
                        let r = object.borrow::<Row>();
                        sender.input(TreeMsg::Select(Some(r.path.clone())));
                    } else {
                        sender.input(TreeMsg::Select(None));
                    };
                });
                tree.set_model(Some(&selection));

                // *self = TreeModel::Full {
                //     annotations: annotations.clone(),
                //     store: self.store().clone(),
                // };

                // self.store().clear();

                // tree.expand_all();
            }
            TreeMsg::Select(Some(path)) => sender.output(AppMsg::Select(path)).unwrap(),
            TreeMsg::Select(None) => sender.output(AppMsg::Unselect).unwrap(),
        }
    }
}

fn get_children(obj: &gtk::glib::Object) -> Option<gtk::gio::ListModel> {
    let boxed = obj.downcast_ref::<gtk::glib::BoxedAnyObject>()?;
    let row: &Row = &boxed.borrow();

    if row.children.is_empty() {
        None
    } else {
        let store = gtk::gio::ListStore::new(gtk::glib::BoxedAnyObject::static_type());

        row.children.iter().for_each(|r| {
            store.append(&gtk::glib::BoxedAnyObject::new(r.clone()));
        });

        Some(store.into())
    }
}

fn tree_to_row(tree: &Node) -> Row {
    match tree {
        Node::Group {
            path,
            location: GroupLocation {
                byte_from, byte_to, ..
            },
            information:
                Information {
                    label: annotation,
                    value,
                    data,
                    tags,
                    ..
                },
            children,
        } => {
            let attrs = gtk::pango::AttrList::new();
            let mut font_desc = gtk::pango::FontDescription::new();
            font_desc.set_style(gtk::pango::Style::Italic);
            attrs.insert(gtk::pango::AttrFontDesc::new(&font_desc));
            Row {
                annotation: annotation.clone(),
                length: Some(byte_to - byte_from),
                data_type: None,
                value: value.preview(),
                attrs,
                path: path.clone(),
                tags: tags.clone(),
                children: children.iter().map(tree_to_row).collect(),
            }
        }
        Node::Leaf(Leaf::Real(RealLeaf {
            location: LeafLocation { from, to, index },
            information:
                Information {
                    label: annotation,
                    value,
                    data,
                    tags,
                    ..
                },
            path,
            ..
        })) => {
            let attrs = gtk::pango::AttrList::new();
            let color =
                gtk::pango::Color::parse(unsafe { THEME.get_unchecked(index % *THEME_SIZE) })
                    .unwrap();
            attrs.insert(gtk::pango::AttrColor::new_foreground(
                color.red(),
                color.green(),
                color.blue(),
            ));
            Row {
                annotation: annotation.clone(),
                length: Some(to - from),
                data_type: data.get("datatype").cloned(),
                value: value.preview(),
                attrs,
                path: path.clone(),
                tags: tags.clone(),
                children: vec![],
            }
        }
        Node::Leaf(Leaf::Virtual(VirtualLeaf {
            information:
                Information {
                    label: annotation,
                    value,
                    ..
                },
            path,
            ..
        })) => Row {
            annotation: annotation.clone(),
            length: None,
            data_type: None,
            value: value.preview(),
            attrs: gtk::pango::AttrList::new(),
            path: path.clone(),
            tags: vec![],
            children: vec![],
        },
    }
}

fn tree_to_model(items: &[Node]) -> gtk::gio::ListModel {
    let store = gtk::gio::ListStore::new(gtk::glib::BoxedAnyObject::static_type());

    items.iter().for_each(|t| {
        store.append(&gtk::glib::BoxedAnyObject::new(tree_to_row(t)));
    });

    store.into()
}

fn get_object(list_item: &gtk::ListItem) -> Option<gtk::glib::BoxedAnyObject> {
    list_item
        .item()
        .and_downcast::<gtk::TreeListRow>()
        .and_then(|r| r.item())
        .and_downcast::<gtk::glib::BoxedAnyObject>()
}

fn on_setup_name(_factory: &gtk::SignalListItemFactory, list_item: &gtk::ListItem) {
    let exp = gtk::TreeExpander::new();
    let rich_label = RichLabel::new();
    exp.set_child(Some(&rich_label));
    list_item.set_child(Some(&exp));
}

fn on_bind_name(_factory: &gtk::SignalListItemFactory, list_item: &gtk::ListItem) {
    if let Some(tree_list_row) = list_item.item().and_downcast::<gtk::TreeListRow>() {
        if let Some(object) = tree_list_row
            .item()
            .and_downcast::<gtk::glib::BoxedAnyObject>()
        {
            if let Some(expander) = list_item.child().and_downcast::<gtk::TreeExpander>() {
                expander.set_list_row(Some(&tree_list_row));

                if let Some(rich_label) = expander.child().and_downcast::<RichLabel>() {
                    let row: &Row = &object.borrow::<Row>();
                    rich_label.set_label(&row.annotation);
                    rich_label.set_attributes(&row.attrs);
                    rich_label.set_tags(&row.tags);
                }
            }
        }
    }
}

fn on_setup_datatype(_factory: &gtk::SignalListItemFactory, list_item: &gtk::ListItem) {
    list_item.set_child(Some(&gtk::Label::new(None)))
}

fn on_bind_datatype(_factory: &gtk::SignalListItemFactory, list_item: &gtk::ListItem) {
    if let Some(object) = get_object(list_item) {
        let row: &Row = &object.borrow();
        list_item
            .child()
            .and_downcast::<gtk::Label>()
            .iter()
            .for_each(|lbl| {
                if let Some(dt) = &row.data_type {
                    lbl.set_label(&dt);
                    lbl.set_attributes(Some(&row.attrs));
                }
            })
    }
}

fn on_setup_length(_factory: &gtk::SignalListItemFactory, list_item: &gtk::ListItem) {
    list_item.set_child(Some(&gtk::Label::new(None)))
}

fn on_bind_length(_factory: &gtk::SignalListItemFactory, list_item: &gtk::ListItem) {
    if let Some(object) = get_object(list_item) {
        let row: &Row = &object.borrow();
        list_item
            .child()
            .and_downcast::<gtk::Label>()
            .iter()
            .for_each(|lbl| {
                if let Some(length) = row.length {
                    lbl.set_label(&format!("{}", length));
                    lbl.set_attributes(Some(&row.attrs));
                }
            })
    }
}

fn on_setup_value(_factory: &gtk::SignalListItemFactory, list_item: &gtk::ListItem) {
    list_item.set_child(Some(&gtk::Label::builder().xalign(0.0).build()))
}

fn on_bind_value(_factory: &gtk::SignalListItemFactory, list_item: &gtk::ListItem) {
    if let Some(object) = get_object(list_item) {
        let row: &Row = &object.borrow();
        list_item
            .child()
            .and_downcast::<gtk::Label>()
            .iter()
            .for_each(|lbl| {
                lbl.set_label(&row.value);
                lbl.set_attributes(Some(&row.attrs));
            })
    }
}
