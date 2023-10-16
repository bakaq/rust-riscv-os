#![no_main]
#![no_std]

#[macro_export]
macro_rules! print {
    ($($args:tt)+) => {{
        use core::fmt::Write;
        // Safety: This points to a valid UART device
        let mut uart = unsafe { crate::uart::Uart::new(0x1000_0000) };
        let _ = write!(uart, $($args)+);
    }};
}

#[macro_export]
macro_rules! println {
    () => {
        print!("\n")
    };
    ($fmt:expr) => {{
        print!(concat!($fmt, "\n"))
    }};
    ($fmt:expr, $($args:tt)+) => {{
        print!(concat!($fmt, "\n"), $($args)+)
    }};
}

mod uart;
mod console;

use core::arch::{asm, global_asm};

global_asm!(include_str!("asm/boot.s"));
global_asm!(include_str!("asm/trap.s"));

#[allow(dead_code)]
extern "C" fn eh_personality() {}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("{}", info);
    abort();
}

#[no_mangle]
extern "C" fn abort() -> ! {
    // TODO: Is this adequate?
    shutdown()
}

#[no_mangle]
extern "C" fn kmain() {
    // Init UART
    // Safety: This points to a valid UART device
    let uart = unsafe { uart::Uart::new(0x1000_0000) };
    uart.init();

    console::Console::new(uart).start();
}

fn shutdown() -> ! {
    // Safety: This points to a valid syscon device
    unsafe { (0x100000 as *mut u16).write_volatile(0x5555) };
    loop {
        unsafe { asm!("wfi") }
    }
}
