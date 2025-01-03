#![no_std]
#![no_main]

use log::info;
use ysos::{proc::print_process_list, *};
use ysos_kernel as ysos;

extern crate alloc;

boot::entry_point!(kernel_main);

pub fn kernel_main(boot_info: &'static boot::BootInfo) -> ! {
    ysos::init(boot_info);
    ysos::wait(spawn_init());
    ysos::shutdown(boot_info);
}

pub fn spawn_init() -> proc::ProcessId {
    // NOTE: you may want to clear the screen before starting the shell
    // print_serial!("\x1b[1;1H\x1b[2J");

    proc::list_app();
    //proc::spawn("hello").unwrap()
    proc::spawn("dinner").unwrap()
}
