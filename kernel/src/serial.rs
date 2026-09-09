//! Serial port logging over COM1.
//!
//! The standard PC serial port is redirected by QEMU to stdio (`-serial
//! stdio`), which makes it the primary debug channel: it works before any
//! other output exists and stays available headless. Phase 1 is single-core
//! with interrupts still disabled, so a plain `spin::Mutex` is sufficient;
//! later phases must revisit locking once interrupt handlers use this port.

use core::fmt;
use spin::Mutex;
use uart_16550::SerialPort;

// SAFETY: COM1 (I/O port 0x3F8) is the conventional PC serial port and QEMU
// exposes it via `-serial stdio`. Exclusive access is mediated by the mutex;
// no other code touches this port.
static SERIAL_PORT: Mutex<SerialPort> = Mutex::new(unsafe { SerialPort::new(0x3F8) });

/// Initializes COM1. Must be called once before any `serial_print!`.
pub fn init() {
    SERIAL_PORT.lock().init();
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use fmt::Write;
    // Serial writes are infallible in practice; a debug log must never panic.
    let _ = SERIAL_PORT.lock().write_fmt(args);
}

/// Prints to COM1 without a trailing newline.
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => ($crate::serial::_print(core::format_args!($($arg)*)));
}

/// Prints to COM1 with a trailing newline.
#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($($arg:tt)*) => ($crate::serial_print!("{}\n", core::format_args!($($arg)*)));
}
