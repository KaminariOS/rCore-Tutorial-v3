use core::fmt::{self, Write};

const STDIN: usize = 0;
const STDOUT: usize = 1;

use super::{read, read as sys_read, write, write as sys_write};
use super::{String, Vec};


struct Stdout;

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        write(STDOUT, s.as_bytes());
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    Stdout.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!($fmt $(, $($arg)+)?));
    }
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?));
    }
}

pub fn getchar() -> u8 {
    let mut c = [0u8; 1];
    read(STDIN, &mut c);
    c[0]
}



pub fn getc() -> u8 {
    let c = 0u8;
    loop {
        let len = sys_read(STDIN, &mut [c]);
        match len {
            1 => return c,
            0 => continue,
            _ => panic!("read stdin len = {}", len),
        }
    }
}

const BEL: u8 = 0x07u8;
const BS: u8 = 0x08u8;
const LF: u8 = 0x0au8;
const CR: u8 = 0x0du8;
const ESC: u8 = 0x1bu8;
const DEL: u8 = 0x7fu8;

pub fn get_line(history: &mut Vec<Vec<u8>>) -> String {
    let mut cursor = 0;
    let mut line_vec = Vec::with_capacity(512);
    let mut history_index = history.len();
    loop {
        match getc() {
            BS | DEL => {
                // Backspace
                if cursor > 0 {
                    cursor -= 1;
                    line_vec.remove(cursor);

                    putc(BS);
                    for byte in &line_vec[cursor..] {
                        putc(*byte);
                    }
                    putc(b' ');
                    for _i in cursor..line_vec.len() {
                        putc(ESC);
                        putc(b'[');
                        putc(b'D');
                    }
                    putc(ESC);
                    putc(b'[');
                    putc(b'D');
                } else {
                    putc(BEL);
                }
            }
            CR | LF => {
                // Return
                putc(CR);
                putc(LF);
                break;
            }
            ESC => {
                match getc() {
                    b'[' => {
                        match getc() {
                            b'D' => {
                                // Left arrow
                                if cursor > 0 {
                                    cursor -= 1;
                                    putc(ESC);
                                    putc(b'[');
                                    putc(b'D');
                                } else {
                                    putc(BEL);
                                }
                            }
                            b'C' => {
                                // Right arrow
                                if cursor < line_vec.len() {
                                    cursor += 1;
                                    putc(ESC);
                                    putc(b'[');
                                    putc(b'C');
                                } else {
                                    putc(BEL);
                                }
                            }
                            direction @ b'A' | direction @ b'B' => {
                                if direction == b'A' && history_index > 0 {
                                    // Up arrow
                                    history_index -= 1;
                                } else if direction == b'B' && history.len() > 0 // usize underflow
                                    && history_index < history.len() - 1
                                {
                                    // Down arrow
                                    history_index += 1;
                                } else {
                                    putc(BEL);
                                    continue;
                                }

                                for _ in 0..line_vec.len() {
                                    putc(ESC);
                                    putc(b'[');
                                    putc(b'D');
                                }
                                for _ in 0..line_vec.len() {
                                    putc(b' ');
                                }
                                for _ in 0..line_vec.len() {
                                    putc(ESC);
                                    putc(b'[');
                                    putc(b'D');
                                }
                                line_vec = history[history_index].clone();
                                cursor = line_vec.len();
                                for byte in &line_vec {
                                    putc(*byte);
                                }
                            }
                            _ => {
                                putc(BEL);
                            }
                        }
                    }
                    _ => {
                        putc(BEL);
                    }
                }
            }
            byte if byte.is_ascii_graphic() || byte == b' ' => {
                line_vec.insert(cursor, byte);
                for byte in &line_vec[cursor..] {
                    putc(*byte);
                }
                cursor += 1;
                for _i in cursor..line_vec.len() {
                    putc(ESC);
                    putc(b'[');
                    putc(b'D');
                }
            }
            _ => {
                // unrecognized characters
                putc(BEL);
            }
        }
    }

    if line_vec.len() > 0 {
        history.push(line_vec.clone());
    }
    String::from_utf8(line_vec).unwrap_or_default()
}

pub fn putc(c: u8) {
    sys_write(STDOUT, &[c]);
}
