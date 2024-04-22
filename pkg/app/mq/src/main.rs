#![no_std]
#![no_main]

use lib::{*, sync::Semaphore};

extern crate lib;

const THREAD_COUNT: usize = 16;
static mut MESSAGES: isize = 0;

static SEM_EMPTY: Semaphore = Semaphore::new(0);
static SEM_FULL: Semaphore = Semaphore::new(1);
static mut SEMAPHORE: Semaphore = Semaphore::new(2);

fn main() {
    let mut pids = [0u16; THREAD_COUNT];
    unsafe {
        SEMAPHORE.init(1);
        SEM_EMPTY.init(THREAD_COUNT * 2);
        SEM_FULL.init(0);
    }

    for i in 0..THREAD_COUNT{
        let pid = sys_fork();
        if pid == 0{
            if i % 2 == 0{
                for _ in 0..10{
                    produce_message();
                }
            }
            else{
                for _ in 0..10{
                    consume_message();
                }
            }
            sys_exit(0);
        }
        else {
            pids[i] = pid; // only parent knows child's pid
        }
    }
    let cpid = sys_get_pid();
    println!("process #{} holds threads: {:?}", cpid, &pids);
    //sys_stat();

    for i in 0..THREAD_COUNT {
        //println!("#{} waiting for #{}...", cpid, pids[i]);
        //sys_stat();
        sys_wait_pid(pids[i]);
        println!("{} exit, {} messages left",pids[i], unsafe{MESSAGES});
    }

    println!("finally {} message is left",unsafe {MESSAGES});
}

fn produce_message(){
    unsafe{
        SEM_EMPTY.wait();
        SEMAPHORE.wait();
        MESSAGES += 1;
        println!("produced a message, now we have {} messages",MESSAGES);
        SEMAPHORE.signal();
        SEM_FULL.signal();
    }
}

fn consume_message(){
    unsafe{
        SEM_FULL.wait();
        SEMAPHORE.wait();
        MESSAGES -= 1;
        println!("consumed a message, now we have {} messages",MESSAGES);
        SEMAPHORE.signal();
        SEM_EMPTY.signal();
    }
}

entry!(main);
