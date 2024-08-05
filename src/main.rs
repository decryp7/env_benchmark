mod cpu_benchmark;
mod disk_benchmark;

use std::io::{Read};
use std::thread::available_parallelism;
use console::{Style};
use sysinfo::{System};
use crate::cpu_benchmark::CPUBenchmark;

fn main() {
    let num_calculations = 100_000_000;
    let num_iterations = 10;

    let mut sys = System::new_all();
    sys.refresh_all();
    let system_info_style = Style::new().bright().green().bold();
    println!("{:<30}{:<10}", "System name:", system_info_style.apply_to(System::name()
        .unwrap_or(String::from("Unknown"))));
    println!("{:<30}{:<10}", "System kernel version:", system_info_style.apply_to(System::kernel_version()
        .unwrap_or(String::from("Unknown"))));
    println!("{:<30}{:<10}", "System OS version:", system_info_style.apply_to(System::os_version()
                .unwrap_or(String::from("Unknown"))));
    println!("{:<30}{:<10}", "Number of CPU threads:", system_info_style.apply_to(sys.cpus().len()));
    println!();

    let mut cpu_benchmark = CPUBenchmark::new(1, num_calculations, num_iterations);
    cpu_benchmark.run();
    println!();

    let available_cores: u64 = available_parallelism().unwrap().get() as u64;
    cpu_benchmark = CPUBenchmark::new(available_cores, num_calculations, num_iterations);
    cpu_benchmark.run();
    println!();

    let term = console::Term::stdout();
    let mut character = term.read_char().unwrap();
    while character != 'q' {
        character = term.read_char().unwrap();
    }
}
