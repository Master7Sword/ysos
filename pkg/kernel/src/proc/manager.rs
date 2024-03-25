use self::processor::set_pid;

use super::*;
use crate::memory::{
    self,
    allocator::{ALLOCATOR, HEAP_SIZE},
    get_frame_alloc_for_sure, PAGE_SIZE,
};
use alloc::{collections::*, format};
use spin::{Mutex, RwLock};
use alloc::sync::Arc;

pub static PROCESS_MANAGER: spin::Once<ProcessManager> = spin::Once::new();

pub fn init(init: Arc<Process>) {

    // FIXME: set init process as Running
    let mut inner = init.write();
    inner.resume();
    drop(inner); // 释放写锁
    // FIXME: set processor's current pid to init's pid
    PROCESS_MANAGER.call_once(|| ProcessManager::new(init.clone()));
    let manager = PROCESS_MANAGER.get().expect("Process Manager should be initialized!");
    manager.push_ready(init.pid());

    PROCESS_MANAGER.call_once(|| ProcessManager::new(init));
}

pub fn get_process_manager() -> &'static ProcessManager {
    PROCESS_MANAGER
        .get()
        .expect("Process Manager has not been initialized")
}

pub struct ProcessManager {
    processes: RwLock<BTreeMap<ProcessId, Arc<Process>>>,
    ready_queue: Mutex<VecDeque<ProcessId>>,
}

// lab3有个莫名其妙的处理函数尚未实现，等到wait_pid要用的时候再写
impl ProcessManager {
    pub fn new(init: Arc<Process>) -> Self {
        let mut processes = BTreeMap::new();
        let ready_queue = VecDeque::new();
        let pid = init.pid();

        trace!("Init {:#?}", init);

        processes.insert(pid, init);
        Self {
            processes: RwLock::new(processes),
            ready_queue: Mutex::new(ready_queue),
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
            let mut queue = self.ready_queue.lock();
            queue.push_back(process.pid());
        }

        drop(inner); // 释放
    }

    pub fn switch_next(&self, context: &mut ProcessContext) -> ProcessId {

        // FIXME: fetch the next process from ready queue
        let processes = self.processes.read();
        let mut process :Option<&Arc<Process>> = None;
        let mut process_arc:  &Arc<Process>;
        let mut read_inner: spin::RwLockReadGuard<'_, ProcessInner>;
        let mut queue = self .ready_queue.lock();
        let mut next_pid: Option<ProcessId> = None;
        let mut inner: spin::rwlock::RwLockWriteGuard<'_, ProcessInner>;
        

        // FIXME: check if the next process is ready,
        //        continue to fetch if not ready
        while true{
            if let Some(pid) = queue.pop_front() {
                next_pid = Some(pid);
                trace!("get next_pid {}",pid);
            } else{
                panic!("the PID queue is empty!");
            }
            process= processes.get(&next_pid.expect("invalid next_pid"));
            process_arc= process.expect("invalid process_arc");
            read_inner = process_arc.read();
            if read_inner.status() == ProgramStatus::Ready{
                break;
            }
        }

        // FIXME: restore next process's context
        process_arc = process.expect("invalid process_arc");
        inner = process_arc.write();
        inner.save(context);

        // FIXME: update processor's current pid
        set_pid(next_pid.expect("invalid next_pid"));

        // FIXME: return next process's pid
        drop(inner);
        next_pid.expect("Expected a next PID but got None")
    }

    // 创建内核进程
    pub fn spawn_kernel_thread(
        &self,
        entry: VirtAddr,
        name: String,
        proc_data: Option<ProcessData>,
    ) -> ProcessId {
        let kproc = self.get_proc(&KERNEL_PID).unwrap();
        let page_table = kproc.read().clone_page_table();
        let proc = Process::new(name, Some(Arc::downgrade(&kproc)), page_table, proc_data);

        // alloc stack for the new process base on pid
        let stack_top = proc.alloc_init_stack();

        // FIXME: set the stack frame
        ProcessContext::init_stack_frame(proc.write().get_process_context(), entry, stack_top);

        // FIXME: add to process map
        let new_pid = proc.pid();
        self.add_proc(new_pid, proc.clone());

        // FIXME: push to ready queue
        self.push_ready(new_pid);

        // FIXME: return new process pid
        new_pid
    }

    pub fn kill_current(&self, ret: isize) {
        self.kill(processor::get_pid(), ret);
    }

    pub fn handle_page_fault(&self, addr: VirtAddr, err_code: PageFaultErrorCode) -> bool {
        // FIXME: handle page fault

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

    
}
