# Valhalla OS Documentation

This document provides a detailed technical overview of Valhalla OS, a 64-bit micro-kernel written in Rust for the x86_64 architecture. It serves as a reference for understanding the core operating system concepts regarding how they are implemented in this codebase.

## 1. Core Architecture

The OS follows a monolithic kernel design where major services (memory management, interrupt handling, drivers) run in kernel space. It uses the `bootloader` crate to boot into long mode (64-bit) and maps the kernel to the higher half of memory.

### Entry Point (`src/main.rs`)

The entry point is defined using the `entry_point!` macro from the `bootloader` crate.
- **Function**: `kernel_main(boot_info: &'static BootInfo) -> !`
- **Role**: Initializes the kernel, sets up memory, and starts the main loop.
- **Loop**: The kernel enters an infinite loop, executing `x86_64::instructions::hlt()` to halt the CPU until the next interrupt, saving power.

## 2. VGA Text Mode (`src/vga_buffer.rs`)

The VGA text buffer is the primary output mechanism, mapping characters to the screen via Memory-Mapped I/O at address `0xb8000`.

### Implementation
- **Volatile Wrappers**: The `volatile` crate is used to wrap memory writes (`Volatile<ScreenChar>`) to prevent the Rust compiler from optimizing away writes to the VGA buffer.
- **Global Writer**: A global `WRITER` instance is protected by a `Spinlock` (via `spin::Mutex`) and initialized strictly once using `lazy_static!`.
- **Macros**: Custom `print!` and `println!` macros are exported to provide a standard output interface similar to Rust's std library.

```rust
// Key structures
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

// 0xb8000 is the standard VGA text buffer address
const VGA_BUFFER_ADDR: usize = 0xb8000;
```

## 3. Global Descriptor Table (GDT) & Stack Protection (`src/gdt.rs`)

The GDT is used primarily for switching between kernel/user space and, critically in this OS, for **Stack Overflow Protection**.

### Implementation
- **TSS (Task State Segment)**: A TSS is defined to hold the **Interrupt Stack Table (IST)**.
- **Double Fault Handler**: A specific stack is allocated for double faults. The index of this stack (`DOUBLE_FAULT_IST_INDEX`) is registered in the IDT. If a stack overflow occurs (causing a double fault), the CPU switches to this known good stack, allowing the kernel to catch the panic instead of causing a triple fault (reboot).

```rust
// The Double Fault stack is part of the implementation for robustness
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
```

## 4. Interrupt Handling (`src/interrupts.rs`)

The OS handles both hardware interrupts and CPU exceptions using an **Interrupt Descriptor Table (IDT)**.

### Implementation
- **IDT**: Statically defined using `lazy_static`.
- **Exceptions Handled**:
    - `Breakpoint`: For debugging.
    - `Page Fault`: Prints the accessed address (`CR2` register) and error code.
    - `Double Fault`: Diverts to the dedicated stack defined in the GDT.
- **Hardware Interrupts**:
    - **PIC (8259)**: Two chained PICs are used to map hardware interrupts to CPU interrupt vectors starting at offset 32 (to avoid conflicts with CPU exceptions 0-31).
    - **Timer**: Fires periodically; handler notifies the PIC `notify_end_of_interrupt`.
    - **Keyboard**: Reads scancodes from port `0x60`, decodes them using `pc-keyboard`, and pushes valid characters to a global `COMMAND_BUFFER`.

## 5. Memory Management (`src/memory.rs`)

Memory management handles the translation between virtual and physical memory (Paging) and provides a frame allocator.

### Implementation
- **Paging**: The OS uses an `OffsetPageTable`. This technique maps the entire physical memory to a specific range in virtual memory (provided by the bootloader), allowing the kernel to access any physical address by adding an offset.
- **Frame Allocator** (`BootInfoFrameAllocator`):
    - Parses the memory map provided by the BIOS/UEFI.
    - Returns physical execution frames (`PhysFrame`) for memory allocation.
    - **Algorithm**: A simple linear iterator that returns the next available 'Usable' 4KiB frame.

```rust
// Initialization of the page table
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> { ... }
```

## 6. Heap Allocation (`src/allocator.rs`)

To support dynamic types like `Box`, `Vec`, and `String`, a heap allocator is implemented.

### Implementation
- **Allocator**: Uses `linked_list_allocator::LockedHeap`.
- **Heap Region**:
    - **Start Address**: `0x_4444_4444_0000`
    - **Size**: 100 KiB
- **Mapping**: The `init_heap` function maps the virtual pages of the heap region to physical frames allocated by the frame allocator and initializes the `LockedHeap`.

```rust
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();
```

## 7. Shell & User Input (`src/shell.rs`)

A basic command-line interface allows interaction with the running kernel.

### Implementation
- **Command Loop**: In `main.rs`, the kernel checks `interrupts::COMMAND_READY`.
- **Input Handling**: The keyboard interrupt handler populates `COMMAND_BUFFER`. When 'Enter' is pressed, it sets the ready flag.
- **Command Processor**: `shell::interpret` parses the string and executes:
    - `help`: Lists commands.
    - `ping`: Responds with "Pong!".
    - `clear`: Scrolls the screen (prints newlines).
    - `heap_test`: Allocates a `Box` to verify heap functionality.

## Summary of Control Flow

1. **Boot**: `_start` -> `kernel_main`.
2. **Init**:
   - `gdt::init()` sets up stack switching.
   - `interrupts::init_idt()` prepares exception handling.
   - `PICS` initialized for hardware interrupts.
   - Memory Paging & Frame Allocator initialized.
   - Heap mapped and active.
3. **Run**:
   - Splash screen prints.
   - `loop`: Checks for user input -> Executes Shell command -> `hlt()` (Sleep).
