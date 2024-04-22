use super::*;
use crate::memory::*;
use alloc::sync::Weak;
use alloc::vec::Vec;
use boot::current_page_table;
use spin::*;
use x86_64::structures::paging::mapper::MapToError;
use x86_64::structures::paging::page::PageRange;
use x86_64::structures::paging::*;
use alloc::sync::Arc;
use crate::proc::sync::SemaphoreResult;


#[derive(Clone)]
pub struct Process {
    pid: ProcessId,
    inner: Arc<RwLock<ProcessInner>>,
}

pub struct ProcessInner {
    name: String,
    parent: Option<Weak<Process>>,
    children: Vec<Arc<Process>>,
    ticks_passed: usize,
    status: ProgramStatus,
    exit_code: Option<isize>,
    context: ProcessContext,
    page_table: Option<PageTableContext>,
    proc_data: Option<ProcessData>,
}

impl Process {
    #[inline]
    pub fn pid(&self) -> ProcessId {
        self.pid
    }

    #[inline]
    pub fn write(&self) -> RwLockWriteGuard<ProcessInner> {
        self.inner.write()
    }

    #[inline]
    pub fn read(&self) -> RwLockReadGuard<ProcessInner> {
        self.inner.read()
    }

    pub fn new(
        name: String,
        parent: Option<Weak<Process>>,
        page_table: PageTableContext,
        proc_data: Option<ProcessData>,
    ) -> Arc<Self> {
        let name = name.to_ascii_lowercase();

        // create context
        let pid = ProcessId::new();

        let inner = ProcessInner {
            name,
            parent,
            status: ProgramStatus::Ready,
            context: ProcessContext::default(),
            ticks_passed: 0,
            exit_code: Some(0),
            children: Vec::new(),
            page_table: Some(page_table),
            proc_data: Some(proc_data.unwrap_or_default()),
        };

        trace!("New process {}#{} created.", &inner.name, pid);

        // create process struct
        Arc::new(Self {
            pid,
            inner: Arc::new(RwLock::new(inner)),
        })
    }

    pub fn kill(&self, ret: isize) {
        let mut inner = self.inner.write();

        debug!(
            "Killing process {}#{} with ret code: {}",
            inner.name(),
            self.pid,
            ret
        );

        inner.kill(ret);
    }

    // pub fn alloc_init_stack(&self) -> VirtAddr {
    //     // FIXME: alloc init stack base on self pid
    //     let pid = self.pid().0 as u64;
    //     let addr = STACK_INIT_BOT - (pid-1)*STACK_MAX_SIZE;
    //     let count = STACK_DEF_PAGE;
    //     let mut page_table = self.read().page_table.as_ref().unwrap().mapper();
    //     let frame_allocator = &mut *get_frame_alloc_for_sure();
    //     println!("pid = {}, addr = {}, count = {}",pid,addr,count);
    //     let _ = elf::map_range(
    //         addr,
    //         count,
    //         &mut page_table,
    //         frame_allocator,
    //         true,
    //     );
        
    //     self.write().set_stack(VirtAddr::new(addr), count);

    //     VirtAddr::new(STACK_INIT_TOP - (pid-1)*STACK_MAX_SIZE) // pid从2开始算
    // }

    pub fn get_data_mut(&self) -> RwLockWriteGuard<ProcessInner> {
        self.inner.write()
    }

    // lab5
    pub fn fork(self: &Arc<Self>) -> Arc<Self> {
        // FIXME: lock inner as write
        let mut inner = self.write();
        // FIXME: inner fork with parent weak ref
        info!("forking from parent pid:{}",self.pid);
        let child_inner:ProcessInner = inner.fork(Arc::downgrade(self));
        // FOR DBG: maybe print the child process info
        //          e.g. parent, name, pid, etc.

        // FIXME: make the arc of child
        let child = Arc::new(Self {
            pid: ProcessId::new(),
            inner: Arc::new(RwLock::new(child_inner)),
        });
        // FIXME: add child to current process's children list
        inner.children.push(Arc::clone(&child));
        // FIXME: set fork ret value for parent with `context.set_rax`
        inner.context.set_rax(child.pid.0 as usize);
        
        // FIXME: mark the child as ready & return it
        child.write().status = ProgramStatus::Ready;
        child
    }

}

impl ProcessInner {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn tick(&mut self) {
        self.ticks_passed += 1;
    }

    pub fn status(&self) -> ProgramStatus {
        self.status
    }

    pub fn pause(&mut self) {
        self.status = ProgramStatus::Ready;
    }

    pub fn resume(&mut self) {
        self.status = ProgramStatus::Running;
    }

    pub fn exit_code(&self) -> Option<isize> {
        self.exit_code
    }

    pub fn clone_page_table(&self) -> PageTableContext {
        self.page_table.as_ref().unwrap().clone_l4()
    }

    pub fn is_ready(&self) -> bool {
        self.status == ProgramStatus::Ready
    }

    /// Save the process's context
    /// mark the process as ready
    pub(super) fn save(&mut self, context: &ProcessContext) {
        // FIXME: save the process's context
        self.context.save(context);
        if self.status == ProgramStatus::Running {
            self.status = ProgramStatus::Ready;
        }
    }

    pub fn block(&mut self){
        self.status = ProgramStatus::Blocked;
    }

    /// Restore the process's context
    /// mark the process as running
    pub(super) fn restore(&mut self, context: &mut ProcessContext) {
        // FIXME: restore the process's context
        self.context.restore(context);
        // FIXME: restore the process's page table
        self.page_table.as_ref().unwrap().load();
        self.status = ProgramStatus::Running;
    }

    pub fn parent(&self) -> Option<Arc<Process>> {
        self.parent.as_ref().and_then(|p| p.upgrade())
    }

    pub fn kill(&mut self, ret: isize) {
        // FIXME: set exit code
        self.exit_code = Some(ret);
        // FIXME: set status to dead
        self.status = ProgramStatus::Dead;
        // FIXME: take and drop unused resources
        self.proc_data.take();
        self.page_table.take();
    }

    // 辅助函数，获取ProcessContext
    pub fn get_process_context(&mut self) -> &mut ProcessContext{
        &mut self.context
    }

    pub fn init_stack_frame(&mut self, entry: VirtAddr, stack_top: VirtAddr) {
        self.context.init_stack_frame(entry, stack_top);
    }

    pub fn init_user_stack_frame(&mut self, entry: VirtAddr, stack_top: VirtAddr) {
        self.context.init_user_stack_frame(entry, stack_top);
    }


    pub fn alloc_new_stack_page(&mut self,addr: VirtAddr){
        let alloc = &mut *get_frame_alloc_for_sure();
        let new_start_page = Page::<Size4KiB>::containing_address(addr);
        let old_stack = self.proc_data.as_ref().unwrap().stack_segment.unwrap();

        let pages = old_stack.start - new_start_page;
        let page_table = &mut self.page_table.as_mut().unwrap().mapper();

        let user_access = processor::current().get_pid().unwrap() != KERNEL_PID;

        let result = elf::map_range(addr.as_u64(), pages, page_table, alloc,user_access);
        // if result.is_err(){
        //     error!("map_range failed");
        // }
    }
    
    pub fn load_elf(&mut self, elf: &ElfFile, pid: u64) -> u64{
        let mut page_table = self.page_table.as_ref().unwrap().mapper();
        let mut frame_allocator = &mut *get_frame_alloc_for_sure();

        let code_segments = elf::load_elf(elf, *PHYSICAL_OFFSET.get().unwrap(), &mut page_table, frame_allocator,true);

        let stack_bot:u64= STACK_INIT_BOT -(pid - 1)* STACK_MAX_SIZE;
        let stack_segment = elf::map_range(stack_bot, STACK_DEF_PAGE, &mut page_table, frame_allocator, true).unwrap();

        let proc_data = self.proc_data.as_mut().unwrap();
        proc_data.stack_segment = Some(stack_segment);
        proc_data.code_segments = Some(code_segments);

        stack_bot
    }

    pub fn proc_data_read(&mut self, fd:u8, buf: &mut [u8]) -> isize{
        self.proc_data.as_ref().expect("invalid proc_data").read(fd,buf)
    }

    // lab5
    pub fn fork(&mut self, parent: Weak<Process>) -> ProcessInner {
        // FIXME: get current process's stack info
        let stack_info = self.stack_segment.unwrap();
        let old_stack_base = stack_info.start.start_address().as_u64();
        let mut new_stack_base = old_stack_base - (self.children.len() as u64 + 1) * STACK_MAX_SIZE;

        // FIXME: clone the process data struct
        let mut child_proc_data = self.proc_data.as_ref().unwrap().clone();

        // FIXME: clone the page table context (see instructions)
        let page_table = self.page_table.as_ref().unwrap().clone_l4();

        // FIXME: alloc & map new stack for child (see instructions)
        while elf::map_range(
            new_stack_base,
            stack_info.count() as u64,
            &mut page_table.mapper(),
            &mut *get_frame_alloc_for_sure(),
            true,
        ).is_err() {
            debug!("Map thread stack to {:#X} failed.", new_stack_base);
            new_stack_base -= STACK_MAX_SIZE; // stack grow down
        }

        // FIXME: copy the *entire stack* from parent to child
        debug!("old_stack_base:{:X}, new_stack_base:{:X}",old_stack_base,new_stack_base);
        elf::clone_range(old_stack_base, new_stack_base, stack_info.count());

        // FIXME: update child's context with new *stack pointer*
        let mut child_context = self.context;
        //          > update child's stack to new base
        child_context.value.stack_frame.stack_pointer += new_stack_base - old_stack_base;
        //          > keep lower bits of *rsp*, update the higher bits  哪能改rsp???
        //          > also update the stack record in process data       
        child_proc_data.stack_memory_usage = stack_info.count();
        child_proc_data.stack_segment = Some(Page::range(
            Page::containing_address(VirtAddr::new_truncate(new_stack_base)),
            Page::containing_address(VirtAddr::new_truncate(
                new_stack_base + stack_info.count() as u64 * Size4KiB::SIZE,
            ))));
        
        // FIXME: set the return value 0 for child with `context.set_rax`
        child_context.set_rax(0);

        // FIXME: construct the child process inner
        let child_page_table = self.page_table.as_ref().unwrap().fork();

        ProcessInner {
            name: self.name.clone(),
            ticks_passed: 0,
            proc_data: Some(child_proc_data),
            page_table: Some(child_page_table),
            context: child_context,
            parent: Some(parent),
            children: Vec::new(),
            status: ProgramStatus::Ready,
            exit_code: Some(0),
        }
        // NOTE: return inner because there's no pid record in inner
    }

    pub fn sem_wait(&self, key: u32, pid: ProcessId) -> SemaphoreResult{
        if let Some(proc_data) = &self.proc_data {
            proc_data.semaphores.write().wait(key, pid)
        } else {
            SemaphoreResult::NotExist
        }
    }

    pub fn sem_signal(&self, key: u32) -> SemaphoreResult{
        if let Some(proc_data) = &self.proc_data {
            proc_data.semaphores.write().signal(key)
        } else {
            SemaphoreResult::NotExist
        }
    }

    pub fn new_sem(&self, key: u32, value: usize) -> bool {
        if let Some(proc_data) = &self.proc_data {
            proc_data.semaphores.write().insert(key, value)
        } else {
            false
        }
    }

    pub fn remove_sem(&self, key: u32) -> bool {
        if let Some(proc_data) = &self.proc_data {
            proc_data.semaphores.write().remove(key)
        } else {
            false
        }
    }
}


impl core::ops::Deref for Process {
    type Target = Arc<RwLock<ProcessInner>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl core::ops::Deref for ProcessInner {
    type Target = ProcessData;

    fn deref(&self) -> &Self::Target {
        self.proc_data
            .as_ref()
            .expect("Process data empty. The process may be killed.")
    }
}

impl core::ops::DerefMut for ProcessInner {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.proc_data
            .as_mut()
            .expect("Process data empty. The process may be killed.")
    }
}

impl core::fmt::Debug for Process {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let mut f = f.debug_struct("Process");
        f.field("pid", &self.pid);

        let inner = self.inner.read();
        f.field("name", &inner.name);
        f.field("parent", &inner.parent().map(|p| p.pid));
        f.field("status", &inner.status);
        f.field("ticks_passed", &inner.ticks_passed);
        f.field(
            "children",
            &inner.children.iter().map(|c| c.pid.0).collect::<Vec<u16>>(),
        );
        f.field("page_table", &inner.page_table);
        f.field("status", &inner.status);
        f.field("context", &inner.context);
        f.field("stack", &inner.proc_data.as_ref().map(|d| d.stack_segment));
        f.finish()
    }
}

impl core::fmt::Display for Process {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let inner = self.inner.read();
        write!(
            f,
            " #{:-3} | #{:-3} | {:14} | {:7} | {:12} | {:?}",
            self.pid.0,
            inner.parent().map(|p| p.pid.0).unwrap_or(0),
            inner.name,
            inner.ticks_passed,
            inner.total_memory_usage(),
            inner.status
        )?;
        Ok(())
    }
}
