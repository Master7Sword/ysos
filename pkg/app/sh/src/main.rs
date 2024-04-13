#![no_std]
#![no_main]

use lib::*;
use lib::vec::Vec;

extern crate lib;

const HELP_INFO: &'static str = {
    "
    Commands:
        ps              | show process info
        lsapp           | show app info
        exec <app_name> | execute app
        kill <pid>      | kill process
        clear           | clear screen
        exit            | exit shell
        sleep <time>    | sleep 
"
};
    
fn main()
{
    println!("+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+- Shell v0.1 -+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+");
    println!("                                                                         by 22331067");
    println!("                                                 Enter 'help' for a list of commands");
    loop{
        print!("> ");
        let input = stdin().read_line();
        let line: Vec<&str> = input.trim().split(' ').collect();
        print!("\n");
        match line[0]{
            "help" => print!("{}",HELP_INFO),
            "ps" => {sys_stat()}, 
            "lsapp" => {sys_list_app()},
            "exec" => {sys_spawn(line[1]);},
            "kill" => {
                let pid:isize = line[1].parse().expect("invalid input, this is not a pid!");
                sys_exit(pid);
            },
            "clear" => {print!("\x1b[1;1H\x1b[2J")},
            "exit" => sys_exit(0),
            "sleep" => {
                let time:i64 = line[1].parse().expect("not a number");
                lib::utils::sleep(time)
            }
            _ => {},
        }
    }
}

entry!(main);