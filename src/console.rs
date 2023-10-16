use crate::uart::Uart;

pub struct Console {
    cursor_line: usize,
    cursor_col: usize,
    command_buffer: Option<u8>,
    uart: Uart,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ConsoleCommand {
    Character(char),
    Backspace,
    UpArrow,
    RightArrow,
    DownArrow,
    LeftArrow,
    CtrlC,
    CtrlD,
    Esc,
    UnknownEscape(u8),
    Byte(u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CsiEscapeSequence {
    args: [u32;2],
    function: char,
}

impl CsiEscapeSequence {
    fn from_ansi_escape(s: &str) -> Result<Self, ()> {
        let mut chars = s.chars();

        if chars.next() != Some('\x1b') {
            return Err(())
        }
        if chars.next() != Some('[') {
            return Err(())
        }
        
        let mut args = [0, 0];
        let mut arg_idx = 0;
        let mut current_char = chars.next().ok_or(())?;

        while arg_idx < args.len()
            && (current_char.is_ascii_digit() || current_char == ';')
        {
            if current_char.is_ascii_digit() {
                args[arg_idx] = args[arg_idx] * 10 + current_char.to_digit(10).unwrap()
            } else {
                assert!(current_char == ';');
                arg_idx += 1;
            }
            current_char = chars.next().ok_or(())?;
        }
        
        if 0x40 <= current_char as u8
            && current_char as u8 <= 0x7F
            && chars.next() == None
        {
            Ok(CsiEscapeSequence {
                args,
                function: current_char,
            })
        } else {
            Err(())
        }
    }
}

impl Console {
    pub fn new(uart: Uart) -> Self {
        Self {
            cursor_line: 1,
            cursor_col: 1,
            command_buffer: None,
            uart,
        }
    }

    pub fn start(&mut self) -> ! {
        println!("Press Ctrl-C or Ctrl-D to shutdown.");

        loop {
            self.prompt();
        }
    }

    fn prompt(&mut self) {
        use ConsoleCommand as CC;

        print!("$ ");
        loop {
            match self.get_console_command() {
                CC::Character(c) => print!("{c}"),
                CC::Backspace => print!("{} {}", 8 as char, 8 as char),
                CC::UpArrow => println!("Up arrow"),
                CC::RightArrow => println!("Right arrow"),
                CC::DownArrow => println!("Down arrow"),
                CC::LeftArrow => println!("Right arrow"),
                CC::CtrlC | CC::CtrlD => {
                    println!();
                    crate::shutdown();
                },
                CC::Esc => println!("Esc"),
                CC::Byte(b) => println!("Byte: {}", b),
                CC::UnknownEscape(b) => println!("Unknown escape: {} {}", b, b as char),
            }
        }
    }

    fn put_console_command(&mut self, command: ConsoleCommand) {
        use ConsoleCommand as CC;
        match command {
            CC::Character(c) => print!("{}", c),
            _ => {}
        }
    }

    fn get_console_command(&mut self) -> ConsoleCommand {
        use ConsoleCommand as CC;

        match self.command_buffer.take().unwrap_or_else(|| self.uart.get_blocking()) {
            3 => CC::CtrlC,
            4 => CC::CtrlD,
            8 | 127 => CC::Backspace,
            0x1b => {
                let bracket = self.uart.get_blocking();
                if bracket == b'[' {
                    match self.uart.get_blocking() {
                        b'A' => CC::UpArrow,
                        b'B' => CC::DownArrow,
                        b'C' => CC::RightArrow,
                        b'D' => CC::LeftArrow,
                        0x1b => CC::Esc,
                        b @ _ => CC::UnknownEscape(b),
                    }
                } else {
                    self.command_buffer = Some(bracket);
                    CC::Esc
                }
            }
            c @ _ => {
                if !c.is_ascii_control() {
                    CC::Character(c as char)
                } else if c == b'\r' {
                    CC::Character('\n')
                } else {
                    CC::Byte(c)
                }
            }
        }
    }
}
