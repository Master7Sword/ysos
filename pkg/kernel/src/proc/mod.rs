mod context;
mod data;
pub mod manager;
mod paging;
mod pid;
mod process;
pub mod processor;
pub mod sync;


pub use manager::*;
use process::*;
use crate::memory::PAGE_SIZE;
use crate::proc::sync::SemaphoreResult;

use alloc::string::String;
pub use context::ProcessContext;
pub use paging::PageTableContext;
pub use data::ProcessData;
pub use pid::ProcessId;

use x86_64::structures::idt::PageFaultErrorCode;
use x86_64::VirtAddr;
use alloc::vec::Vec;
use alloc::sync::Arc;
use crate::alloc::string::ToString;
use xmas_elf::ElfFile;

// 0xffff_ff00_0000_0000 is the kernel's address space
pub const STACK_MAX: u64 = 0x0000_4000_0000_0000;

pub const STACK_MAX_PAGES: u64 = 0x100000;
pub const STACK_MAX_SIZE: u64 = STACK_MAX_PAGES * PAGE_SIZE;
pub const STACK_START_MASK: u64 = !(STACK_MAX_SIZE - 1);
// [bot..0x2000_0000_0000..top..0x3fff_ffff_ffff]
// init stack
pub const STACK_DEF_PAGE: u64 = 1;
pub const STACK_DEF_SIZE: u64 = STACK_DEF_PAGE * PAGE_SIZE;
pub const STACK_INIT_BOT: u64 = STACK_MAX - STACK_DEF_SIZE;
pub const STACK_INIT_TOP: u64 = STACK_MAX - 8;
// [bot..0xffffff0100000000..top..0xffffff01ffffffff]
// kernel stack
pub const KSTACK_MAX: u64 = 0xffff_ff02_0000_0000;
pub const KSTACK_DEF_PAGE: u64 = /* FIXME: decide on the boot config */ 512;
pub const KSTACK_DEF_SIZE: u64 = KSTACK_DEF_PAGE * PAGE_SIZE;
pub const KSTACK_INIT_BOT: u64 = KSTACK_MAX - KSTACK_DEF_SIZE;
pub const KSTACK_INIT_TOP: u64 = KSTACK_MAX - 8;

pub const KERNEL_PID: ProcessId = ProcessId(1);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ProgramStatus {
    Running,
    Ready,
    Blocked,
    Dead,
}

/// init process manager
pub fn init(boot_info: &'static boot::BootInfo) {
    let mut kproc_data = ProcessData::new();

    // FIXME: set the kernel stack
    kproc_data.set_stack(VirtAddr::new(0xffffff0100000000),KSTACK_DEF_SIZE);

    trace!("Init process data: {:#?}", kproc_data);

    // kernel process
    /* FIXME: create kernel process */
    let kproc = Process::new(String::from("kernel_process"),None,PageTableContext::new(),Some(kproc_data));
    
    // manager::init(kproc);

    // lab4 新增
    let app_list = boot_info.loaded_apps.as_ref();
    manager::init(kproc,  app_list);

    info!("Process Manager Initialized.");
}

pub fn switch(context: &mut ProcessContext) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        // FIXME: switch to the next process
        //info!("without interrupts: switched to next process");
        get_process_manager().save_current(context);
        get_process_manager().switch_next(context);
    });
}

// pub fn spawn_kernel_thread(entry: fn() -> !, name: String, data: Option<ProcessData>) -> ProcessId {
//     x86_64::instructions::interrupts::without_interrupts(|| {
//         let entry = VirtAddr::new(entry as usize as u64);
//         get_process_manager().spawn_kernel_thread(entry, name, data)
//     })
// }

pub fn print_process_list() {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager().print_process_list();
    })
}

pub fn env(key: &str) -> Option<String> {
    x86_64::instructions::interrupts::without_interrupts(|| {
        // FIXME: get current process's environment variable
        get_process_manager().current().read().env(key)
    })
}

pub fn process_exit(ret: isize) -> ! {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager().kill_current(ret);
    });

    loop {
        x86_64::instructions::hlt();
    }
}

pub fn handle_page_fault(addr: VirtAddr, err_code: PageFaultErrorCode) -> bool {
    x86_64::instructions::interrupts::without_interrupts(|| {
        get_process_manager().handle_page_fault(addr, err_code)
    })
}

// lab4新增

pub fn list_app() {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let app_list = get_process_manager().app_list();
        if app_list.is_none() {
            println!("[!] No app found in list!");
            return;
        }

        let apps = app_list
            .unwrap()
            .iter()
            .map(|app| app.name.as_str())
            .collect::<Vec<&str>>()
            .join(", ");

        // TODO: print more information like size, entry point, etc.

        println!("[+] App list: {}", apps);
    });
}

pub fn spawn(name: &str) -> Option<ProcessId> {
    info!("start to spawn");
    let app = x86_64::instructions::interrupts::without_interrupts(|| {
        let app_list = get_process_manager().app_list()?;
        app_list.iter().find(|&app| app.name.eq(name))
    })?;
    elf_spawn(name.to_string(), &app.elf)
}

pub fn elf_spawn(name: String, elf: &ElfFile) -> Option<ProcessId> {
    let pid = x86_64::instructions::interrupts::without_interrupts(|| {
        let manager = get_process_manager();
        let process_name = name.to_lowercase();
        let parent = Arc::downgrade(&manager.current());
        let pid = manager.spawn(elf, name, Some(parent), None);
        debug!("Spawned process: {}#{}", process_name, pid);
        pid
    });
    Some(pid)
}

// lab4新增

// pub fn read(fd: u8, buf: &mut [u8]) -> isize {
//     x86_64::instructions::interrupts::without_interrupts(|| get_process_manager().read(fd, buf))
// }

// pub fn write(fd: u8, buf: &[u8]) -> isize {
//     x86_64::instructions::interrupts::without_interrupts(|| get_process_manager().write(fd, buf))
// }

pub fn exit(id: isize, context: &mut ProcessContext) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let manager = get_process_manager();
        // FIXME: implement this for ProcessManager
        if id == 0{
            manager.kill_self(64);
            manager.switch_next(context);
        }else{
            manager.kill(ProcessId(id as u16), 64);
        }
        //manager.kill_self(id);
    })
}

#[inline]
pub fn still_alive(pid: ProcessId) -> bool {
    x86_64::instructions::interrupts::without_interrupts(|| {
        // check if the process is still alive
        get_process_manager().get_proc(&pid).unwrap().read().status() != ProgramStatus::Dead
    })
}

// lab5新增
pub fn fork(context: &mut ProcessContext) -> u16 {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let manager = get_process_manager();
        // FIXME: save_current as parent
        manager.save_current(context);
        // FIXME: fork to get child
        let child_pid: ProcessId = manager.fork();
        // FIXME: push to child & parent to ready queue
        manager.push_ready(child_pid);
        // FIXME: switch to next process
        manager.switch_next(context);
        u16::from(child_pid)
    })
}

pub fn sem_wait(key: u32, context: &mut ProcessContext) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let manager = get_process_manager();
        let pid = processor::current().get_pid().unwrap();
        let ret = manager.current().write().sem_wait(key, pid);
        match ret {
            SemaphoreResult::Ok => context.set_rax(0),
            SemaphoreResult::NotExist => context.set_rax(1),
            SemaphoreResult::Block(pid) => {
                // FIXME: save, block it, then switch to next
                //        maybe use `save_current` and `switch_next`
                manager.save_current(context);
                manager.block(pid);
                manager.switch_next(context);
            }
            _ => unreachable!(),
        }
    })
}

pub fn sem_signal(key: u32, context: &mut ProcessContext) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        let manager = get_process_manager();
        let ret = manager.current().write().sem_signal(key);
        match ret{
            SemaphoreResult::WakeUp(pid) => manager.wakeup(pid),
            SemaphoreResult::Ok => context.set_rax(114),
            SemaphoreResult::NotExist => context.set_rax(514), // any
            _ => unreachable!(),
        }
    })
}

pub fn new_sem(key: u32, value: usize) -> usize {
    x86_64::instructions::interrupts::without_interrupts(|| {
        if get_process_manager().current().write().new_sem(key, value) {
            0
        } else {
            1
        }
    })
}

pub fn remove_sem(key: u32) -> usize {
    x86_64::instructions::interrupts::without_interrupts(|| {
        if get_process_manager().current().write().remove_sem(key) {
            0
        } else {
            1
        }
    })
}