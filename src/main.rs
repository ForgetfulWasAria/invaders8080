#![allow(unused)]
pub mod cpu;
pub mod instructions;
pub mod machine;

fn main() {
    let mut bus = machine::Machine::new();
    let mut cpu = cpu::CPU::build(bus);
    println!("Hello, world!");
}
