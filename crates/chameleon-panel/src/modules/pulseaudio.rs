use crate::common::Metadata;
use crate::modules::{BaseModule, ModuleFactory};
use crate::services::{CurrentSink, PULSEAUDIO_SERVICE};
use gtk::prelude::WidgetExt;
use gtke::Component;
use gtkio::future::spawn;
use std::rc::Rc;
use std::sync::LazyLock;
use tokio::sync::watch;

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
                log::error!("{e}");
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
                            log::error!("{e}");
                        }
                    }
                    Err(e) => log::error!("{e}"),
                }
            }
        }
    });

    sender
});

#[derive(Debug, Component)]
pub struct Pulseaudio {
    #[root]
    base: BaseModule,
}

impl ModuleFactory for Pulseaudio {
    type Config = ();

    fn create(
        _config: &Self::Config,
        _meta: &Metadata,
    ) -> anyhow::Result<Rc<dyn Component>> {
        let pa = Pulseaudio::new();

        Ok(Rc::new(pa))
    }
}

impl Pulseaudio {
    const NAME: &str = "pulseaudio";

    pub fn new() -> Self {
        let state = PA_HANDLER.subscribe();
        let base = BaseModule::new(state);

        base.set_widget_name(Self::NAME);
        base.add_css_class("module");

        Self { base }
    }

    pub fn label_from_sink(sink: &CurrentSink) -> String {
        if sink.muted {
            "MUTED".to_string()
        } else {
            format!("VOL {}%", sink.volume)
        }
    }
}
