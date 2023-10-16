use crate::uart::Uart;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ConsoleCommand {
    Character(char),
    Backspace,
    Esc,
    CsiEscape(CsiEscapeSequence),
    UnknownEscape,
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

    fn print_ansi_escape(&self) {
        print!("\x1b[{};{}{}", self.args[0], self.args[1], self.function);
    }
}

pub struct Console {
    command_buffer: Option<u8>,
    uart: Uart,
}

impl Console {
    pub fn new(uart: Uart) -> Self {
        Self {
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
                CC::Character('\n') => {
                    println!();
                    break;
                },
                CC::Byte(3) | CC::Byte(4) => { // Ctrl-C and Ctrl-D
                    println!();
                    crate::shutdown();
                },
                command => self.execute_console_command(command),
            }
        }
    }

    fn execute_console_command(&mut self, command: ConsoleCommand) {
        self.put_console_command(command);
    }

    fn put_console_command(&mut self, command: ConsoleCommand) {
        use ConsoleCommand as CC;
        match command {
            CC::Character(c) => print!("{c}"),
            CC::Backspace => print!("{} {}", 8 as char, 8 as char),
            CC::Esc => println!("Esc"),
            CC::Byte(b) => println!("Byte: {}", b),
            CC::UnknownEscape => println!("Unknown escape"),
            CC::CsiEscape(csi_escape) => csi_escape.print_ansi_escape(),
        }
    }

    fn get_console_command(&mut self) -> ConsoleCommand {
        use ConsoleCommand as CC;

        match self.command_buffer.take().unwrap_or_else(|| self.uart.get_blocking()) {
            127 => CC::Backspace,
            0x1b => {
                let bracket = self.uart.get_blocking();
                if bracket == b'[' {
                    let mut buffer = [0;10];
                    buffer[0] = 0x1b;
                    buffer[1] = b'[';
                    let mut buffer_idx = 2;
                    let mut current_char = self.uart.get_blocking();
                    loop {
                        buffer[buffer_idx] = current_char as u8;
                        if 0x40 <= current_char as u8
                            && current_char as u8 <= 0x7F 
                        {
                            break;
                        }

                        buffer_idx += 1;
                        current_char = self.uart.get_blocking();
                    }

                    match CsiEscapeSequence::from_ansi_escape(
                            core::str::from_utf8(&buffer[..=buffer_idx]).unwrap()
                        )
                    {
                        Ok(csi_escape) => CC::CsiEscape(csi_escape),
                        Err(_) => CC::UnknownEscape,
                    }
                } else {
                    self.command_buffer = Some(bracket);
                    CC::Esc
                }
            }
            c => {
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
