#![no_main]
#![no_std]

use core::arch::{asm, global_asm};

mod uart;

macro_rules! print {
    ($($args:tt)+) => {{
        use core::fmt::Write;
        // Safety: This points to a valid UART device
        let mut uart = unsafe { crate::uart::Uart::new(0x1000_0000) };
        let _ = write!(uart, $($args)+);
    }};
}

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

global_asm!(include_str!("asm/boot.s"));
global_asm!(include_str!("asm/trap.s"));

#[allow(dead_code)]
extern "C" fn eh_personality() {}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    print!("Aborting: ");
    info.location().map(|loc| {
        let payload = info
            .payload()
            .downcast_ref::<&str>()
            .unwrap_or(&"unknown panic.");
        println!("line {}, file {}: {}", loc.line(), loc.file(), payload,);
    });
    abort();
}

#[no_mangle]
extern "C" fn abort() -> ! {
    loop {
        unsafe { asm!("wfi") }
    }
}

#[no_mangle]
extern "C" fn kmain() {
    // Init UART

    // Safety: This points to a valid UART device
    let uart = unsafe { uart::Uart::new(0x1000_0000) };
    uart.init();

    println!("Hello world!");

    // Safety: This points to a valid syscon device
    unsafe { (0x100000 as *mut u16).write_volatile(0x5555) };
}
