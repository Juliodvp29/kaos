//! Kaos kernel entry stub (Phase 0).
//!
//! This module only proves that the freestanding (`no_std`) toolchain setup
//! works. It does not boot yet: the real bootloader integration and the boot

#![no_std]
#![no_main]

/// Phase 0 placeholder entry point.
///
/// It intentionally does nothing but halt the CPU. Phase 1 replaces it with
/// the `bootloader_api` entry point.
#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    loop {
        core::hint::spin_loop();
    }
}

/// Kernel panic handler (Phase 0: halt on panic).
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
