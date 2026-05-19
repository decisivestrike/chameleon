use crate::compositor::Compositor;
use gtkio::future::spawn_blocking;
use niri_ipc::socket::Socket;
use niri_ipc::{Event, Request, Response};
use std::io;
use tokio::sync::broadcast;

pub struct Niri {
    event_sender: broadcast::Sender<Event>,
}

impl Compositor for Niri {
    type Event = Event;

    fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.event_sender.subscribe()
    }
}

impl Niri {
    pub fn init() -> io::Result<Self> {
        let mut socket = Socket::connect()?;
        let event_sender = broadcast::Sender::new(64);

        let reply = socket.send(Request::EventStream)?;

        if matches!(reply, Ok(Response::Handled)) {
            let mut read_event = socket.read_events();

            spawn_blocking({
                let sender = event_sender.clone();
                move || {
                    loop {
                        if let Ok(event) = read_event() {
                            sender.send(event).unwrap();
                        }
                    }
                }
            });
        }

        Ok(Self { event_sender })
    }
}
