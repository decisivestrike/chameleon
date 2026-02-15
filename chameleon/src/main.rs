mod app;
mod hot_reload;
mod instance_manager;

use crate::app::Chameleon;

use grapes::glib::{self};

fn init_logger() {
    env_logger::builder().format_timestamp(None).init();
}

fn main() -> glib::ExitCode {
    // console_subscriber::init();
    init_logger();

    // TODO: replace ~ on home

    let app = Chameleon::new(argh::from_env());

    app.run()
}
