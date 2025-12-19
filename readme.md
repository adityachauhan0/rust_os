
# Valhalla OS 🌌

A 64-bit micro-kernel written in **Rust** for the x86_64 architecture.

Valhalla OS demonstrates the core principles of operating system design, including hardware abstraction, interrupt handling, and complex memory management through a 4-level paging system and a dynamic heap allocator.
<img width="763" height="573" alt="Screenshot 2025-12-18 162447" src="https://github.com/user-attachments/assets/c0445b7c-6db6-4d76-bdeb-89505e57d564" />

📖 **[Read the Documentation](documentation.md)** for a detailed technical breakdown.

## 🚀 How to Run

### Prerequisites

1. **Rust Nightly:** This project uses experimental features.
```bash
rustup override set nightly

```


2. **Bootimage Tool:** To create the bootable disk image.
```bash
cargo install bootimage

```


3. **QEMU:** For hardware emulation.
* **Ubuntu:** `sudo apt install qemu-system-x86`
* **Windows:** Install via [qemu.org](https://www.qemu.org/)



### Execution

1. Clone the repository:
```bash
git clone https://github.com/adityachauhan0/valhalla_os
cd valhalla_os

```


2. Build and Run:
```bash
cargo run

```


*The kernel will compile, link with the bootloader, and launch a QEMU window.*

---

## 🛠 Feature Log

### Level 0: The Foundation

* **VGA Text Buffer Driver:** Custom implementation of the `print!` and `println!` macros to interface with the `0xb8000` memory-mapped I/O.
* **Stack Overflow Protection:** Implemented a **Global Descriptor Table (GDT)** with an Interrupt Stack Table (IST) to catch stack overflows via a Double Fault handler.

### Level 1: Interrupts & Input

* **IDT Setup:** Created an Interrupt Descriptor Table to handle CPU exceptions (Breakpoint, Double Fault, Page Fault).
* **Hardware Interrupts:** Remapped the 8259 PIC to handle Timer and Keyboard interrupts.
* **PS/2 Keyboard Driver:** Scancode processing using `pc-keyboard` supporting US-layout and basic control keys.

### Level 2: Memory Management (The Crown Jewel)

* **Paging:** Implemented a 4-level page table walker to translate virtual addresses to physical RAM.
* **Frame Allocation:** A `BootInfoFrameAllocator` that parses the BIOS memory map to identify usable RAM regions.
* **Dynamic Memory (Heap):** Integrated the `linked_list_allocator` to enable `Box`, `Vec`, and `String` in a `no_std` environment.
* **Interactive Shell:** A command-line interface supporting commands like `ping`, `clear`, and `heap_test`.

---

## 🪵 Development Log & Error Resolution

Building Valhalla OS involved overcoming several "bare-metal" hurdles. Below are the most significant errors faced and their solutions:

| Error | Cause | Resolution |
| --- | --- | --- |
| `E0423: expected value, found struct ScancodeSet1` | `ScancodeSet1` is a unit struct; the compiler was confused between the type and value. | Changed the initialization to `ScancodeSet1 {}` to be explicit. |
| `No field physical_memory_offset` | Bootloader v0.10+ removed this field from the default struct. | Downgraded to `bootloader 0.9.23` and enabled the `map_physical_memory` feature. |
| `can't find crate for alloc` | The compiler didn't know how to link the allocation library to a custom target. | Updated `.cargo/config.toml` to include `alloc` in the `build-std` list. |
| `Mismatched types: expected *mut u8, found usize` | The heap allocator required a raw pointer for initialization. | Used `HEAP_START as *mut u8` to cast the virtual address correctly. |
| `CPU Triple Fault` | Usually caused by an unhandled interrupt or a bad GDT entry. | Implemented a proper Double Fault handler to catch the error before the CPU resets. |

---

## 📜 Commands

Once booted into Valhalla OS, you can use the following commands in the shell:

* `help`: Displays available commands.
* `ping`: Verifies CPU and interrupt responsiveness.
* `heap_test`: Performs a dynamic allocation of a `Box` and a `Vec` to verify the Memory Manager.
* `clear`: Clears the VGA text buffer.

---
