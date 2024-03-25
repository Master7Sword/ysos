use super::consts::*;
use x86_64::structures::idt::InterruptStackFrame;
use x86_64::structures::idt::InterruptDescriptorTable;
use core::sync::atomic::{AtomicU64, Ordering};
use crate::memory::gdt;

pub unsafe fn register_idt(idt: &mut InterruptDescriptorTable) {
    idt[Interrupts::IrqBase as usize + Irq::Timer as usize]
        .set_handler_fn(clock_handler).set_stack_index(gdt::CLOCK_INTERRUPT_INDX);
}

pub extern "x86-interrupt" fn clock_handler(_sf: InterruptStackFrame) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        if inc_counter() % 0x2000 == 0 {  // 数字不能太大
            // info!("Tick! @{}", read_counter());
        }
        super::ack();
    });
}

pub static COUNTER: AtomicU64 = AtomicU64::new(0);

#[inline]
pub fn read_counter() -> u64 {
    COUNTER.load(Ordering::SeqCst)
}

#[inline]
pub fn inc_counter() -> u64 {
    COUNTER.fetch_add(1, Ordering::SeqCst) + 1
}