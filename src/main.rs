mod core;
mod widgets;

use crate::widgets::WidgetLayer;
use grapes::gtk::{
    self,
    gio::prelude::{ApplicationExt, ApplicationExtManual},
};
use gtk::glib::{self};

fn build_ui(application: &gtk::Application) {
    for monitor in core::monitors().iter() {
        println!("{monitor:?}");
        let widget_layer = WidgetLayer::new(application, monitor);

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
            
            #widget-layer #widget-wrapper {
                margin: 0px;
                padding: 0px;
                border: none;
                border-radius: 100px;
            }

            label {
                font-family: "JetBrainMono Nerd Font Mono";
                color: whitesmoke;
                font-size: 40px;
                padding: 4px 10px;
                text-shadow: 1px 1px 2px rgba(0, 0, 0, 0.7);
                border: 1px solid white;
                border-radius: 20px;
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
