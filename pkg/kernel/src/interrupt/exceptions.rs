use crate::memory::*;
use x86_64::registers::control::Cr2;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use x86_64::VirtAddr;

pub unsafe fn register_idt(idt: &mut InterruptDescriptorTable) {
    idt.divide_error.set_handler_fn(divide_error_handler);
    idt.double_fault
        .set_handler_fn(double_fault_handler)
        .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
    idt.page_fault
        .set_handler_fn(page_fault_handler)
        .set_stack_index(gdt::PAGE_FAULT_IST_INDEX);

    // TODO: you should handle more exceptions here
    // especially gerneral protection fault (GPF)
    // see: https://wiki.osdev.org/Exceptions

    idt.debug.set_handler_fn(debug_handler);
    idt.non_maskable_interrupt.set_handler_fn(nmi_handler);
    idt.overflow.set_handler_fn(overflow_handler);
    idt.breakpoint.set_handler_fn(breakpoint_handler);
    idt.bound_range_exceeded.set_handler_fn(bound_range_exceeded_handler);
    idt.device_not_available.set_handler_fn(device_not_available_handler);
    idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
    idt.invalid_tss.set_handler_fn(invalid_tss_handler);
    idt.segment_not_present.set_handler_fn(segment_not_present_handler);
    idt.stack_segment_fault.set_handler_fn(stack_segment_fault_handler);
    idt.general_protection_fault.set_handler_fn(general_protection_fault_handler);
    idt.x87_floating_point.set_handler_fn(x87_floating_point_handler);
    idt.alignment_check.set_handler_fn(alignment_check_handler);
    idt.machine_check.set_handler_fn(machine_check_handler);
    idt.simd_floating_point.set_handler_fn(simd_floating_point_handler);
    idt.virtualization.set_handler_fn(virtualization_handler);
    idt.security_exception.set_handler_fn(security_exception_handler);


}

pub extern "x86-interrupt" fn debug_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: DEBUG ERROR\n\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn nmi_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: NMI ERROR\n\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn bound_range_exceeded_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: BOUND RANGE EXCEEDED ERROR\n\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn device_not_available_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: DEVICE NOT AVAILABLE ERROR\n\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn invalid_tss_handler(stack_frame: InterruptStackFrame,num: u64) {
    info!("invalid_tss_handler number:{}",num);
    panic!("EXCEPTION: INVALID TSS ERROR\n\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn overflow_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: OVERFLOW ERROR\n\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn segment_not_present_handler(stack_frame: InterruptStackFrame,num: u64) {
    info!("segment_not_present_handler number:{}",num);
    panic!("EXCEPTION: SEGMENT NOT PRESENT ERROR\n\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn stack_segment_fault_handler(stack_frame: InterruptStackFrame,num: u64) {
    info!("stack_segment_fault_handler number:{}",num);
    panic!("EXCEPTION: STACK SEGMENT FAULT ERROR\n\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn x87_floating_point_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: X87 FLOATING POINT ERROR\n\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn alignment_check_handler(stack_frame: InterruptStackFrame,num: u64) {
    info!("alignment_check_handler number:{}",num);
    panic!("EXCEPTION: ALIGNMENT CHECK ERROR\n\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn machine_check_handler(stack_frame: InterruptStackFrame)->! {
    panic!("EXCEPTION: MACHINE CHECK ERROR\n\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn simd_floating_point_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: SIMD FLOATING POINT ERROR\n\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn virtualization_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: VIRTUALIZATION ERROR\n\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn security_exception_handler(stack_frame: InterruptStackFrame,num: u64) {
    info!("security_exception_handler number:{}",num);
    panic!("EXCEPTION: SECUIRITY EXCEPTION ERROR\n\n{:#?}", stack_frame);
}

/////////////////////////////////////////////////////////////////////////////////////

pub extern "x86-interrupt" fn divide_error_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: DIVIDE ERROR\n\n{:#?}", stack_frame);
}

pub extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) -> ! {
    panic!(
        "EXCEPTION: DOUBLE FAULT, ERROR_CODE: 0x{:016x}\n\n{:#?}",
        error_code, stack_frame
    );
}

pub extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    err_code: PageFaultErrorCode,
) {
    if !crate::proc::handle_page_fault(Cr2::read().unwrap_or(VirtAddr::new(0xdeadbeef)), err_code) {
        warn!(
            "EXCEPTION: PAGE FAULT, ERROR_CODE: {:?}\n\nTrying to access: {:#x}\n{:#?}",
            err_code,
            Cr2::read().unwrap_or(VirtAddr::new(0xdeadbeef)),
            stack_frame
        );
        // FIXME: print info about which process causes page fault?
        panic!("Cannot handle page fault!");
    }
}
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: BREAKPOINT\n\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn invalid_opcode_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: INVALID OPCODE\n\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn general_protection_fault_handler(stack_frame: InterruptStackFrame, error_code: u64) {
    panic!(
        "EXCEPTION: GENERAL PROTECTION FAULT, ERROR_CODE: 0x{:016x}\n\n{:#?}",
        error_code, stack_frame
    );
}