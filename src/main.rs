#![no_std]
#![no_main]

extern crate alloc;

use bootloader::{BootInfo, entry_point};
use core::panic::PanicInfo;
use rust_os::{print, println}; // <--- Import print here

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // Note: 'shell' is now available because we added it to lib.rs
    use rust_os::{allocator, interrupts, memory, shell};
    use x86_64::VirtAddr;

    rust_os::init();

    // Memory initialization
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator =
        unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    // Splash screen
    rust_os::vga_buffer::print_logo();
    println!("Welcome to Valhalla. Memory management online.");
    print!("> "); // This macro now works!

    // The Shell Loop
    loop {
        if *interrupts::COMMAND_READY.lock() {
            let mut buffer = interrupts::COMMAND_BUFFER.lock();
            let mut ready = interrupts::COMMAND_READY.lock();

            let command = buffer.clone();
            buffer.clear();
            *ready = false;

            shell::interpret(command);
        }
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
