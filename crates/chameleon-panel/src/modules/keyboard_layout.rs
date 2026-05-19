use crate::modules::{Metadata, PanelModule};
use anyhow::Result;
use chameleon_ipc::KEYBOARD_LAYOUT;
use gtk::glib::clone;
use gtk::glib::object::Cast;
use gtk::{Widget, glib};
use std::rc::Rc;

pub struct KeyboardLayout;

impl PanelModule for KeyboardLayout {
    type Rules = ();

    fn create(_: Self::Rules, _: Rc<Metadata>) -> Result<Widget> {
        let kb_layout = gtk::Label::builder()
            .name(Self::NAME)
            .css_classes(["module"])
            .build();

        let current_layout = &*KEYBOARD_LAYOUT.borrow();
        kb_layout
            .set_label(&KeyboardLayout::shrink_layout_name(current_layout));

        KEYBOARD_LAYOUT.listen_local(clone!(
            #[weak]
            kb_layout,
            async move |state| {
                let layout_name = state.borrow_and_update();
                let shrinked_name =
                    KeyboardLayout::shrink_layout_name(&*layout_name);

                kb_layout.set_label(&shrinked_name);
            }
        ));

        Ok(kb_layout.upcast())
    }
}

impl KeyboardLayout {
    const NAME: &str = "keyboard-layout";

    pub fn shrink_layout_name(name: &String) -> String {
        name.chars()
            .take(2)
            .flat_map(|c| c.to_uppercase())
            .collect()
    }
}
