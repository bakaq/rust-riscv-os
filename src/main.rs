#![no_main]
#![no_std]


#![allow(dead_code)]
#![allow(unused_variables)]


use core::arch::{asm, global_asm};

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


global_asm!(include_str!("asm/boot.s"));
global_asm!(include_str!("asm/trap.s"));

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
extern "C" fn kmain() { 
    let _ = 1 + 2;
}
