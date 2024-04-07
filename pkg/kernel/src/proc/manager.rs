use self::processor::{set_pid, Processor};
use arrayvec::ArrayVec;
use elf::load_elf;

use super::*;
use crate::memory::{
    self,
    allocator::{ALLOCATOR, HEAP_SIZE},
    get_frame_alloc_for_sure, PAGE_SIZE,
};
use alloc::{collections::*, format};
use boot::AppListRef;
use spin::{Mutex, RwLock};
use alloc::sync::Arc;
use alloc::sync::Weak;

pub static PROCESS_MANAGER: spin::Once<ProcessManager> = spin::Once::new();

pub fn init(init: Arc<Process>, apps:AppListRef) {

    // FIXME: set init process as Running
    let mut inner = init.write();
    inner.resume();
    // 思考题：这里把进程管理器初始化后的状态设置为Ready，看看有什么问题
    //inner.pause();
    drop(inner); // 释放写锁
    // FIXME: set processor's current pid to init's pid
    set_pid(init.pid());


    PROCESS_MANAGER.call_once(|| ProcessManager::new(init,apps));
}

pub fn get_process_manager() -> &'static ProcessManager {
    PROCESS_MANAGER
        .get()
        .expect("Process Manager has not been initialized")
}

// lab4新增，辅助app_list初始化
// lazy_static! {
//     static ref GLOBAL_APP_LIST: boot::AppList = ArrayVec::new();
// }

pub struct ProcessManager {
    processes: RwLock<BTreeMap<ProcessId, Arc<Process>>>,
    ready_queue: Mutex<VecDeque<ProcessId>>,
    app_list: boot::AppListRef,
}

// lab3有个莫名其妙的处理函数尚未实现，等到wait_pid要用的时候再写
impl ProcessManager {
    pub fn new(init: Arc<Process>, app_list: boot::AppListRef) -> Self {
        let mut processes = BTreeMap::new();
        let ready_queue = VecDeque::new();
        let pid = init.pid();

        trace!("Init {:#?}", init);

        processes.insert(pid, init);
        Self {
            processes: RwLock::new(processes),
            ready_queue: Mutex::new(ready_queue),
            app_list,
        }
    }

    #[inline]
    pub fn push_ready(&self, pid: ProcessId) {
        self.ready_queue.lock().push_back(pid);
    }

    #[inline]
    fn add_proc(&self, pid: ProcessId, proc: Arc<Process>) {
        self.processes.write().insert(pid, proc);
    }

    #[inline]
    pub fn get_proc(&self, pid: &ProcessId) -> Option<Arc<Process>> {
        self.processes.read().get(pid).cloned()
    }

    pub fn current(&self) -> Arc<Process> {
        self.get_proc(&processor::get_pid())
            .expect("No current process")
    }

    pub fn save_current(&self, context: &ProcessContext) {
        // FIXME: update current process's tick count
        let process = self.current();
        let mut inner = process.write();
        inner.tick();
        
        // FIXME: update current process's context
        inner.save(context);
        
        // FIXME: push current process to ready queue if still alive
        if inner.status() != ProgramStatus::Dead{
            self.push_ready(process.pid());
        }

        drop(inner); // 释放
    }

    pub fn switch_next(&self, context: &mut ProcessContext) -> ProcessId {

        let mut pid = self.current().pid();

        while let Some(next) = self.ready_queue.lock().pop_front() {
            let map = self.processes.read();
            let proc = map.get(&next).expect("Process not found");
            
            if !proc.read().is_ready() {
                continue;
            }
            
            if pid != next {
                //println!("Before switching, current status:{:?}, next status:{:?}, current pid:{:?}, next pid:{:?},",self.current().read().status(),proc.read().status(),self.current().pid(),proc.pid());
                proc.write().restore(context);
                processor::set_pid(next);
                pid = next;
                //println!("After switching, current status:{:?}, next status:{:?}, current pid:{:?}, next pid:{:?},",self.current().read().status(),proc.read().status(),self.current().pid(),proc.pid());

            }
            break;
        }
        pid
    }

    // lab3 创建内核进程，lab4中注释
    // pub fn spawn_kernel_thread(
    //     &self,
    //     entry: VirtAddr,
    //     name: String,
    //     proc_data: Option<ProcessData>,
    // ) -> ProcessId {
    //     let kproc = self.get_proc(&KERNEL_PID).unwrap();
    //     let page_table = kproc.read().clone_page_table();
    //     let proc = Process::new(name, Some(Arc::downgrade(&kproc)), page_table, proc_data);

    //     // alloc stack for the new process base on pid
    //     let stack_top = proc.alloc_init_stack();
    //     //info!("alloc_init_stack success!");

    //     // FIXME: set the stack frame
    //     let mut inner = proc.write();
    //     inner.pause();
    //     inner.init_stack_frame(entry,stack_top);
        
    //     //info!("init_stack_frame success!");

    //     // FIXME: add to process map
    //     let new_pid = proc.pid();
    //     info!("Spawn process: {}#{}", inner.name(), new_pid);
    //     drop(inner);
    //     self.add_proc(new_pid, proc.clone());

    //     // FIXME: push to ready queue
    //     self.push_ready(new_pid);
    //     //info!("push_ready success!");

    //     // FIXME: return new process pid
    //     new_pid
    // }

    pub fn kill_current(&self, ret: isize) {
        self.kill(processor::get_pid(), ret);
    }

    pub fn handle_page_fault(&self, addr: VirtAddr, err_code: PageFaultErrorCode) -> bool {
        // FIXME: handle page fault
        let process = self.current();
        let pid = process.pid().0 as u64;
        let min_addr = STACK_MAX - (pid)*STACK_MAX_SIZE;
        let max_addr = min_addr + STACK_MAX_SIZE;
        let addr_u64 = addr.as_u64();
        println!("addr:{:X}, min_addr:{:X}, max_addr:{:X}",addr_u64,min_addr,max_addr);
        if !err_code.contains(PageFaultErrorCode::PROTECTION_VIOLATION) && addr_u64 >= min_addr && addr_u64 <= max_addr{
            info!("handling...");
            process.write().alloc_new_stack_page(addr);
            return true
        }
        false
    }

    pub fn kill(&self, pid: ProcessId, ret: isize) {
        let proc: Option<Arc<Process>> = self.get_proc(&pid);

        if proc.is_none() {
            warn!("Process #{} not found.", pid);
            return;
        }

        let proc = proc.unwrap();

        if proc.read().status() == ProgramStatus::Dead {
            warn!("Process #{} is already dead.", pid);
            return;
        }

        trace!("Kill {:#?}", &proc);

        proc.kill(ret);
    }

    pub fn print_process_list(&self) {
        let mut output = String::from("  PID | PPID | Process Name |  Ticks  | Status\n");

        for (_, p) in self.processes.read().iter() {
            if p.read().status() != ProgramStatus::Dead {
                output += format!("{}\n", p).as_str();
            }
        }

        // TODO: print memory usage of kernel heap

        output += format!("Queue  : {:?}\n", self.ready_queue.lock()).as_str();

        output += &processor::print_processors();

        print!("{}", output);
    }

    pub fn app_list(&self) -> boot::AppListRef {
        self.app_list
    }

    // lab4 新增

    pub fn spawn(
        &self,
        elf: &ElfFile,
        name: String,
        parent: Option<Weak<Process>>,
        proc_data: Option<ProcessData>,
    ) -> ProcessId {
        let kproc = self.get_proc(&KERNEL_PID).unwrap();
        let page_table = kproc.read().clone_page_table();
        let proc = Process::new(name, parent, page_table, proc_data);
        let pid = proc.pid();
    
        let mut inner = proc.write();
        // FIXME: load elf to process pagetable
        let stack_bot = inner.load_elf(elf, pid.0 as u64);
        let stack_top = stack_bot + STACK_DEF_SIZE - 8;

        // FIXME: alloc new stack for process

        let entry_addr = elf.header.pt2.entry_point();
        inner.init_stack_frame(VirtAddr::new(entry_addr), VirtAddr::new(stack_top)); // 栈顶超出范围了
        
        // FIXME: mark process as ready
        inner.pause();

        drop(inner);
    
        trace!("New {:#?}", &proc);
    
        // FIXME: something like kernel thread
        self.add_proc(pid, proc);
        self.push_ready(pid);

        pid
    }
    
    pub fn kill_self(&self, ret: isize) {
        self.kill(processor::current().get_pid().unwrap(), ret);
    }
    
}
