mod apic;
mod consts;
pub mod clock;
mod serial;
mod exceptions;

use apic::*;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::memory::physical_to_virtual;

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            exceptions::register_idt(&mut idt);
            clock::register_idt(&mut idt);
            serial::register_idt(&mut idt);
            //info!("IDT loaded!");
        }
        idt
    };
}

// 这一个用来触发 Triple Fault
// lazy_static! {
//     static ref IDT: InterruptDescriptorTable = {
//         let mut idt = InterruptDescriptorTable::new();
//         // 注册一个错误的IDT入口
//         unsafe {
//             let handler: extern "x86-interrupt" fn(InterruptStackFrame) = core::mem::transmute(0xDEADBEEFusize);
//             idt.breakpoint.set_handler_fn(handler);
//         }
//         idt
//     };
// }

/// init interrupts system
pub fn init() {
    IDT.load();

    // FIXME: check and init APIC
    let mut Apic = unsafe{XApic::new(physical_to_virtual(LAPIC_ADDR))};
    if XApic::support(){
        Apic.cpu_init();
    }
    
    // FIXME: enable serial irq with IO APIC (use enable_irq)
    enable_irq(consts::Irq::Serial0 as u8, 0);

    info!("Interrupts Initialized.");
}

#[inline(always)]
pub fn enable_irq(irq: u8, cpuid: u8) {
    let mut ioapic = unsafe { IoApic::new(physical_to_virtual(IOAPIC_ADDR)) };
    ioapic.enable(irq, cpuid);
}

#[inline(always)]
pub fn ack() {
    let mut lapic = unsafe { XApic::new(physical_to_virtual(LAPIC_ADDR)) };
    lapic.eoi();
}
