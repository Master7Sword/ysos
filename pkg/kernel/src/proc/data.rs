use alloc::{collections::BTreeMap, sync::Arc};
use spin::RwLock;
use x86_64::structures::paging::{
    page::{PageRange, PageRangeInclusive},
    Page,
};
use crate::resource::ResourceSet;
use crate::proc::sync::SemaphoreSet;

use super::*;

#[derive(Debug, Clone)]
pub struct ProcessData {
    // shared data
    pub(super) env: Arc<RwLock<BTreeMap<String, String>>>,

    // process specific data
    pub(super) stack_segment: Option<PageRange>,

    pub(super) code_segments: Option<Vec<PageRangeInclusive>>,

    pub(super) stack_memory_usage: usize,

    pub(super) resources: Arc<RwLock<ResourceSet>>,

    pub(super) semaphores: Arc<RwLock<SemaphoreSet>>,
}

impl Default for ProcessData {
    fn default() -> Self {
        Self {
            env: Arc::new(RwLock::new(BTreeMap::new())),
            stack_segment: None,
            code_segments:None,
            stack_memory_usage: 0,
            resources: Arc::new(RwLock::new(ResourceSet::default())),
            semaphores: Arc::new(RwLock::new(SemaphoreSet::default())),
        }
    }
}

impl ProcessData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn env(&self, key: &str) -> Option<String> {
        self.env.read().get(key).cloned()
    }

    pub fn set_env(&mut self, key: &str, val: &str) {
        self.env.write().insert(key.into(), val.into());
    }

    pub fn set_stack(&mut self, start: VirtAddr, size: u64) {
        let start = Page::containing_address(start);
        self.stack_segment = Some(Page::range(start, start + size));
    }

    pub fn is_on_stack(&self, addr: VirtAddr) -> bool {
        // FIXME: check if the address is on the stack
        if let Some(ref range) = self.stack_segment {
            let start_addr = range.start.start_address().as_u64();
            let end_addr = range.end.start_address().as_u64(); 
            let addr_val = addr.as_u64();
            
            addr_val >= start_addr && addr_val < end_addr
        } else {
            false
        }
    }
    
    // lab4新增
    pub fn read(&self, fd: u8, buf: &mut [u8]) -> isize {
        self.resources.read().read(fd, buf)
    }
    
    pub fn write(&self, fd: u8, buf: &[u8]) -> isize {
        self.resources.read().write(fd, buf)
    }

    pub fn stack_memory_usage(&self) -> usize {
        if let Some(ref range) = self.stack_segment {
            let start_addr = range.start.start_address().as_u64();
            let end_addr = range.end.start_address().as_u64();
            (end_addr - start_addr) as usize
        } else {
            0
        }
    }

    pub fn code_memory_usage(&self) -> usize {
        if let Some(ref segments) = self.code_segments {
            segments.iter().map(|range| {
                let start_addr = range.start.start_address().as_u64();
                let end_addr = range.end.start_address().as_u64();
                (end_addr - start_addr) as usize
            }).sum()
        } else {
            0
        }
    }

    pub fn total_memory_usage(&self) -> usize {
        self.stack_memory_usage() + self.code_memory_usage()
    }
}
