mod cpu_benchmark;
mod disk_benchmark;
#[cfg(target_os = "windows")]
mod win32;

use std::{env, mem, thread};
use std::fs::metadata;
use std::hint::black_box;
use std::io::{Read};
use std::path::Path;
use std::sync::Arc;
use std::thread::available_parallelism;
use std::time::Instant;
use clap::{arg, Arg, Parser};
use console::{Style};
use indicatif::DecimalBytes;
use parse_size::{parse_size, Error};
use sysinfo::{System};
use crate::cpu_benchmark::CPUBenchmark;
use crate::disk_benchmark::DiskBenchmark;

///Environment benchmark program to compare relative performance between virtual and physical machine
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    ///Total number of calculations to execute for CPU multicore test
    #[arg(short, long, default_value_t = 20)]
    num_calculations: u32,

    ///Number of iterations to execute to get the average result
    #[arg(short, long, default_value_t = 5)]
    iterations: u32,

    ///PI accuracy to number of  decimal points
    #[arg(short, long, default_value_t = 3000)]
    pi_precision: u32,

    ///Size of benchmark file for testing file read and write performance
    #[arg(short, long, default_value = "4GB")]
    filesize: String,

    ///Read and Write buffer size
    #[arg(short, long, default_value = "100MB")]
    buffer_size: String,

    ///Location of benchmark file. Change this to benchmark other storage locations
    #[arg(short, long, default_value_t = env::temp_dir().into_os_string().into_string().unwrap())]
    temp_file_directory: String
}

fn main() {
    let args = Args::parse();

    let num_calculations = (args.num_calculations > 0).then(|| args.num_calculations).or_else(|| Some(20)).unwrap();
    let num_iterations = (args.iterations > 0).then(|| args.iterations).or_else(|| Some(5)).unwrap();
    let precision = (args.pi_precision > 0).then(|| args.pi_precision).or_else(|| Some(3000)).unwrap() as usize;
    let mut file_size = parse_size("4GB").unwrap();
    let mut buffer_size = parse_size("100MB").unwrap();
    let mut file_path = env::temp_dir().into_os_string().into_string().unwrap();

    match parse_size(args.filesize) {
        Ok(f) => {
            file_size = f;
        }
        Err(_) => {}
    }

    match parse_size(args.buffer_size) {
        Ok(f) => {
            buffer_size = f;
        }
        Err(_) => {}
    }

    let metadata = metadata(&args.temp_file_directory);
    if metadata.is_ok() && metadata.unwrap().is_dir() {
        file_path = args.temp_file_directory;
    }

    let mut sys = System::new_all();
    sys.refresh_all();
    let system_info_style = Style::new().bright().green().bold();
    println!("{:<30}{:<10}", "System name:", system_info_style.apply_to(System::name()
        .unwrap_or(String::from("Unknown"))));
    println!("{:<30}{:<10}", "System kernel version:", system_info_style.apply_to(System::kernel_version()
        .unwrap_or(String::from("Unknown"))));
    println!("{:<30}{:<10}", "System OS version:", system_info_style.apply_to(System::long_os_version()
                .unwrap_or(String::from("Unknown"))));
    println!("{:<30}{:<10}", "Number of CPU threads:", system_info_style.apply_to(sys.cpus().len()));
    println!("{:<30}{:<10}", "Available memory:", system_info_style.apply_to(format!("{}/{}", DecimalBytes(sys.available_memory()), DecimalBytes(sys.total_memory()))));
    println!();

    let mut cpu_benchmark = Arc::new(CPUBenchmark::new(precision,
                                                   num_iterations,
                                                   1));
    cpu_benchmark.run();
    println!();

    cpu_benchmark = Arc::new(CPUBenchmark::new(precision,
                                                num_iterations,
                                                num_calculations));
    cpu_benchmark.run();
    println!();

    let disk_benchmark = DiskBenchmark::new(file_path,
                                            file_size,
                                            num_iterations,
                                            buffer_size);
    disk_benchmark.run();
    println!();

    println!("Benchmark completed!");
    let term = console::Term::stdout();
    let mut character = term.read_char().unwrap();
    while character != 'q' {
        character = term.read_char().unwrap();
    }
}
