#![no_main]
#![no_std]

use core::arch::asm;

macro_rules! print {
    ($($args:tt)+) => {{
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

extern "C" fn eh_personality() {}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    print!("Aborting: ");
    info.location()
        .map(|loc| {
            let payload = info.payload()
                .downcast_ref::<&str>()
                .unwrap_or(&"unknown panic.");
            println!(
                "line {}, file {}: {s}",
                loc.line(),
                loc.file(),
                payload,
            );
        });
    abort();
}

#[no_mangle]
extern "C" fn abort() -> ! {
    loop {
        unsafe {
            asm!("wfi")
        }
    }
}

#[no_mangle]
extern "C" fn kmain() { }
