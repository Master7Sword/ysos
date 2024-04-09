use crate::*;
use alloc::string::{String, ToString};
use alloc::vec;

pub struct Stdin;
pub struct Stdout;
pub struct Stderr;

impl Stdin {
    fn new() -> Self {
        Self
    }

    pub fn read_char_with_buf(&self,buf: &mut [u8]) -> Option<char>{
        self::print!("11111111");
        if let Some(size) = sys_read(0, buf){
            if size > 0{
                self::print!("2222222");
                return Some(String::from_utf8_lossy(&buf[..size]).to_string().remove(0)); // remove(0)将字符串第一个字符删除并返回
            } 
        }
        None
    }

    pub fn read_line(&self) -> String {
        // FIXME: allocate string
        let mut line = String::new();

        // FIXME: read from input buffer
        //       - maybe char by char?
        // FIXME: handle backspace / enter...
        let mut buf = [0;4];
        loop{
            if let Some(char) = self.read_char_with_buf(&mut buf[..4]){
                match char{
                    '\n' =>{
                        //self::print!("enter");
                        break;
                    }
                    '\x08' => { 
                        line.pop();
                    }
                    _ => {
                        self::print!("{}",char);
                        line.push(char);
                    }
                }
            }
        }

        // FIXME: return string
        line
    }
}

impl Stdout {
    fn new() -> Self {
        Self
    }

    pub fn write(&self, s: &str) {
        sys_write(1, s.as_bytes());
    }
}

impl Stderr {
    fn new() -> Self {
        Self
    }

    pub fn write(&self, s: &str) {
        sys_write(2, s.as_bytes());
    }
}

pub fn stdin() -> Stdin {
    Stdin::new()
}

pub fn stdout() -> Stdout {
    Stdout::new()
}

pub fn stderr() -> Stderr {
    Stderr::new()
}
