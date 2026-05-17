use crate::config::{Module, Position, Rules};
use crate::modules::{Clock, Metadata, PanelModule};
use gtk::glib::Object;
use gtk::prelude::GtkWindowExt;
use gtk::subclass::prelude::*;
use gtk::{Orientation, gdk, glib};
use layer_shell::{Edge, LayerShell};
use std::rc::Rc;

mod imp {
    use gtk::glib;
    use gtk::prelude::*;
    use gtk::subclass::prelude::*;
    use layer_shell::{KeyboardMode, LayerShell};

    #[derive(Default)]
    pub struct PanelImp {
        pub left: gtk::Box,
        pub center: gtk::Box,
        pub right: gtk::Box,
        pub centerbox: gtk::CenterBox,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PanelImp {
        const NAME: &'static str = "ChameleonPanel";
        type Type = super::Panel;
        type ParentType = gtk::Window;
    }

    impl ObjectImpl for PanelImp {
        fn constructed(&self) {
            self.parent_constructed();

            self.centerbox.set_start_widget(Some(&self.left));
            self.centerbox.set_center_widget(Some(&self.center));
            self.centerbox.set_end_widget(Some(&self.right));

            let window = self.obj();
            window.set_child(Some(&self.centerbox));

            window.set_widget_name("panel");
            window.set_decorated(false);
            window.set_resizable(false);

            // layer shell
            window.init_layer_shell();
            window.set_namespace(Some("chameleon-panel"));
            window.set_keyboard_mode(KeyboardMode::None);
        }
    }

    impl WidgetImpl for PanelImp {}

    impl WindowImpl for PanelImp {}
}

// Obj
glib::wrapper! {
    /// Panel
    pub struct Panel(ObjectSubclass<imp::PanelImp>)
        @extends gtk::Window, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Panel {
    pub fn new(rules: Rules, monitor: gdk::Monitor) -> Self {
        let panel: Self = Object::builder().build();
        panel.set_monitor(Some(&monitor));

        let orientation = match rules.position {
            Position::Top | Position::Bottom => Orientation::Horizontal,
            Position::Right | Position::Left => Orientation::Vertical,
        };

        match orientation {
            Orientation::Horizontal => {
                if let Some(thickness) = rules.thickness {
                    panel.set_default_height(thickness);
                    panel.set_exclusive_zone(thickness);
                } else {
                    panel.set_default_height(-1);
                    panel.auto_exclusive_zone_enable();
                }
            }
            Orientation::Vertical => {
                if let Some(thickness) = rules.thickness {
                    panel.set_default_width(thickness);
                    panel.set_exclusive_zone(thickness);
                } else {
                    panel.set_default_width(-1);
                    panel.auto_exclusive_zone_enable();
                }
            }
            _ => unreachable!(),
        }

        let meta = Rc::new(Metadata {
            monitor,
            orientation,
        });

        for module in rules.modules.left {
            let widget = match module {
                Module::Clock => Clock::create(rules.clock, meta),
                Module::Battery => todo!(),
                Module::Workspaces => todo!(),
                Module::KeyboardLayout => todo!(),
                Module::Pulseaudio => todo!(),
                Module::Separator => todo!(),
            };

            panel.append_left_module(&widget.unwrap());
        }

        for module in rules.modules.center {
            let widget = match module {
                Module::Clock => Clock::create(rules.clock, meta),
                Module::Battery => todo!(),
                Module::Workspaces => todo!(),
                Module::KeyboardLayout => todo!(),
                Module::Pulseaudio => todo!(),
                Module::Separator => todo!(),
            };

            panel.append_center_module(&widget.unwrap());
        }

        for module in rules.modules.right {
            let widget = match module {
                Module::Clock => Clock::create(rules.clock, meta),
                Module::Battery => todo!(),
                Module::Workspaces => todo!(),
                Module::KeyboardLayout => todo!(),
                Module::Pulseaudio => todo!(),
                Module::Separator => todo!(),
            };

            panel.append_right_module(&widget.unwrap());
        }

        panel
    }

    fn append_left_module(&self, module: &gtk::Widget) {
        self.imp().left.append(module);
    }

    fn append_center_module(&self, module: &gtk::Widget) {
        self.imp().center.append(module);
    }

    fn append_right_module(&self, module: &gtk::Widget) {
        self.imp().right.append(module);
    }

    fn set_position(&self, position: &Position) {
        let (top, right, bottom, left) = match position {
            Position::Top => (true, true, false, true),
            Position::Right => (true, true, true, false),
            Position::Bottom => (false, true, true, true),
            Position::Left => (true, false, true, true),
        };

        self.set_anchor(Edge::Top, top);
        self.set_anchor(Edge::Right, right);
        self.set_anchor(Edge::Bottom, bottom);
        self.set_anchor(Edge::Left, left);
    }
}
