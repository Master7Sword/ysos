#![no_std]
#![no_main]

use lib::{sync::{Semaphore, SpinLock}, utils::sleep, *};

extern crate lib;

const THREAD_COUNT: usize = 5;
static mut CHOPSTICKS: [Semaphore; THREAD_COUNT] = [
    Semaphore::new(0),
    Semaphore::new(1),
    Semaphore::new(2),
    Semaphore::new(3),
    Semaphore::new(4),
];
static mut PHILOSOPHERS: Semaphore = Semaphore::new(5);


fn main() -> isize {
    for i in 0..THREAD_COUNT {
        unsafe {
            CHOPSTICKS[i].init(1);
        }
    }
    unsafe {PHILOSOPHERS.init(4);}
    let mut pids = [0u16; THREAD_COUNT];
    for i in 0..THREAD_COUNT {
        let pid = sys_fork();
        if pid == 0 {
            philosopher(i);
            sys_exit(0);
        } else {
            pids[i] = pid;
        }
    }
    for i in 0..THREAD_COUNT {
        sys_wait_pid(pids[i]);
    }
    println!("All philosophers finished eating");
    0
}


fn philosopher(id: usize) {
    for _ in 0..20{
        unsafe{
            //PHILOSOPHERS.wait();
            CHOPSTICKS[id].wait();
            //sleep(3);
            CHOPSTICKS[(id + 1) % THREAD_COUNT].wait();
            println!("Philosopher #{} starts eating.", id);
            sleep(1000);
            println!("Philosopher #{} finishes eating and starts thinking.", id);
            CHOPSTICKS[id].signal();
            CHOPSTICKS[(id + 1) % THREAD_COUNT].signal();
            //PHILOSOPHERS.signal();
        }   
    }
}

entry!(main);
