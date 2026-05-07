use pulse::callbacks::ListResult;
use pulse::context::introspect::{Introspector, SinkInfo};
use pulse::context::subscribe::{Facility, InterestMaskSet, Operation};
use pulse::context::{Context, State};
use pulse::mainloop::standard::{IterateResult, Mainloop};
use pulse::proplist::Proplist;
use pulse::volume::Volume;
use std::sync::LazyLock;
use std::thread;
use tokio::sync::watch;

pub static PULSEAUDIO_SERVICE: LazyLock<watch::Sender<CurrentSink>> =
    LazyLock::new(|| {
        let sender = watch::Sender::new(CurrentSink {
            name: "unknown".to_string(),
            volume: 0,
            muted: false,
        });

        thread::spawn({
            let sender = sender.clone();
            move || listen_pa_events(sender)
        });

        sender
    });

#[derive(Clone)]
pub struct CurrentSink {
    pub name: String,
    pub volume: u8,
    pub muted: bool,
}

fn listen_pa_events(sender: watch::Sender<CurrentSink>) -> ! {
    let mut proplist = Proplist::new().unwrap();
    proplist
        .set_str(
            pulse::proplist::properties::APPLICATION_NAME,
            "chameleon-panel",
        )
        .unwrap();

    let mut mainloop = Mainloop::new().unwrap();
    let mut context =
        Context::new_with_proplist(&mainloop, "MainConn", &proplist).unwrap();

    context
        .connect(None, pulse::context::FlagSet::NOFLAGS, None)
        .unwrap();

    loop {
        match mainloop.iterate(true) {
            IterateResult::Err(_) => {
                eprintln!("Error: iterate state was not success");
            }
            IterateResult::Quit(_) => {
                eprintln!("Quit: iterate state was not success");
            }
            _ => (),
        }

        match context.get_state() {
            State::Ready => break,
            State::Failed | State::Terminated => {
                eprintln!("context state failed/terminated");
            }
            _ => {}
        }
    }

    context.subscribe(InterestMaskSet::ALL, |ok| {
        if !ok {
            log::error!("Some kind of error while subscribe callback")
        }
    });

    let subscribtion_callback =
        create_subscribe_callback(context.introspect(), sender.clone());
    context.set_subscribe_callback(Some(subscribtion_callback));

    context.introspect().get_sink_info_by_name(
        "@DEFAULT_SINK@",
        create_result_handler(sender.clone()),
    );

    loop {
        mainloop.iterate(true);
    }
}

fn create_subscribe_callback(
    introspector: Introspector,
    sender: watch::Sender<CurrentSink>,
) -> Box<dyn FnMut(Option<Facility>, Option<Operation>, u32) + 'static> {
    Box::new(move |facility, operation, index| {
        if let Some(Facility::Sink) = facility
            && let Some(Operation::Changed) = operation
        {
            introspector.get_sink_info_by_index(
                index,
                create_result_handler(sender.clone()),
            );
        }
    })
}

fn create_result_handler(
    sender: watch::Sender<CurrentSink>,
) -> impl FnMut(ListResult<&SinkInfo>) + 'static {
    move |result| {
        if let ListResult::Item(info) = result {
            let name = if let Some(ref active_port) = info.active_port
                && let Some(ref description) = active_port.description
            {
                description.to_string()
            } else {
                "unknown".to_string()
            };

            let sink_info = CurrentSink {
                name,
                volume: volume_to_u8(info.volume.max()),
                muted: info.mute,
            };

            if let Err(e) = sender.send(sink_info) {
                log::error!("{e}");
            }
        }
    }
}

fn volume_to_u8(volume: Volume) -> u8 {
    (volume.0 as f32 * 100.0 / 65536.0).round() as u8
}
