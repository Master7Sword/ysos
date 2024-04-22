use core::alloc::Layout;

use crate::proc::*;
use crate::utils::*;


use self::processor::current;

use super::SyscallArgs;

pub fn spawn_process(args: &SyscallArgs) -> usize {
    // FIXME: get app name by args
    //       - core::str::from_utf8_unchecked  将字节切片转换为字符串切片
    //       - core::slice::from_raw_parts     接受裸指针和长度，返回切片
    let name = unsafe{
        core::str::from_utf8_unchecked(core::slice::from_raw_parts(args.arg0 as *const u8, args.arg1))
    };
    // FIXME: spawn the process by name
    let pid = crate::proc::spawn(name);
    // FIXME: handle spawn error, return 0 if failed
    if !pid.is_some(){
        warn!("spawn error!");
        return 0;
    }
    // FIXME: return pid as usize
    
    u16::from(pid.unwrap()) as usize
}

pub fn sys_write(args: &SyscallArgs) -> usize {
    // FIXME: get buffer and fd by args
    //       - core::slice::from_raw_parts
    let buffer = unsafe{
        core::slice::from_raw_parts(args.arg1 as *const u8, args.arg2)
    };
    let fd = args.arg0 as u8;
    // FIXME: call proc::write -> isize
    let pid = current().get_pid().unwrap();
    let proc = get_process_manager().get_proc(&pid).unwrap();
    let proc_data = proc.get_data_mut();
    let result = proc_data.write(fd, buffer);
    // FIXME: return the result as usize
    
    result as usize
}

pub fn sys_read(args: &SyscallArgs) -> usize {
    let mut buffer = unsafe{
        core::slice::from_raw_parts_mut(args.arg1 as *mut u8, args.arg2)
    };
    let fd = args.arg0 as u8;

    let pid = current().get_pid().unwrap();
    let proc = get_process_manager().get_proc(&pid).unwrap();
    let mut proc_inner = proc.get_data_mut();
    let result = proc_inner.proc_data_read(fd, buffer);
    result as usize
}

pub fn exit_process(args: &SyscallArgs, context: &mut ProcessContext) {
    // FIXME: exit process with retcode
    exit(args.arg0 as isize, context);
}

pub fn list_process() {
    // FIXME: list all processes
    get_process_manager().print_process_list();
}

pub fn sys_allocate(args: &SyscallArgs) -> usize {
    let layout = unsafe { (args.arg0 as *const Layout).as_ref().unwrap() };

    if layout.size() == 0 {
        return 0;
    }

    let ret = crate::memory::user::USER_ALLOCATOR
        .lock()
        .allocate_first_fit(*layout);

    match ret {
        Ok(ptr) => ptr.as_ptr() as usize,
        Err(_) => 0,
    }
}

pub fn sys_deallocate(args: &SyscallArgs) {
    let layout = unsafe { (args.arg1 as *const Layout).as_ref().unwrap() };

    if args.arg0 == 0 || layout.size() == 0 {
        return;
    }

    let ptr = args.arg0 as *mut u8;

    unsafe {
        crate::memory::user::USER_ALLOCATOR
            .lock()
            .deallocate(core::ptr::NonNull::new_unchecked(ptr), *layout);
    }
}

pub fn sys_wait_pid(args: &SyscallArgs) -> isize{
    let pid = args.arg0;
    let target_process = get_process_manager().get_proc(&ProcessId((pid) as u16)).unwrap();
    let exit_code = target_process.read().exit_code().unwrap();
    //info!("{}",exit_code);
    exit_code
}

// lab4 加分项  

pub fn sys_clock() -> i64 {
    if let Some(t) = clock::now() {
        return t.and_utc().timestamp_nanos_opt().unwrap_or_default();
    } else {
        return -1;
    }
}

// lab5

pub fn sys_sem(args: &SyscallArgs, context: &mut ProcessContext) {
    match args.arg0 {
        0 => context.set_rax(new_sem(args.arg1 as u32, args.arg2)),
        1 => context.set_rax(remove_sem(args.arg1 as u32)),
        2 => sem_signal(args.arg1 as u32, context),
        3 => sem_wait(args.arg1 as u32, context),
        _ => context.set_rax(usize::MAX),
    }
}

