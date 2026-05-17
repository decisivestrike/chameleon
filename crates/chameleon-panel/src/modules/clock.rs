use crate::config::ClockRules;
use crate::modules::{Metadata, PanelModule};
use chrono::Local;
use gtk::glib::clone::Downgrade;
use gtk::glib::object::Cast;
use gtk::glib::{self, ControlFlow};
use std::rc::Rc;

/// Clock module
pub struct Clock;

impl PanelModule for Clock {
    type Rules = ClockRules;

    fn create(
        rules: Self::Rules,
        _meta: Rc<Metadata>,
    ) -> Result<gtk::Widget, super::Error> {
        let clock = gtk::Label::builder()
            .name("clock")
            .css_classes(["module"])
            .build();

        glib::timeout_add_seconds_local(1, {
            let clock_weak = clock.downgrade();
            move || {
                if let Some(clock) = clock_weak.upgrade() {
                    let time = Local::now().format(&rules.format).to_string();
                    clock.set_label(&time);

                    ControlFlow::Continue
                } else {
                    ControlFlow::Break
                }
            }
        });

        Ok(clock.upcast())
    }
}
