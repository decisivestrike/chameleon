use grapes::{
    Css,
    css::StylePriority,
    gtk::{
        self, ApplicationWindow, EventControllerKey, EventControllerMotion,
        Fixed, Orientation, Widget,
        gdk::{self, Key},
        glib::clone,
        prelude::{GtkWindowExt, *},
    },
    layer_shell::{self, Edge},
};
use layer_shell::{KeyboardMode, Layer, LayerShell};
use log::{Level, info, log_enabled};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

#[derive(Clone)]
pub struct WidgetsLayer {
    window: ApplicationWindow,
    fixer: Fixed,
    active_widget: Rc<RefCell<Option<Widget>>>,
    tracked_widget: Rc<RefCell<Option<Widget>>>,
    previous_mouse_positon: Rc<Cell<(f64, f64)>>,
}

impl WidgetsLayer {
    pub fn new(application: &gtk::Application, monitor: &gdk::Monitor) -> Self {
        let window = ApplicationWindow::new(application);

        let fixer = Fixed::new();
        window.set_child(Some(&fixer));

        window.connect_realize(clone!(
            #[strong]
            fixer,
            move |_| {
                let mut child = fixer.first_child();
                while let Some(widget) = child {
                    let (x, y) = fixer.child_position(&widget);
                    fixer.move_(&widget, x, y);

                    child = widget.next_sibling();
                }
            }
        ));

        let widget_layer = Self {
            window,
            fixer,
            active_widget: Default::default(),
            tracked_widget: Default::default(),
            previous_mouse_positon: Default::default(),
        };

        widget_layer.setup(monitor);

        Css::from_str(include_str!("../styles/widget-layer.css"))
            .apply(StylePriority::User);

        widget_layer
    }

    /// Add widget to a layer
    pub fn append(&self, widget: impl AsRef<Widget>, x: f64, y: f64) {
        let wrapper = gtk::Box::new(Orientation::Horizontal, 0);
        wrapper.set_widget_name("widget-wrapper");

        let motion_controller = gtk::EventControllerMotion::new();

        motion_controller.connect_enter(clone!(
            #[strong(rename_to=active_widget)]
            self.active_widget,
            #[strong]
            wrapper,
            move |_, _, _| {
                let widget: Widget = wrapper.clone().into();

                if log_enabled!(Level::Info) {
                    let widget_name =
                        widget.first_child().unwrap().widget_name();
                    info!("Active widget: {widget_name}.",);
                }

                *active_widget.borrow_mut() = Some(widget);
            }
        ));

        motion_controller.connect_leave(clone!(
            #[strong(rename_to=active_widget)]
            self.active_widget,
            move |_| {
                *active_widget.borrow_mut() = None;
                info!("No active widget.");
            }
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
        window.set_default_size(
            monitor.geometry().width(),
            monitor.geometry().height(),
        );

        window.set_decorated(false);
        window.set_resizable(false);

        window.init_layer_shell();
        window.set_namespace(Some("chameleon-widgets"));
        window.set_keyboard_mode(KeyboardMode::OnDemand);
        window.set_layer(Layer::Bottom);

        window.set_anchor(Edge::Top, true);
        window.set_anchor(Edge::Right, true);
        window.set_anchor(Edge::Bottom, true);
        window.set_anchor(Edge::Left, true);

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
        let motion_handler = clone!(
            #[strong(rename_to=previous_mouse_positon)]
            self.previous_mouse_positon,
            #[strong(rename_to=tracked_widget)]
            self.tracked_widget,
            #[strong(rename_to=fixer)]
            self.fixer,
            move |_: &EventControllerMotion, mouse_x, mouse_y| {
                let tracked = tracked_widget.borrow();

                if let Some(widget) = tracked.as_ref() {
                    let (previous_mouse_x, previous_mouse_y) =
                        previous_mouse_positon.get();
                    let (widget_x, widget_y) = fixer.child_position(widget);

                    let diff_x = mouse_x - previous_mouse_x;
                    let diff_y = mouse_y - previous_mouse_y;

                    let x = widget_x + diff_x;
                    let y = widget_y + diff_y;

                    if log_enabled!(Level::Info) {
                        let widget_name =
                            widget.first_child().unwrap().widget_name();
                        info!("Move {widget_name} to x: {x:.0}, y: {y:.0}",);
                    }

                    fixer.move_(widget, x, y);
                }

                previous_mouse_positon.set((mouse_x, mouse_y));
            }
        );

        let motion_controller = EventControllerMotion::new();
        motion_controller.connect_motion(motion_handler);
        self.window.add_controller(motion_controller);
    }

    pub fn destroy(&self) {
        self.window.destroy();
    }
}
