#![no_std]
#![no_main]

use log::info;
use ysos::*;
use ysos_kernel as ysos;

extern crate alloc;

boot::entry_point!(kernel_main);


pub fn kernel_main(boot_info: &'static boot::BootInfo) -> ! {
    ysos::init(boot_info);

    // unsafe{
    //     core::arch::asm!("mov $rsp,{}",in(reg) 0);
    // }

    loop {
        print!("> ");
        let input = input::get_line();
        //info!("successfully get input!");
        match input.trim() {
            "exit" => break,
            _ => {
                println!("You said: {}", input);
                println!("The counter value is {}", interrupt::clock::read_counter());
            }
        }
    }

    ysos::shutdown(boot_info);
}
