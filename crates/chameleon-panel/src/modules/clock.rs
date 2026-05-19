use crate::config::ClockRules;
use crate::modules::{Metadata, PanelModule};
use anyhow::Result;
use chrono::Local;
use gtk::Widget;
use gtk::glib::clone::Downgrade;
use gtk::glib::object::Cast;
use gtk::glib::{self, ControlFlow};
use std::rc::Rc;

/// Clock module
pub struct Clock;

impl PanelModule for Clock {
    type Rules = ClockRules;

    fn create(rules: ClockRules, _meta: Rc<Metadata>) -> Result<Widget> {
        let clock = gtk::Label::builder()
            .name("clock")
            .css_classes(["module"])
            .build();

        Clock::update_label(&clock, &rules.format);

        glib::timeout_add_seconds_local(1, {
            let clock_weak = clock.downgrade();
            move || {
                if let Some(clock) = clock_weak.upgrade() {
                    Clock::update_label(&clock, &rules.format);
                    ControlFlow::Continue
                } else {
                    ControlFlow::Break
                }
            }
        });

        Ok(clock.upcast())
    }
}

impl Clock {
    fn update_label(label: &gtk::Label, format: &str) {
        let time = Local::now().format(format).to_string();
        label.set_label(&time);
    }
}
