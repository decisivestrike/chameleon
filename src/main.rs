mod widgets;

use grapes::gtk;
use gtk::glib::{self};
use gtk::prelude::*;

use crate::widgets::WidgetLayer;

fn build_ui(application: &gtk::Application) {
    let widget_layer = WidgetLayer::new(application);

    {
        let l = gtk::Label::new(Some("Drag Me!"));
        widget_layer.append(&l, 50.0, 50.0);
    }

    {
        let l = gtk::Label::new(Some("Drag Me Too!"));
        widget_layer.append(&l, 100.0, 100.0);
    }

    {
        let l = gtk::Label::new(Some("Pretty good!"));
        widget_layer.append(&l, 150.0, 150.0);
    }

    widget_layer.present();
}

fn main() -> glib::ExitCode {
    let app = gtk::Application::builder()
        .application_id("decisivestrike.chameleon")
        .build();

    app.connect_startup(|_| {
        let provider = gtk::CssProvider::new();
        provider.load_from_string(
            r#"
            #widget-layer {
                margin: 0px;
                padding: 0px;
                border: none;
                background-color: rgba(0, 0, 0, 0.0);
            }
            
            box {
                margin: 0px;
                padding: 0px;
                border: none;
                border-radius: 0px;
            }

            label {
                font-family: "JetBrainMono Nerd Font Mono";
                color: whitesmoke;
                font-size: 40px;
                padding: 4px 10px;
                text-shadow: 1px 1px 2px rgba(0, 0, 0, 0.7);
                border: 2px dashed white;
                border-radius: 10px;
            }
            "#,
        );

        gtk::style_context_add_provider_for_display(
            &gtk::gdk::Display::default().expect("Could not connect to a display."),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    });

    app.connect_activate(|app| build_ui(app));

    app.run()
}
