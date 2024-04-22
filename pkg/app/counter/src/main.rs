#![no_std]
#![no_main]

use lib::{sync::{Semaphore, SpinLock}, utils::sleep, *};

extern crate lib;

const THREAD_COUNT: usize = 8;
static mut COUNTER: isize = 0;

static mut SPIN_LOCK: SpinLock = SpinLock::new();
static mut SEMAPHORE: Semaphore = Semaphore::new(0);

fn test_spin(){
    let mut pids = [0u16; THREAD_COUNT];

    for i in 0..THREAD_COUNT {
        let pid = sys_fork();
        if pid == 0 {
            do_counter_inc();
            sys_exit(0);
        } else {
            pids[i] = pid; // only parent knows child's pid
        }
    }

    let cpid = sys_get_pid();
    println!("process #{} holds threads: {:?}", cpid, &pids);
    sys_stat();

    for i in 0..THREAD_COUNT {
        println!("#{} waiting for #{}...", cpid, pids[i]);
        sys_wait_pid(pids[i]);
    }

    println!("SPIN COUNTER result: {}", unsafe { COUNTER });

}

fn test_semaphore(){
    let mut pids = [0u16; THREAD_COUNT];
    unsafe {SEMAPHORE.init(1);}

    for i in 0..THREAD_COUNT {
        let pid = sys_fork();
        if pid == 0 {
            do_counter_inc_semaphore();
            sys_exit(0);
        } else {
            pids[i] = pid; // only parent knows child's pid
        }
    }

    let cpid = sys_get_pid();
    println!("process #{} holds threads: {:?}", cpid, &pids);
    sys_stat();

    for i in 0..THREAD_COUNT {
        println!("#{} waiting for #{}...", cpid, pids[i]);
        sys_wait_pid(pids[i]);
    }

    println!("SEMAPHORE COUNTER result: {}", unsafe { COUNTER });

}

fn main() -> isize {
    //test_spin();

    //unsafe {COUNTER = 0;}
    
    test_semaphore();

    0
}

fn do_counter_inc() {
    for _ in 0..100 {
        // FIXME: protect the critical section
        unsafe {SPIN_LOCK.acquire();} 
        inc_counter();
        unsafe {SPIN_LOCK.release();}
    }
}

fn do_counter_inc_semaphore() {
    for _ in 0..100 {
        // FIXME: protect the critical section
        unsafe {SEMAPHORE.wait()}
        inc_counter();
        unsafe {SEMAPHORE.signal()}
    }
}

/// Increment the counter
///
/// this function simulate a critical section by delay
/// DO NOT MODIFY THIS FUNCTION
fn inc_counter() {
    unsafe {
        delay();
        let mut val = COUNTER;
        delay();
        val += 1;
        delay();
        COUNTER = val;  
    }
}

#[inline(never)]
#[no_mangle]
fn delay() {
    for _ in 0..0x100 {
        core::hint::spin_loop();
    }
}

entry!(main);
