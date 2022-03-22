use core::fmt::Write;
use pc_keyboard::{DecodedKey, HandleControl, Keyboard, ScancodeSet, ScancodeSet1};
use pc_keyboard::layouts::Us104Key;
use pic8259::ChainedPics;
use x86_64::instructions::port::Port;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::{debug, halt_loop, print, println, SpinlockMutex, vga_buffer};

pub const PIC_OFFSET: u8 = 32;
pub const PIC_SIZE: u8 = 8;

pub const INTERRUPT_TIMER: u8 = PIC_OFFSET;
pub const INTERRUPT_KEYBOARD: u8 = PIC_OFFSET + 1;

pub static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

pub static mut PICS: Option<SpinlockMutex<ChainedPics>> = None;

pub static mut KEYBOARD: Option<SpinlockMutex<Keyboard<Us104Key, ScancodeSet1>>> = None;

pub fn load() {
    unsafe {
        // IDT.breakpoint.set_handler_fn(breakpoint_handler);
        IDT.double_fault.set_handler_fn(doublefault_handler);
        IDT[INTERRUPT_TIMER as usize].set_handler_fn(timer_interrupt_handler);
        IDT[INTERRUPT_KEYBOARD as usize].set_handler_fn(keyboard_interrupt_handler);
        IDT.load();

        PICS = Some(SpinlockMutex::new(ChainedPics::new(PIC_OFFSET, PIC_OFFSET + PIC_SIZE)));

        KEYBOARD = Some(SpinlockMutex::new(Keyboard::new(Us104Key, ScancodeSet1, HandleControl::Ignore)))
    }
}

pub fn notify_pic(index: u8) {
    unsafe {PICS.as_ref().unwrap().spinlock().notify_end_of_interrupt(index);}
}

extern "x86-interrupt" fn keyboard_interrupt_handler(stack_frame: InterruptStackFrame) {
    let mut port = Port::<u8>::new(0x60);
    let mut keyboard = unsafe {KEYBOARD.as_ref().unwrap().spinlock()};

    let scancode = unsafe {port.read()};

    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => {
                    match character as usize {
                        0x08 => {
                            vga_buffer::backspace();
                        }
                        _ => {
                            print!("{}", character);
                        }
                    }
                }
                DecodedKey::RawKey(key) => println!("{:?}", key),
            }
        }
    }

    notify_pic(INTERRUPT_KEYBOARD);
}

extern "x86-interrupt" fn timer_interrupt_handler(stack_frame: InterruptStackFrame) {
    // print!(".");

    notify_pic(INTERRUPT_TIMER);
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    debug!("breakpoint, {:?}", stack_frame)
}

extern "x86-interrupt" fn doublefault_handler(stack_frame: InterruptStackFrame, error_code: u64) -> ! {
    vga_buffer::fatal("Double Fault Error!\n");
    vga_buffer::fatal("");
    vga_buffer::write_fmt(format_args!("Stack Frame: {:?}", stack_frame));
    vga_buffer::fatal("");
    vga_buffer::write_fmt(format_args!("Error Code: {:?}", error_code));

    halt_loop()
}