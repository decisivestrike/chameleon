use crate::modules::{Metadata, PanelModule};
use anyhow::Result;
use gtk::Widget;
use gtk::glib::object::Cast;
use gtk::prelude::{GestureSingleExt, WidgetExt};
use std::io::Write;
use std::os::unix::net::UnixStream;
use std::rc::Rc;
use tracing::error;

/// Clock module
pub struct LauncherToggle;

impl PanelModule for LauncherToggle {
    type Rules = ();

    fn create(rules: (), _meta: Rc<Metadata>) -> Result<Widget> {
        let toggler = gtk::Image::builder()
            .file("../../assets/svg/launcher-icon.svg")
            .pixel_size(16)
            .name("launcher-toggle")
            .css_classes(["module"])
            .build();

        let toggle_controller = gtk::GestureClick::new();
        toggle_controller.set_button(1);
        toggle_controller.connect_pressed(|_gesture, _n_press, _x, _y| {
            LauncherToggle::toggle_launcher();
        });
        toggler.add_controller(toggle_controller);

        Ok(toggler.upcast())
    }
}

impl LauncherToggle {
    fn toggle_launcher() {
        let mut stream =
            match UnixStream::connect("/tmp/chameleon/launcher.sock") {
                Ok(stream) => stream,
                Err(e) => {
                    error!("{e}");
                    return;
                }
            };

        if let Err(e) = stream.write_all(&[2]) {
            error!("{e}");
        }
    }
}
