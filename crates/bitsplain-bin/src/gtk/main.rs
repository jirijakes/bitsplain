use gtk::gdk;
use gtk::gio::Menu;
use gtk::prelude::ApplicationExt;
use gtk::traits::GtkApplicationExt;
use relm4::gtk;

mod app;
mod comp;
mod hexy;
mod rich_label;
mod tag;

relm4::new_action_group!(WindowActionGroup, "win");
relm4::new_stateless_action!(QuitAction, WindowActionGroup, "quit");
relm4::new_stateless_action!(PasteAction, WindowActionGroup, "paste");

fn main() {
    gtk::init().expect(":-(");

    relm4::menu! {
    file_menu: {
        "Quit" => QuitAction
    },
    edit_menu: {
        "Paste" => PasteAction
    }
    }

    // let model = app::AppModel::new(annotations, aa.data.bytes());
    let app = gtk::Application::default();
    let menubar = Menu::new();
    menubar.append_submenu(Some("File"), &file_menu);
    menubar.append_submenu(Some("Edit"), &edit_menu);
    app.connect_startup(move |a| {
        a.set_menubar(Some(&menubar));
    });

    let provider = gtk::CssProvider::new();
    provider.load_from_data(include_str!("x.css"));
    if let Some(display) = &gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        )
    };

    let x = relm4::RelmApp::from_app(app);
    x.run::<app::AppModel>(std::env::args().nth(1));
}
