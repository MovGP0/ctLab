mod register_port;
pub use register_port::RegisterPort;
mod mcu;
pub use mcu::Mcu;
mod atmega32;
pub use atmega32::Atmega32;
mod atmega644;
pub use atmega644::Atmega644;
mod avrd_port_io;
pub use avrd_port_io::AvrdPortIo;

use core::marker::PhantomData;
use core::ptr;
use core::sync::atomic::{compiler_fence, Ordering};

pub unsafe fn read_u8(register: *mut u8) -> u8 {
    ptr::read_volatile(register)
}

pub unsafe fn write_u8(register: *mut u8, value: u8) {
    ptr::write_volatile(register, value);
}

pub unsafe fn update_u8(register: *mut u8, f: impl FnOnce(u8) -> u8) {
    let value = read_u8(register);
    write_u8(register, f(value));
}

pub const fn set_or_clear_bit(value: u8, bit: u8, high: bool) -> u8 {
    if high {
        value | (1 << bit)
    } else {
        value & !(1 << bit)
    }
}
