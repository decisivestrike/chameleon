use crate::config::Configuration;
use gtk::gdk::prelude::MonitorExt;
use gtk::glib::clone;
use gtk::prelude::{
    BoxExt, FixedExt, GestureSingleExt, GtkWindowExt, WidgetExt,
};
use gtk::{
    EventControllerMotion, Fixed, GestureClick, Orientation, Widget, Window,
    gdk,
};
use gtke::css::StylePriority;
use gtke::{Css, WindowComponent};
use layer_shell::{self, Edge, KeyboardMode, Layer, LayerShell};
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use tracing::{Level, debug, enabled, trace};

#[derive(Clone, WindowComponent)]
pub struct WidgetsLayer {
    #[root]
    window: Window,
    fixer: Fixed,
    active_widget: Rc<RefCell<Option<Widget>>>,
    tracked_widget: Rc<RefCell<Option<Widget>>>,
    previous_mouse_positon: Rc<Cell<(f64, f64)>>,
}

impl WidgetsLayer {
    pub fn new(monitor: &gdk::Monitor, _config: &Configuration) -> Self {
        let window = Window::new();
        let fixer = Fixed::new();
        window.set_child(Some(&fixer));

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

                if enabled!(Level::DEBUG) {
                    let widget_name =
                        widget.first_child().unwrap().widget_name();
                    debug!("Active widget: {widget_name}",);
                }

                *active_widget.borrow_mut() = Some(widget);
            }
        ));

        motion_controller.connect_leave(clone!(
            #[strong(rename_to=active_widget)]
            self.active_widget,
            move |_| {
                *active_widget.borrow_mut() = None;
                debug!("No active widget.");
            }
        ));

        wrapper.add_controller(motion_controller);
        wrapper.append(widget.as_ref());

        self.fixer.put(&wrapper, x, y);
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
        let controller = GestureClick::new();
        controller.set_button(1);

        controller.connect_pressed(clone!(
            #[strong(rename_to=tracked)]
            self.tracked_widget,
            #[strong(rename_to=active)]
            self.active_widget,
            move |_, _, _x, _y| {
                *tracked.borrow_mut() = active.borrow().clone();
            }
        ));

        controller.connect_released(clone!(
            #[strong(rename_to=tracked)]
            self.tracked_widget,
            move |_, _, _x, _y| {
                *tracked.borrow_mut() = None;
            }
        ));

        self.window.add_controller(controller);
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

                    if enabled!(Level::TRACE) {
                        let widget_name =
                            widget.first_child().unwrap().widget_name();
                        trace!("Move {widget_name} to x: {x:.0}, y: {y:.0}",);
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
}
