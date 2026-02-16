mod app;
mod instance_manager;

use crate::app::Chameleon;
use chameleon_cli::ARGS;
use chameleon_ipc::COMPOSITOR;
use grapes::glib::{self};

fn init_logger() {
    env_logger::builder().format_timestamp(None).init();
}

fn main() -> glib::ExitCode {
    // console_subscriber::init();
    init_logger();

    COMPOSITOR.run();

    // TODO: replace ~ on home

    let app = Chameleon::new(&ARGS);

    app.run()
}
