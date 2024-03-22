use core::fmt;
use bitflags::bitflags;
use x86_64::instructions::port::{PortReadOnly, PortWriteOnly};

/// A port-mapped UART 16550 serial interface.
pub struct SerialPort{
    PORT: u16,
}

bitflags!{
    struct LcrFlags: u8{
        const ENABLE_DLAB = 0x80;
        const DATA_BITS_8 = 0x03;
    }
    struct IerFlags: u8{
        const DISABLED = 0x00;
    }
    struct FcrFlags: u8{
        const ENABLE_FIFO = 0xC7;
    }
    struct McrFlags: u8{
        const IRQS_ENABLED = 0x0B;
        const LOOPBACK_MODE = 0x1E;
        const NORMAL_OPERATION = 0x0F;
    }
}

impl SerialPort {
    pub const fn new(port: u16) -> Self {
        Self {
            PORT : port,
        }
    }

    /// Initializes the serial port.
    pub fn init(&self) -> i32{
        // FIXME: Initialize the serial port
        unsafe{
            PortWriteOnly::new(self.PORT + 1).write(IerFlags::DISABLED.bits()); // Disable all interrupts
            PortWriteOnly::new(self.PORT + 3).write(LcrFlags::ENABLE_DLAB.bits()); // Enable DLAB (set baud rate divisor)
            PortWriteOnly::new(self.PORT + 0).write(LcrFlags::DATA_BITS_8.bits()); // Set divisor to 3 (lo byte) 38400 baud
            PortWriteOnly::new(self.PORT + 1).write(IerFlags::DISABLED.bits()); // (hi byte)
            PortWriteOnly::new(self.PORT + 3).write(LcrFlags::DATA_BITS_8.bits()); // 8 bits, no parity, one stop bit
            PortWriteOnly::new(self.PORT + 2).write(FcrFlags::ENABLE_FIFO.bits()); // Enable FIFO, clear them, with 14-byte threshold
            PortWriteOnly::new(self.PORT + 4).write(McrFlags::IRQS_ENABLED.bits()); // IRQs enabled, RTS/DSR set
            PortWriteOnly::new(self.PORT + 4).write(McrFlags::LOOPBACK_MODE.bits()); // Set in loopback mode, test the serial chip
            PortWriteOnly::new(self.PORT + 0).write(0xAEu8); // Test serial chip (send byte 0xAE and check if serial returns same byte)

            // Check if serial is faulty (i.e: not same byte as sent)
            if PortReadOnly::<u8>::new(self.PORT + 0).read() != 0xAEu8 {
                return 1;
            }

            // If serial is not faulty set it in normal operation mode
            // (not-loopback with IRQs enabled and OUT#1 and OUT#2 bits enabled)
            PortWriteOnly::new(self.PORT + 4).write(McrFlags::NORMAL_OPERATION.bits());

            PortWriteOnly::new(self.PORT+1).write(0x01u8); // 实验二新增
        }
        return 0;
    }

    fn is_transmit_empty(&self) -> u8{
        unsafe{
            return PortReadOnly::<u8>::new(self.PORT + 5).read() & 0x20;
        }
    }
    /// Sends a byte on the serial port.
    pub fn send(&mut self, data: u8) {
        // FIXME: Send a byte on the serial port
        while self.is_transmit_empty() == 0{}

        unsafe{
            PortWriteOnly::new(self.PORT).write(data);
        }
    }

    fn serial_received(&self) -> u8{
        unsafe{
            return PortReadOnly::<u8>::new(self.PORT + 5).read() & 0x1;
        }
    }
    /// Receives a byte on the serial port no wait.
    pub fn receive(&mut self) -> Option<u8> {
        // FIXME: Receive a byte on the serial port no wait
        if self.serial_received() != 0 {
            unsafe {
                Some(PortReadOnly::<u8>::new(self.PORT).read())
            }
        } else {
            None
        }
    }
}

impl fmt::Write for SerialPort {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.send(byte);
        }
        Ok(())
    }
}