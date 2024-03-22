use super::LocalApic;
use bit_field::BitField;
use core::fmt::{Debug, Error, Formatter};
use core::ptr::{read_volatile, write_volatile};
use x86::cpuid::CpuId;
use crate::interrupt::consts;

/// Default physical address of xAPIC
pub const LAPIC_ADDR: u64 = 0xFEE00000;

use bitflags::bitflags;

bitflags! {
    /// 定义Local APIC的Spurious Interrupt Vector Register标志位
    pub struct SpuriousInterruptFlags: u32 {
        const ENABLE_APIC             = 1 << 8;
    }
}

bitflags! {
    /// 定义Local APIC的LVT Timer Register标志位
    pub struct LvtTimerFlags: u32 {
        const MASKED                  = 1 << 16;
        const PERIODIC                = 1 << 17;
    }
}

bitflags! {
    /// 定义Local APIC的Error Status Register标志位
    /// 注意：Error Status Register通常用于读取错误状态，这里仅作为示例
    pub struct ErrorStatusFlags: u32 {
        const SEND_CHECKSUM_ERROR     = 1 << 0;
        const RECEIVE_CHECKSUM_ERROR  = 1 << 1;
        const SEND_ACCEPT_ERROR       = 1 << 2;
        const RECEIVE_ACCEPT_ERROR    = 1 << 3;
        // 根据APIC文档，可以继续添加更多标志位
    }
}

bitflags! {
    /// 定义Local APIC的Interrupt Command Register (ICR)标志位
    pub struct InterruptCommandFlags: u64 {
        const DESTINATION_FIELD       = 0xFF << 56;
        const DESTINATION_SHORTHAND   = 0b11 << 18;
        const TRIGGER_MODE_LEVEL      = 1 << 15;
        const TRIGGER_MODE_EDGE       = 0 << 15;
        const DELIVERY_MODE_FIXED     = 0b000 << 8;
        const DELIVERY_MODE_LOWEST    = 0b001 << 8;
        const DELIVERY_MODE_SMI       = 0b010 << 8;
        // 根据APIC文档，可以继续添加更多标志位
    }
}

// 使用示例
fn set_apic_flags() {
    let mut spurious = SpuriousInterruptFlags::empty();
    spurious.insert(SpuriousInterruptFlags::ENABLE_APIC);

    let mut lvt_timer = LvtTimerFlags::empty();
    lvt_timer.insert(LvtTimerFlags::PERIODIC);
    lvt_timer.insert(LvtTimerFlags::MASKED);

    let mut icr = InterruptCommandFlags::empty();
    icr.insert(InterruptCommandFlags::DELIVERY_MODE_FIXED);
    // 设置其他所需标志位
}

pub struct XApic {
    addr: u64,
} 

impl XApic {
    pub unsafe fn new(addr: u64) -> Self {
        XApic { addr }
    }

    unsafe fn read(&self, reg: u32) -> u32 {
        read_volatile((self.addr + reg as u64) as *const u32)
    }

    unsafe fn write(&mut self, reg: u32, value: u32) {
        write_volatile((self.addr + reg as u64) as *mut u32, value);
        self.read(0x20); // Local AIPC ID Register
    }
    unsafe fn set_lvt_timer(&mut self, flags: LvtTimerFlags, vector: u8) {
        let mut lvt_timer: u32 = self.read(0x320);
        lvt_timer.set_bits(0..=7, vector as u32); // 设置Vector
        lvt_timer.set_bits(16..=17, flags.bits() >> 16); // 应用标志位
        self.write(0x320, lvt_timer);
    }
}

impl LocalApic for XApic {
    /// If this type APIC is supported
    fn support() -> bool {
        // FIXME: Check CPUID to see if xAPIC is supported.
        CpuId::new().get_feature_info().map(|f| f.has_apic()).unwrap_or(false)
    }

    /// Initialize the xAPIC for the current CPU.
    fn cpu_init(&mut self) {
        unsafe {
            // FIXME: Enable local APIC; set spurious interrupt vector.

            let mut spiv = self.read(0xF0);
            spiv |= 1 << 8; // set EN bit
            // clear and set Vector
            spiv &= !(0xFF);  // 清除低8位
            spiv |= consts::Interrupts::IrqBase as u32 + consts::Irq::Spurious as u32;
            self.write(0xF0, spiv);

            // FIXME: The timer repeatedly counts down at bus frequency

            self.write(0x3E0, 0b1011); // set Timer Divide to 1
            self.write(0x380, 0x20000); // set initial count to 0x20000

            // let mut lvt_timer: u32 = self.read(0x320);
            // // clear and set Vector
            // lvt_timer &= !(0xFF);
            // lvt_timer |= consts::Interrupts::IrqBase as u32 + consts::Irq::Timer as u32; // 0x20 and 0
            // lvt_timer &= !(1 << 16); // clear Mask
            // lvt_timer |= 1 << 17; // set Timer Periodic Mode
            // self.write(0x320, lvt_timer);
            self.set_lvt_timer(LvtTimerFlags::PERIODIC, consts::Interrupts::IrqBase as u8 + consts::Irq::Timer as u8);

            // FIXME: Disable logical interrupt lines (LINT0, LINT1)

            self.write(0x350, 1 << 16);
            self.write(0x360, 1 << 16);

            // FIXME: Disable performance counter overflow interrupts (PCINT)

            self.write(0x340, 1 << 16);
 
            // FIXME: Map error interrupt to IRQ_ERROR.

            let mut lvt_error = self.read(0x370);
            lvt_error &= !(0xFF);
            lvt_error |= consts::Interrupts::IrqBase as u32 + consts::Irq::Error as u32;
            lvt_error &= !(1 << 16);
            self.write(0x370, lvt_error);

            // FIXME: Clear error status register (requires back-to-back writes).

            self.write(0x280, 0);
            self.write(0x280, 0); // 为啥要这么写

            // FIXME: Ack any outstanding interrupts.

            self.eoi(); // EOI(End of Interrupt) register

            // FIXME: Send an Init Level De-Assert to synchronise arbitration ID's.

            self.write(0x310, 0); // set ICR 0x310
            const BCAST: u32 = 1 << 19;
            const INIT: u32 = 5 << 8;
            const TMLV: u32 = 1 << 15; // TM = 1, LV = 0
            self.write(0x300, BCAST | INIT | TMLV); // set ICR 0x300
            const DS: u32 = 1 << 12;
            while self.read(0x300) & DS != 0 {} // wait for delivery status

            // FIXME: Enable interrupts on the APIC (but not on the processor).

            self.write(0x080,0); // 开中断，TPR中存储的值代表了当前任务的优先级，置0即可被其他中断打断
        }

        // NOTE: Try to use bitflags! macro to set the flags.
    }

    fn id(&self) -> u32 {
        // NOTE: Maybe you can handle regs like `0x0300` as a const.
        unsafe { self.read(0x0020) >> 24 }
    }

    fn version(&self) -> u32 {
        unsafe { self.read(0x0030) }
    }

    fn icr(&self) -> u64 {
        unsafe { (self.read(0x0310) as u64) << 32 | self.read(0x0300) as u64 }
    }

    fn set_icr(&mut self, value: u64) {
        unsafe {
            while self.read(0x0300).get_bit(12) {}
            self.write(0x0310, (value >> 32) as u32);
            self.write(0x0300, value as u32);
            while self.read(0x0300).get_bit(12) {}
        }
    }

    fn eoi(&mut self) {
        unsafe {
            self.write(0x00B0, 0);
        }
    }
}

impl Debug for XApic {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.debug_struct("Xapic")
            .field("id", &self.id())
            .field("version", &self.version())
            .field("icr", &self.icr())
            .finish()
    }
}
