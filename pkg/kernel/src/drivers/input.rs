use alloc::string::String;
use crossbeam_queue::ArrayQueue;
use core::str;
use alloc::vec::Vec;
use x86_64::instructions::interrupts;

use crate::proc::get_process_manager;

type Key = u8;

lazy_static! {
    static ref INPUT_BUF: ArrayQueue<Key> = ArrayQueue::new(128);
}

// ////////////////////TEST////////////////////
// pub fn init() {
//     // 假设这个函数在程序的初始化阶段被调用
//     INPUT_BUF.push(64).expect("Failed to push to INPUT_BUF");
// }
// ////////////////////////////////////////////

#[inline]
pub fn push_key(key: Key) {
    if INPUT_BUF.push(key).is_err() {
        warn!("Input buffer is full. Dropping key '{:?}'", key);
    }
}

// #[inline]
// pub fn pop_key(key: Key) {
//     if try_pop_key(key).is_err() {
//         warn!("Input buffer is full. Dropping key '{:?}'", key);
//     }
// }

#[inline]
pub fn try_pop_key() -> Option<Key> {
    INPUT_BUF.pop()
}

//简单版本，只能处理英文和数字输入
#[inline]
pub fn get_line() -> String {
    let mut line = String::with_capacity(128);
    loop{
        if let Some(key) = try_pop_key(){ // 如果try_pop_key()返回值是Some(key)，则进入大括号中的代码块，否则跳过
            match key{
                0xD => {
                    line.push(key as char);
                    break
                },
                0x08 | 0x7F =>{
                    if !line.is_empty(){
                        line.pop();
                    }
                },
                _ => line.push(key as char),
            }
            //info!("match key successfully");
        }
    }
    line
}

// #[inline]
// pub fn get_line() -> String {
//     let mut line = String::with_capacity(128);
//     let mut utf8_buf: Vec<u8> = Vec::new(); // 临时存储UTF-8序列
//     loop {
//         if let Some(byte) = try_pop_key() {
//             match byte {
//                 0xD => {
//                     if let Ok(s) = str::from_utf8(&utf8_buf) {
//                         line.push_str(s);
//                     }
//                     break;
//                 },
//                 0x08 | 0x7F => { 
//                     // 这里的处理比较简单，实际上UTF-8字符可能需要删除多个字节
//                     line.pop();
//                 },
//                 _ => utf8_buf.push(byte), // 将字节加入到UTF-8缓冲区
//             }
            
//             // 尝试将缓冲区解码为UTF-8字符串
//             if let Ok(s) = str::from_utf8(&utf8_buf) {
//                 line.push_str(s);
//                 utf8_buf.clear(); // 清空缓冲区，准备下一个字符的接收
//             }
//             info!("{}",byte);
//         }
//         else{
//             break;
//         }
//     }

//     line
// }

#[inline]
pub fn get_char_as_u8() -> u8 { 
    if let Some(key) = try_pop_key(){ 
        return key
    }
    0
}