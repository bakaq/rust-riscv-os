use crate::uart::Uart;

pub struct Console {
    cursor_line: usize,
    cursor_col: usize,
    uart: Uart,
}

#[non_exhaustive]
enum ConsoleCommand {
    Character(char),
    Newline,
    Backspace,
    UpArrow,
    RightArrow,
    DownArrow,
    LeftArrow,
    CtrlC,
    CtrlD,
    UnknownEscape(u8),
    Byte(u8),
    Unknown,
}

impl Console {
    pub fn new(uart: Uart) -> Self {
        Self {
            cursor_line: 1,
            cursor_col: 1,
            uart,
        }
    }

    pub fn start(&mut self) -> ! {
        use ConsoleCommand as CC;

        println!("Hello world!");
        println!("Press Ctrl-C or Ctrl-D to shutdown.");

        loop {
            match self.get_console_command() {
                CC::Character(c) => print!("{c}"),
                CC::Newline => println!(),
                CC::Backspace => print!("{} {}", 8 as char, 8 as char),
                CC::UpArrow => println!("Up arrow"),
                CC::RightArrow => println!("Right arrow"),
                CC::DownArrow => println!("Down arrow"),
                CC::LeftArrow => println!("Right arrow"),
                CC::CtrlC | CC::CtrlD => {
                    println!();
                    crate::shutdown();
                },
                CC::Byte(b) => println!("Byte: {}", b),
                CC::UnknownEscape(b) => println!("Unknown escape: {} {}", b, b as char),
                _ => println!("Unknown console command"),
            }
        }
    }

    fn get_console_command(&mut self) -> ConsoleCommand {
        use ConsoleCommand as CC;
        match self.uart.get_blocking() {
            3 => CC::CtrlC,
            4 => CC::CtrlD,
            8 | 127 => CC::Backspace,
            b'\n' | b'\r' => CC::Newline,
            0x1b => {
                if self.uart.get_blocking() == b'[' {
                    match self.uart.get_blocking() {
                        b'A' => CC::UpArrow,
                        b'B' => CC::DownArrow,
                        b'C' => CC::RightArrow,
                        b'D' => CC::LeftArrow,
                        b @ _ => CC::UnknownEscape(b),
                    }
                } else {
                    CC::Unknown
                }
            }
            c @ _ => {
                if !c.is_ascii_control() {
                    CC::Character(c as char)
                } else {
                    CC::Byte(c)
                }
            }
        }
    }
}
