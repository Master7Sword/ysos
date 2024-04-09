#![no_std]
#![no_main]

use lib::*;
use lib::vec::Vec;

extern crate lib;

fn main()
{
    println!("+-+-+-+-+-+-+-+- Shell v0.1 -+-+-+-+-+-+-+-+");
    loop{
        print!("> ");
        let input = stdin().read_line();
        let line: Vec<&str> = input.trim().split(' ').collect();
        if line[0] == "exit"{
            exit();
        }
        println!("{}",input);
    }
}

entry!(main);