use grapes::{
    gtk::{
        self, ApplicationWindow, EventControllerKey, Fixed, Orientation, Widget,
        gdk::{self, Key},
        glib::clone,
        prelude::{GtkWindowExt, *},
    },
    layer_shell,
};
use layer_shell::{KeyboardMode, Layer, LayerShell};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

#[derive(Clone)]
pub struct WidgetLayer {
    window: ApplicationWindow,
    fixer: Fixed,
    active_widget: Rc<RefCell<Option<Widget>>>,
    tracked_widget: Rc<RefCell<Option<Widget>>>,
    previous_mouse_positon: Rc<Cell<(f64, f64)>>,
    monitor_width: i32,
    monitor_height: i32,
}

impl WidgetLayer {
    pub fn new(application: &gtk::Application, monitor: &gdk::Monitor) -> Self {
        let window = ApplicationWindow::new(application);

        let fixer = Fixed::new();
        window.set_child(Some(&fixer));

        let layer = Self {
            window,
            fixer,
            active_widget: Default::default(),
            tracked_widget: Default::default(),
            previous_mouse_positon: Default::default(),
            monitor_width: monitor.geometry().width(),
            monitor_height: monitor.geometry().height(),
        };

        layer.setup(monitor);

        layer
    }

    pub fn append(&self, widget: impl AsRef<Widget>, x: f64, y: f64) {
        let wrapper = gtk::Box::new(Orientation::Horizontal, 0);
        wrapper.set_widget_name("widget-wrapper");

        let motion_controller = gtk::EventControllerMotion::new();

        motion_controller.connect_leave(clone!(
            #[strong(rename_to=active_widget)]
            self.active_widget,
            move |_| *active_widget.borrow_mut() = None
        ));

        motion_controller.connect_enter(clone!(
            #[strong(rename_to=active_widget)]
            self.active_widget,
            #[strong]
            wrapper,
            move |_, _, _| *active_widget.borrow_mut() = Some(wrapper.clone().into())
        ));

        wrapper.add_controller(motion_controller);
        wrapper.append(widget.as_ref());

        self.fixer.put(&wrapper, x, y);
    }

    /// This is almost an ordinary window, so it should be presented.
    pub fn present(&self) {
        self.window.present();
    }

    fn setup(&self, monitor: &gdk::Monitor) {
        let window = &self.window;

        window.set_widget_name("widget-layer");
        window.set_default_size(self.monitor_width, self.monitor_height);

        window.set_decorated(false);
        window.set_resizable(false);

        window.init_layer_shell();
        window.set_keyboard_mode(KeyboardMode::OnDemand);
        window.set_layer(Layer::Bottom);

        window.set_monitor(Some(monitor));

        self.setup_motion_controller();
        self.setup_tracker();
    }

    /// Allows to track the widget
    fn setup_tracker(&self) {
        let key_controller = EventControllerKey::new();

        key_controller.connect_key_pressed(clone!(
            #[strong(rename_to=tracked)]
            self.tracked_widget,
            #[strong(rename_to=active)]
            self.active_widget,
            move |_, key, _keyval, _state| {
                if key == Key::Super_L {
                    *tracked.borrow_mut() = active.borrow().clone();
                }
                gtk::glib::Propagation::Proceed
            }
        ));

        key_controller.connect_key_released(clone!(
            #[strong(rename_to=tracked)]
            self.tracked_widget,
            move |_, key, _keyval, _state| {
                if key == Key::Super_L {
                    *tracked.borrow_mut() = None;
                }
            }
        ));

        self.window.add_controller(key_controller);
    }

    fn setup_motion_controller(&self) {
        let motion_controller = gtk::EventControllerMotion::new();

        let monitor_width = self.monitor_width;
        let monitor_height = self.monitor_height;

        motion_controller.connect_motion(clone!(
            #[strong(rename_to=previous_mouse_positon)]
            self.previous_mouse_positon,
            #[strong(rename_to=tracked_widget)]
            self.tracked_widget,
            #[strong(rename_to=fixer)]
            self.fixer,
            move |_, x, y| {
                let tracked = tracked_widget.borrow();

                if let Some(widget) = tracked.as_ref() {
                    let (prev_x, prev_y) = previous_mouse_positon.get();
                    let (wx, wy) = fixer.child_position(widget);

                    let diff_x = x - prev_x;
                    let diff_y = y - prev_y;

                    let max_x = (monitor_width - widget.width()) as f64;
                    let new_x_unchecked = wx + diff_x;

                    let new_x = if new_x_unchecked > max_x {
                        max_x
                    } else if new_x_unchecked < 0.0 {
                        0.0
                    } else {
                        new_x_unchecked
                    };

                    let max_y = (monitor_height - widget.height()) as f64;
                    let new_y_unchecked = wy + diff_y;

                    let new_y = if new_y_unchecked > max_y {
                        max_y
                    } else if new_y_unchecked < 0.0 {
                        0.0
                    } else {
                        new_y_unchecked
                    };

                    println!("Move to {new_x} {new_y}");
                    fixer.move_(widget, new_x, new_y);
                }

                previous_mouse_positon.set((x, y));
            }
        ));

        self.window.add_controller(motion_controller);
    }
}
