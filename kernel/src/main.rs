//! Kaos kernel (Phase 1): bootloader entry, serial logging, framebuffer text.
//!
//! Takes ownership of the bootloader framebuffer, prints a greeting on both
//! the screen and the serial port, then halts. Wiring only: interrupt setup
//! (Phase 2) and memory management (Phase 3) build on top of this.

#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
// The entry point, panic handler, and hardware globals are compiled out
// under `cargo test`; silence the resulting dead-code noise there only.
#![cfg_attr(test, allow(dead_code))]

mod framebuffer;
mod serial;

#[cfg(not(test))]
use bootloader_api::BootInfo;

#[cfg(not(test))]
bootloader_api::entry_point!(kernel_main);

/// Kernel entry point, invoked by the bootloader.
///
/// Runs single-core with interrupts still disabled. A missing framebuffer is
/// reported over serial and halts the machine instead of panicking.
#[cfg(not(test))]
fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    serial::init();
    serial_println!("Kaos kernel started.");

    let framebuffer = match boot_info.framebuffer.take() {
        Some(framebuffer) => framebuffer,
        None => {
            serial_println!("ERROR: bootloader provided no framebuffer; halting.");
            halt();
        }
    };

    let info = framebuffer.info();
    serial_println!(
        "Framebuffer: {}x{}, {} bytes per pixel, stride {}.",
        info.width,
        info.height,
        info.bytes_per_pixel,
        info.stride
    );
    framebuffer::init(framebuffer.into_buffer(), info);

    println!("Hello, Kaos!");
    serial_println!("Hello, Kaos!");
    serial_println!("Halting.");
    halt()
}

/// Halts the CPU forever. Spinning (instead of `hlt`) is deliberate: with no
/// interrupt descriptor table yet, any interrupt during `hlt` would fault.
#[cfg(not(test))]
fn halt() -> ! {
    loop {
        core::hint::spin_loop();
    }
}

/// Kernel panic handler: reports over serial, then halts.
///
/// Note: if the panic happens while the serial lock is already held, this
/// deadlocks instead of printing. Acceptable in Phase 1 (single-core, no
/// interrupts, tiny critical sections); revisited with interrupt-aware
/// locking in Phase 2.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    serial_println!("[PANIC] {}", info);
    halt()
}
