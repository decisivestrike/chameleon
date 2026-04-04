mod app;
mod instance_manager;

use std::{io::Write, os::unix::net::UnixStream};

use crate::app::Chameleon;
use chameleon_cli::ARGS;
use grapes::glib::{self};

fn init_logger() {
    env_logger::builder().format_timestamp(None).init();
}

fn main() -> glib::ExitCode {
    // console_subscriber::init();

    if ARGS.toggle_launcher {
        let socket_path = &format!("/tmp/chameleon-recv.sock");

        let mut stream = UnixStream::connect(socket_path).unwrap();
        stream.write_all(&[2]).unwrap();

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
