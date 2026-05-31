use crate::modules::Metadata;
use crate::services::pulseaudio::{CurrentSink, PULSEAUDIO_SERVICE};
use anyhow::Result;
use gtk::glib::clone;
use gtk::glib::object::Cast;
use gtk::prelude::WidgetExt;
use gtk::{Widget, glib};
use gtkio::future::spawn;
use gtkio::workers::WatchWorker;
use std::rc::Rc;
use std::sync::LazyLock;
use tokio::sync::watch;
use tracing::error;

pub static PA_HANDLER: LazyLock<watch::Sender<String>> = LazyLock::new(|| {
    let mut current_sink = PULSEAUDIO_SERVICE.subscribe();

    let label = Pulseaudio::label_from_sink(&current_sink.borrow());
    let sender = watch::Sender::new(label);

    spawn({
        let sender = sender.clone();
        let sink_clone = current_sink.clone();
        async move {
            let sink = sink_clone.borrow();
            let label = Pulseaudio::label_from_sink(&sink);
            if let Err(e) = sender.send(label) {
                error!("{e}");
            }
        }
    });

    spawn({
        let sender = sender.clone();
        async move {
            loop {
                match current_sink.changed().await {
                    Ok(()) => {
                        let sink = current_sink.borrow();
                        let label = Pulseaudio::label_from_sink(&sink);

                        if let Err(e) = sender.send(label) {
                            error!("{e}");
                        }
                    }
                    Err(e) => error!("{e}"),
                }
            }
        }
    });

    sender
});

pub struct Pulseaudio;

impl super::PanelModule for Pulseaudio {
    type Rules = ();

    fn create(_: Self::Rules, _: Rc<Metadata>) -> Result<Widget> {
        let pulseaudio = gtk::Label::new(None);

        PA_HANDLER.listen_local(clone!(
            #[weak]
            pulseaudio,
            async move |recv| {
                let value = recv.borrow_and_update();
                pulseaudio.set_label(&value);
            }
        ));

        pulseaudio.set_widget_name(Self::NAME);
        pulseaudio.add_css_class("module");

        Ok(pulseaudio.upcast())
    }
}

impl Pulseaudio {
    const NAME: &str = "pulseaudio";

    pub fn label_from_sink(sink: &CurrentSink) -> String {
        if sink.muted {
            "MUTED".to_string()
        } else {
            format!("VOL {}%", sink.volume)
        }
    }
}
