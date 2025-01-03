#![no_std]
#![no_main]
#![feature(alloc_error_handler)]

#[macro_use]
extern crate log;
extern crate alloc;

use alloc::boxed::Box;
use alloc::vec;
use elf::{load_elf, map_physical_memory};
use uefi::prelude::*;
use x86_64::{addr, registers::control::*, structures::paging::frame};
use ysos_boot::*;

//
use crate::config::Config;
////
mod config;

const CONFIG_PATH: &str = "\\EFI\\BOOT\\boot.conf";

#[entry]
fn efi_main(image: uefi::Handle, mut system_table: SystemTable<Boot>) -> Status {
    uefi_services::init(&mut system_table).expect("Failed to initialize utilities");

    log::set_max_level(log::LevelFilter::Info);
    info!("Running UEFI bootloader...");

    let bs = system_table.boot_services();

    // 1. Load config
    let mut config_file = open_file(bs, CONFIG_PATH);
    let config_content = load_file(bs,&mut config_file);
    let mut config = Config::parse(&config_content);

    info!("Config: {:#x?}", config);

    // 2. Load ELF files
    let mut elf_file = open_file(bs,&mut config.kernel_path);
    let elf_content = load_file(bs,&mut elf_file);
    let elf = match xmas_elf::ElfFile::new(elf_content){
        Ok(file) => file,
        Err(msg) => {
            panic!("Failed to parse ELF file: {}",msg);
        }
    };
    //panic!("Failed to parse ELF file: {}","msg");

    unsafe {
        set_entry(elf.header.pt2.entry_point() as usize);
    }

    // lab4 新增
    let apps = if config.load_apps {
        info!("Loading apps...");
        Some(load_apps(system_table.boot_services()))
    } else {
        info!("Skip loading apps");
        None
    };
    

    // 3. Load MemoryMap
    let max_mmap_size = system_table.boot_services().memory_map_size();
    let mmap_storage = Box::leak(
        vec![0; max_mmap_size.map_size + 10 * max_mmap_size.entry_size].into_boxed_slice(),
    );
    let mmap = system_table
        .boot_services()
        .memory_map(mmap_storage)
        .expect("Failed to get memory map");

    let max_phys_addr = mmap
        .entries()
        .map(|m| m.phys_start + m.page_count * 0x1000)
        .max()
        .unwrap()
        .max(0x1_0000_0000); // include IOAPIC MMIO area

    // 4. Map ELF segments, kernel stack and physical memory to virtual memory
    let mut page_table = current_page_table();

    // FIXME: root page table is readonly, disable write protect (Cr0)
    unsafe{
        Cr0::update(|f| f.remove(Cr0Flags::WRITE_PROTECT))
    }

    let mut frame_allocator = UEFIFrameAllocator(bs);
    // FIXME: map physical memory to specific virtual address offset
    elf::map_physical_memory(config.physical_memory_offset, max_phys_addr,&mut page_table,&mut frame_allocator);

    // FIXME: load and map the kernel elf file
    elf::load_elf(&elf, config.physical_memory_offset,&mut page_table,&mut frame_allocator,false);

    // FIXME: map kernel stack
    elf::map_range(
        config.kernel_stack_address,
        config.kernel_stack_size,
        &mut page_table,
        &mut frame_allocator,
        false,
    );

    // FIXME: recover write protect (Cr0)
    unsafe{
        Cr0::update(|f| f.insert(Cr0Flags::WRITE_PROTECT))
    }

    free_elf(bs, elf);

    // 5. Exit boot and jump to ELF entry
    info!("Exiting boot services...");

    let (runtime, mmap) = system_table.exit_boot_services(MemoryType::LOADER_DATA);
    // NOTE: alloc & log are no longer available


    // construct BootInfo
    let bootinfo = BootInfo {
        memory_map: mmap.entries().copied().collect(),
        physical_memory_offset: config.physical_memory_offset,
        system_table: runtime,
        loaded_apps: apps,
    };

    // align stack to 8 bytes
    let stacktop = config.kernel_stack_address + config.kernel_stack_size * 0x1000 - 8;

    unsafe {
        jump_to_entry(&bootinfo, stacktop);
    }
}
