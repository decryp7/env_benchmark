use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;
use std::time::{Duration, Instant};
use console::Style;
use indicatif::{HumanCount, HumanDuration, HumanFloatCount, ProgressBar, ProgressStyle};
use color_eyre::eyre::{Report, Result};
use dashu::base::SquareRoot;
use dashu::float::FBig;
use dashu::integer::IBig;
use sysinfo::System;

pub struct CPUBenchmark {
    num_cpu_threads: usize,
    precision: usize,
    num_iterations: u32,
    num_calculations: u32,
    remaining_calculations: AtomicU32,
}

impl CPUBenchmark {
    pub fn new(num_cpu_threads: usize,
                precision: usize,
               num_iterations: u32,
               num_calculations: u32) -> CPUBenchmark {
        Self {num_cpu_threads, 
            precision, 
            num_iterations, 
            num_calculations,
            remaining_calculations: AtomicU32::new(0)}
    }

    fn binary_split(a: u32, b: u32) -> (IBig, IBig, IBig) {
        if b - a == 1 {
            if a == 0 {
                let pab = IBig::from(1);
                let qab = IBig::from(1);
                let rab = IBig::from(&pab * (13_591_409 + 545_140_134 * a));
                return (pab, qab, rab);
            }
            let a_bigint = IBig::from(a);
            let pab: IBig = (IBig::from(6 * &a_bigint) - 5)
                * (IBig::from(2 * &a_bigint) - 1)
                * (IBig::from(6 * &a_bigint) - 1);
            let qab = a_bigint.clone().pow(3) * 10_939_058_860_032_000u64;
            let rab = &pab * (13_591_409 + 545_140_134 * a_bigint);

            if a % 2 == 0 {
                return (pab, qab, rab);
            }
            return (pab, qab, -1 * rab);
        }
        let m = (a + b) / 2;
        let (pam, qam, ram) = Self::binary_split(a, m);
        let (pmb, qmb, rmb) = Self::binary_split(m, b);
        let p1n = IBig::from(&pam * &pmb);
        let q1n = IBig::from(&qam * &qmb);
        let r1n = IBig::from(&ram * &qmb) + IBig::from(&pam * &rmb);
        (p1n, q1n, r1n)
    }

    /// https://github.com/mikeleppane/mojo-rust-python-perf/blob/main/pidigits_rust/src/lib.rs
    /// Returns an error if the input is invalid.
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_precision_loss)]
    pub fn chudnovsky(digits: usize) -> Result<FBig> {
        match digits {
            0 => return Ok(FBig::try_from(3f64).unwrap().with_precision(53).unwrap()),
            1 => return Ok(FBig::try_from(3.1f64).unwrap().with_precision(53).unwrap()),
            _ => {
                if digits.checked_mul(4).is_none() {
                    return Err(Report::msg(
                        "Invalid digits: value must be between 0 <= x < (2^32-1)/4",
                    ));
                }
            }
        }
        let used_precision = digits * 4;
        let digits_per_term = f32::log10(10_939_058_860_032_000_f32 / 6f32 / 2f32 / 6f32);
        let n = (digits as f32 / digits_per_term).ceil() as u32;
        let i1 = IBig::from(426_880);
        let i2 = FBig::try_from( 10_005).unwrap().with_precision(used_precision).unwrap();

        let (_, q1n, r1n) = Self::binary_split(0, n);
        Ok(((i1 * i2.sqrt() * q1n) / r1n).into())
    }

    pub fn single_thread_run(&self){
        let value_style = Style::new().bright().green().bold().underlined();

        let bar = ProgressBar::new(self.num_iterations as u64)
            .with_message(format!("Running PI calculation with precision {} for {} times",
                                  self.precision,
                                  self.num_iterations));
        bar.set_style(ProgressStyle::with_template("{msg} [{elapsed}]\n{wide_bar:.cyan/blue} {pos}/{len}")
            .unwrap()
            .progress_chars("##-"));
        bar.enable_steady_tick(Duration::from_secs(1));
        bar.inc(0);

        let mut measurements = vec![0; self.num_iterations as usize];
        for _ in 0..self.num_iterations {
            let now = Instant::now();
            Self::chudnovsky(self.precision).unwrap().to_decimal().value();
            let elapsed = now.elapsed();
            measurements.push(elapsed.as_millis());
            bar.inc(1);
        }
        bar.finish();
        let average= measurements.iter().sum::<u128>() / self.num_iterations as u128;

        println!("PI calculation (single thread) with precision {} took {} on average.",
                 self.precision,
                 value_style
                     .apply_to(HumanDuration(Duration::from_millis(average as u64))));
    }

    fn multithread_one_iteration(&self) {
        thread::scope(|s| {
            let mut threads = Vec::new();
            let mut results = Vec::new();

            for _ in 0..self.num_cpu_threads {
                threads.push(s.spawn(move || {
                    while self.remaining_calculations.load(Ordering::Acquire) > 0 {
                        self.remaining_calculations.fetch_sub(1, Ordering::Acquire);
                        Self::chudnovsky(self.precision).unwrap().to_decimal().value();
                        // println!("[Thread {:?}] Remaining calculations: {}.",thread::current().id(),
                        //          self.remaining_calculations.load(Ordering::Acquire));
                    }
                }));
            }

            for thread in threads {
                results.extend(thread.join());
            }
        });
    }

    pub fn multithread_run(&self){
        self.remaining_calculations.swap(self.num_calculations, Ordering::Acquire);
        
        let value_style = Style::new().bright().green().bold().underlined();
        let bar = ProgressBar::new(self.num_iterations as u64)
            .with_message(format!("Running {} PI calculations with precision {} on {} threads for {} times",
                                  self.num_calculations,
                                  self.precision,
                                  self.num_cpu_threads,
                                  self.num_iterations));
        bar.set_style(ProgressStyle::with_template("{msg} [{elapsed}]\n{wide_bar:.cyan/blue} {pos}/{len}")
            .unwrap()
            .progress_chars("##-"));
        bar.enable_steady_tick(Duration::from_secs(1));
        bar.inc(0);

        let mut measurements = vec![0; self.num_iterations as usize];
        for _ in 0..self.num_iterations {
            let now = Instant::now();
            self.multithread_one_iteration();
            let elapsed = now.elapsed();
            measurements.push(elapsed.as_millis());
            bar.inc(1);
        }
        bar.finish();
        let average= measurements.iter().sum::<u128>() / self.num_iterations as u128;

        println!("{} PI calculations with precision {} took {} on average.",
                 self.num_calculations,
                 self.precision,
                 value_style
                     .apply_to(HumanDuration(Duration::from_millis(average as u64))));
    }

    pub fn run(&self){
        self.single_thread_run();
        println!();
        thread::sleep(Duration::from_secs(5));

        self.multithread_run();
    }
}

