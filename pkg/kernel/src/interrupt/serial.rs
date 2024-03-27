use super::consts::*;
use crate::interrupt::InterruptDescriptorTable;
use x86_64::structures::idt::InterruptStackFrame;

use crate::drivers::uart16550::SerialPort;
use crate::drivers::input::push_key;


pub unsafe fn register_idt(idt: &mut InterruptDescriptorTable) {
    idt[Interrupts::IrqBase as u8 + Irq::Serial0 as u8]
        .set_handler_fn(serial_handler);
}

pub extern "x86-interrupt" fn serial_handler(_st: InterruptStackFrame) {
    //info!("serial_handler starts");
    receive();
    super::ack();  // 向局部APIC（高级可编程中断控制器）发送一个结束中断（EOI）信号，表示当前的中断处理程序已经完成处理，并且系统可以处理下一个中断
    //info!("serial_handler ends");
}

static mut SERIAL_PORT: SerialPort = SerialPort::new(0x3F8); // 通常0x3F8是COM1端口的地址


/// Receive character from uart 16550
/// Should be called on every interrupt
fn receive() {
    // FIXME: receive character from uart 16550, put it into INPUT_BUFFER
    unsafe {
        // 循环尝试从串口接收数据
        loop {
            if let Some(byte) = SERIAL_PORT.receive() {
                push_key(byte);
                //info!("received!");
                break; // 成功接收到数据后退出循环
            }
        }
    }
}