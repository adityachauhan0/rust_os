# Valhalla OS Technical Documentation

This document provides a comprehensive deep-dive into the design, architecture, and implementation details of Valhalla OS. It describes the system from the bootloader up to the user shell, explaining the design choices and technical hurdles overcome during development.

## 1. System Architecture Overview

Valhalla OS is a 64-bit micro-kernel written in Rust. It runs on the x86_64 architecture and leverages specific hardware features like the Global Descriptor Table (GDT), Interrupt Descriptor Table (IDT), and Paging.

*   **Language**: Rust (Nightly, `no_std`)
*   **Bootloader**: `bootloader` crate (v0.9.x spec)
*   **Architecture**: Monolithic Kernel design (currently), aiming for micro-kernel characteristics.
*   **Build System**: Cargo with `bootimage` for disk creation.

---

## 2. Level 0: The Foundation

This layer handles the most basic interactions with the hardware, allowing the kernel to output information and survive basic crashes.

### 2.1 VGA Text Buffer Driver (`src/vga_buffer.rs`)

The kernel communicates with the user primarily through the VGA text buffer, a memory-mapped I/O region located at physical address `0xb8000`.

*   **Memory-Mapped I/O**: The buffer consists of 25 rows and 80 columns. Each screen character takes up 2 bytes: one for the ASCII character and one for the color attribute (foreground/background).
*   **Volatile Access**: To prevent the Rust compiler from optimizing away memory writes (since it doesn't know about side effects on hardware), the `volatile` crate is used to wrap the buffer.
*   **Concurrency**: A global static `WRITER` is protected by a `spin::Mutex`. Since the OS cannot use standard OS threads or mutexes yet, a spinlock is used to ensure that only one core (or interrupt handler) writes to the screen at a time.
*   **Macros**: `print!` and `println!` are implemented by hooking into the global writer, providing a seamless high-level interface.

### 2.2 Global Descriptor Table (GDT) & Stack Protection (`src/gdt.rs`)

While the GDT is required for 64-bit mode, Valhalla OS utilizes it specifically for **Stack Overflow Protection**.

*   **The Double Fault Problem**: If the kernel overflows the stack, it triggers a page fault. If the CPU cannot push the page fault frame onto the full stack, it triggers a Double Fault. If the Double Fault handler cannot run (because the stack is still full), the CPU Triple Faults and reboots.
*   **The Solution (IST)**: We set up an **Interrupt Stack Table (IST)** in the Task State Segment (TSS).
*   **Implementation**:
    1.  A dedicated stack is allocated in `gdt.rs`.
    2.  This stack is registered in the IST at index `DOUBLE_FAULT_IST_INDEX`.
    3.  The IDT entry for Double Faults is configured to physically switch the CPU to this fresh stack when the exception occurs, allowing the kernel to catch the panic gracefully.

---

## 3. Level 1: Interrupts & Input

This layer allows the OS to respond to asynchronous hardware events and CPU exceptions.

### 3.1 Interrupt Descriptor Table (IDT) (`src/interrupts.rs`)

The IDT maps exception vectors to function handlers.

*   **Exceptions**:
    *   **Breakpoint**: Used for unit testing (pauses execution).
    *   **Page Fault**: Critical for debugging memory issues. accessible via `CR2` register.
    *   **Double Fault**: As described above, handled on a separate stack.
*   **Calling Convention**: Handlers use the `x86-interrupt` calling convention, which saves all registers (not just caller-saved ones) to prevent corrupting the state of the interrupted process.

### 3.2 Hardware Interrupts & PIC

To handle external devices like the Timer and Keyboard, we us the 8259 Programmable Interrupt Controller (PIC).

*   **Remapping**: By default, the PIC maps interrupts to ranges 0-15, contradicting CPU exceptions. We remap the generic interrupts to offsets **32-47**.
    *   `PIC_1_OFFSET`: 32
*   **Timer**: defaults to ~18.2Hz (or configured otherwise). The handler sends an "End of Interrupt" (EOI) signal to the PIC to confirm processing.

### 3.3 PS/2 Keyboard Driver

Keyboard input is processed via Port I/O `0x60`.

*   **Scancodes**: The keyboard sends raw bytes (scancodes). We use `pc-keyboard` to decode Scancode Set 1.
*   **Processing**:
    1.  Interrupt fires -> `keyboard_interrupt_handler` reads byte.
    2.  `layouts::Us104Key` maps scancode to Char.
    3.  Character is pushed to `COMMAND_BUFFER`.
    4.  If `\n` (Enter) is pressed, `COMMAND_READY` flag is set to notify the shell.

---

## 4. Level 2: Memory Management

This is the most complex part of the kernel ("The Crown Jewel"), transforming raw physical RAM into a safe, dynamic environment.

### 4.1 Paging (`src/memory.rs`)

*   **4-Level Paging**: The OS uses the standard x86_64 4-level page table structure.
*   **Offset Page Table**: The bootloader maps the *entire* physical memory to a virtual address range.
    *   If physical memory is at `0x0`, and offset is `0xFFFF_8000_0000_0000`, the kernel can access physical `0x1000` at virtual `0xFFFF_8000_0000_1000`.
    *   This allows the kernel to easily edit page tables since it can "reach" the physical frames from virtual space.

### 4.2 Frame Allocator

To create new page tables or map new memory, we need physical frames (4KiB chunks).
*   **Strategy**: `BootInfoFrameAllocator` reads the memory map passed by the BIOS/UEFI.
*   **Logic**: It iterates through regions marked `Usable`. It returns the address of the next available frame and increments a counter. This is a linear allocator; it does not yet support freeing frames (leaks memory if pages are unmapped).

### 4.3 Dynamic Heap (`src/allocator.rs`)

To support standard Rust types like `Vec`, `Box`, and `String`, we implement a heap.

*   **Heap Address**: Fixed at specific virtual address `0x_4444_4444_0000`.
*   **Heap Size**: 100 KiB.
*   **Allocator**: We use the `linked_list_allocator` crate.
    *   The `init_heap` function manually maps the 100 KiB virtual range to new physical frames allocated by our Frame Allocator.
    *   It then initializes the `LockedHeap` global allocator, allowing safe `no_std` dynamic memory usage.

---

## 5. Development Log: Challenges & Solutions

Developing a bare-metal OS involves resolving unique errors that don't exist in standard software development.

| Issue | Technical Root Cause | Solution Implemented |
| :--- | :--- | :--- |
| **Compiler Error: E0423** | `ScancodeSet1` is a Unit Struct (zero-sized type) in the `pc-keyboard` crate, but text was treating it like a value. | Explicitly instantiated it: `ScancodeSet1::new()` or `ScancodeSet1 {}`. |
| **Bootloader Field Missing** | The `bootloader` crate v0.10+ dramatically changed the `BootInfo` struct, removing `physical_memory_offset`. | We pinned dependency to **`0.9.23`** and enabled the `map_physical_memory` feature in `Cargo.toml`. |
| **Linker Error: Alloc** | The `alloc` crate is part of the standard library but not linked by default in `no_std`. | Added `alloc` to the `build-std` array in `.cargo/config.toml` to recompile it for our custom target. |
| **Pointer Casting Panic** | The heap initializer expects a raw pointer `*mut u8` but we had a `usize` address. | Performed an explicit cast `HEAP_START as *mut u8`. |
| **Triple Fault Loops** | Occurred early on when IDT entries were malformed or the Stack Overflow handler failed. | Defining a robust **Double Fault Handler** with a dedicated IST stack prevented the CPU from resetting, allowing us to see the panic message. |

---

## 6. The Shell

The user interface (`src/shell.rs`) is a simple loop running in `main.rs`.

*   **Architecture**: Event-driven. The main loop sleeps (`hlt`) until an interrupt (Keyboard) wakes it.
*   **Commands**:
    *   `ping`: Trivial test of responsiveness.
    *   `heap_test`: Critical test. It allocates a `Box<i32>` and prints the pointer. If memory management failed, this would crash the kernel.
    *   `clear`: Simulates a screen clear by printing newlines.
