mod cpu_benchmark;
mod disk_benchmark;
#[cfg(target_os = "windows")]
mod win32;

use std::{env, mem};
use std::hint::black_box;
use std::io::{Read};
use std::path::Path;
use std::thread::available_parallelism;
use std::time::Instant;
use console::{Style};
use parse_size::parse_size;
use sysinfo::{System};
use crate::cpu_benchmark::CPUBenchmark;
use crate::disk_benchmark::DiskBenchmark;

fn main() {
    let num_calculations = 1000;
    let num_iterations = 5;

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

    // let now = Instant::now();
    // let mut result = CPUBenchmark::chudnovsky(1000).unwrap();
    // println!("{} in {}.", result.to_decimal().value(), now.elapsed().as_secs());

    let mut cpu_benchmark = CPUBenchmark::new(1, num_calculations, num_iterations);
    cpu_benchmark.run();
    println!();

    let available_cores: u64 = available_parallelism().unwrap().get() as u64;
    cpu_benchmark = CPUBenchmark::new(available_cores, num_calculations, num_iterations);
    cpu_benchmark.run();
    println!();

    let disk_benchmark = DiskBenchmark::new(Path::new(env::temp_dir().as_os_str())
                                                .join("disk.benchmark").to_str().unwrap().to_string(),
                                            parse_size("4 GB").unwrap());
    disk_benchmark.run();
    println!();

    let term = console::Term::stdout();
    let mut character = term.read_char().unwrap();
    while character != 'q' {
        character = term.read_char().unwrap();
    }
}
