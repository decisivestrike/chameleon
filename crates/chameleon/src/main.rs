mod app;
mod instance_manager;
mod panic_hook;

use std::{io::Write, os::unix::net::UnixStream};

use crate::{app::Chameleon, panic_hook::set_custom_panic_hook};
use chameleon_cli::ARGS;
use grapes::glib::{self};

const SOCKET_PATH: &str = "/tmp/chameleon-recv.sock";

fn init_logger() {
    env_logger::builder().format_timestamp(None).init();
}

fn main() -> glib::ExitCode {
    if !cfg!(debug_assertions) {
        set_custom_panic_hook();
    }

    // console_subscriber::init();

    if ARGS.toggle_launcher {
        let mut stream = match UnixStream::connect(SOCKET_PATH) {
            Ok(stream) => stream,
            Err(e) => {
                log::error!("{e}");
                return glib::ExitCode::new(1);
            }
        };

        if let Err(e) = stream.write_all(&[2]) {
            log::error!("{e}");
            return glib::ExitCode::new(1);
        }

        return glib::ExitCode::new(0);
    }

    // if !layer_shell::is_supported() {
    //     log::error!("Oh shit I'am sorry");
    //     return glib::ExitCode::new(2);
    // }

    init_logger();

    // TODO: replace ~ on home

    let app = Chameleon::new(&ARGS);

    app.run()
}
