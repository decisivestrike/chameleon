use pulse::callbacks::ListResult;
use pulse::context::introspect::{Introspector, ServerInfo, SinkInfo};
use pulse::context::subscribe::{Facility, InterestMaskSet, Operation};
use pulse::context::{Context, State};
use pulse::mainloop::standard::{IterateResult, Mainloop};
use pulse::proplist::Proplist;
use pulse::volume::Volume;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{LazyLock, RwLock};
use std::thread;
use tokio::sync::watch;
use tracing::error;

const DEFAULT_SINK_NAME: &str = "@DEFAULT_SINK@";
static ACTIVE_SINK_INDEX: RwLock<Option<u32>> = RwLock::new(None);

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

    let context = Rc::new(RefCell::new(context));

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

        match context.borrow().get_state() {
            State::Ready => break,
            State::Failed | State::Terminated => {
                eprintln!("context state failed/terminated");
            }
            _ => {}
        }
    }

    context.borrow_mut().subscribe(InterestMaskSet::ALL, |ok| {
        if !ok {
            error!("Some kind of error while subscribe callback")
        }
    });

    let ctx_clone = context.clone();
    context.borrow().introspect().get_server_info(Box::new(
        move |server_info: &ServerInfo| {
            if let Some(sink_name) = &server_info.default_sink_name {
                println!("Default sink name: {}", sink_name);

                let sink_name_clone = sink_name.clone();
                ctx_clone.borrow().introspect().get_sink_info_by_name(
                    &sink_name_clone,
                    Box::new(|result: ListResult<&SinkInfo>| {
                        if let ListResult::Item(sink_info) = result {
                            *ACTIVE_SINK_INDEX.write().unwrap() =
                                Some(sink_info.index);
                        }
                    }),
                );
            }
        },
    ));

    let subscribtion_callback = create_subscribe_callback(
        context.borrow().introspect(),
        sender.clone(),
    );
    context
        .borrow_mut()
        .set_subscribe_callback(Some(subscribtion_callback));

    context.borrow().introspect().get_sink_info_by_name(
        DEFAULT_SINK_NAME,
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
    let introspector = Rc::new(introspector);

    Box::new(move |facility, operation, index| {
        if matches!(facility, Some(Facility::Sink) | Some(Facility::Server)) {
            println!(
                "{:?} {:?} {}\n Default: {:?}",
                facility,
                operation,
                index,
                ACTIVE_SINK_INDEX.read().unwrap()
            );
        }

        match (facility, operation, index) {
            (
                Some(Facility::Server),
                Some(Operation::Changed) | Some(Operation::New),
                i,
            ) => {
                *ACTIVE_SINK_INDEX.write().unwrap() = Some(i);

                let introspector = introspector.clone();
                let introspector_clone = introspector.clone();

                let sender = sender.clone();
                introspector.get_server_info(Box::new(
                    move |server_info: &ServerInfo| {
                        if let Some(sink_name) = &server_info.default_sink_name
                        {
                            let sender = sender.clone();
                            introspector_clone.get_sink_info_by_name(
                                sink_name,
                                Box::new(
                                    move |result: ListResult<&SinkInfo>| {
                                        if let ListResult::Item(sink_info) =
                                            result
                                        {
                                            *ACTIVE_SINK_INDEX
                                                .write()
                                                .unwrap() =
                                                Some(sink_info.index);

                                            create_result_handler(
                                                sender.clone(),
                                            )(
                                                ListResult::Item(sink_info)
                                            );
                                        }
                                    },
                                ),
                            );
                        }
                    },
                ));
            }
            (Some(Facility::Server), Some(Operation::Removed), i) => {
                *ACTIVE_SINK_INDEX.write().unwrap() = Some(i);

                introspector.get_sink_info_by_index(
                    index,
                    create_result_handler(sender.clone()),
                );
            }
            (
                Some(Facility::Sink),
                Some(Operation::Changed) | Some(Operation::New),
                i,
            ) => {
                if *ACTIVE_SINK_INDEX.read().unwrap() == Some(i) {
                    introspector.get_sink_info_by_index(
                        index,
                        create_result_handler(sender.clone()),
                    );
                }
            }
            _ => (),
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
                error!("{e}");
            }
        }
    }
}

fn volume_to_u8(volume: Volume) -> u8 {
    (volume.0 as f32 * 100.0 / 65536.0).round() as u8
}
