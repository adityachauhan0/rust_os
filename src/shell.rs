use crate::print;
use crate::println;
use alloc::string::String;
use alloc::vec::Vec;

pub fn interpret(command: Vec<char>) {
    let cmd_str: String = command.into_iter().collect();
    let parts: Vec<&str> = cmd_str.split_whitespace().collect();

    if parts.is_empty() {
        print!("> ");
        return;
    }

    match parts[0] {
        "help" => {
            println!("Commands: help, clear, ping, heap_test");
        }
        "ping" => {
            println!("Pong! Kernel is responsive.");
        }
        "clear" => {
            // A simple way to 'clear' is to print many lines
            for _ in 0..25 {
                println!("");
            }
        }
        "heap_test" => {
            let x = alloc::boxed::Box::new(42);
            println!("Successfully allocated {} at {:p}", x, x);
        }
        _ => {
            println!("Unknown command: {}", parts[0]);
        }
    }
    print!("> ");
}
