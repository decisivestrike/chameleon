use std::fs::OpenOptions;
use std::io::Write;
use std::panic::{self, PanicHookInfo};

pub fn set_custom_panic_hook() {
    panic::set_hook(Box::new(write_to_file));
}

fn write_to_file(info: &PanicHookInfo) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/chameleon-panic.log")
        .expect("cant open log file");

    file.write_all(info.to_string().as_bytes())
        .expect("cant write to file");
}
