#![no_std]
#![no_main]
#![feature(
    custom_test_frameworks,
    panic_info_message,
    fmt_internals,
    abi_x86_interrupt
)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

pub mod vga_buffer;
pub mod spin_mutex;
pub mod interrupts;
pub mod gdt;

use core::fmt::{Arguments, Display, Formatter, Write};
use core::ops::DerefMut;
use core::panic::PanicInfo;
use core::ptr::null_mut;
use x86_64::instructions::port::Port;
use crate::spin_mutex::SpinlockMutex;
use crate::vga_buffer::{_print, Color, ColorText, VGAWriter};

static mut VGA_BUFFER: Option<SpinlockMutex<VGAWriter>> = None;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    vga_buffer::load();
    vga_buffer::info("VGA Buffer loaded");

    interrupts::load();
    vga_buffer::info("Interrupts loaded");

    gdt::load();
    vga_buffer::info("GDT loaded");

    unsafe { interrupts::PICS.as_ref().unwrap().spinlock().initialize() }
    x86_64::instructions::interrupts::enable();
    vga_buffer::info("PICS loaded");

    vga_buffer::info("");

    #[cfg(test)]
        test_main();


    halt_loop()
}

fn exit_qemu(code: u32) {
    unsafe {
        let mut io_port = Port::new(0xf4);
        io_port.write(code);
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    vga_buffer::fatal("");
    vga_buffer::write_fmt(format_args!("{}", _info));
    halt_loop()
}

fn halt_loop() -> ! {
    loop { x86_64::instructions::hlt(); }
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    vga_buffer::get().debug("");
    println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
}