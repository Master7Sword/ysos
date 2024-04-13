// use super::consts::*;
// use x86_64::structures::idt::InterruptStackFrame;
// use x86_64::structures::idt::InterruptDescriptorTable;
// use core::sync::atomic::{AtomicU64, Ordering};
// use crate::memory::gdt;
// use crate::proc::ProcessContext;

// pub unsafe fn register_idt(idt: &mut InterruptDescriptorTable) {
//     idt[Interrupts::IrqBase as u8 + Irq::Timer as u8]
//         .set_handler_fn(clock_handler).set_stack_index(gdt::CLOCK_INTERRUPT_INDX);
// }

// pub extern "x86-interrupt" fn clock(mut context: ProcessContext) {
//     x86_64::instructions::interrupts::without_interrupts(|| {
//         crate::proc::switch(&mut context);
//         super::ack();
//     });
// }

// as_handler!(clock);

use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::memory::gdt;
use crate::proc::ProcessContext;

use super::consts::*;

pub unsafe fn reg_idt(idt: &mut InterruptDescriptorTable) {
    idt[Interrupts::IrqBase as u8 + Irq::Timer as u8].set_handler_fn(teapot_handler).set_stack_index(gdt::CLOCK_INTERRUPT_INDX);
}

pub extern "C" fn teapot(mut context: ProcessContext) {
    crate::proc::switch(&mut context);
    //info!("clock");
    super::ack();
}

as_handler!(teapot);