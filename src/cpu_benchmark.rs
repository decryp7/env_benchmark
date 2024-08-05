use std::time::{Duration, Instant};
use console::Style;
use indicatif::{HumanCount, HumanDuration, HumanFloatCount, ProgressBar, ProgressStyle};

pub struct CPUBenchmark {
    num_cpu_cores: u64,
    num_calculations: u64,
    num_iterations: u64,
}

impl CPUBenchmark {
    pub fn new(num_cpu_cores: u64,
               num_calculations: u64,
               num_iterations: u64) -> CPUBenchmark {
        Self { num_cpu_cores, num_calculations, num_iterations}
    }

    fn factorial(num: u128) -> u128 {
        (1..=num).product()
    }

    fn add_one_loop(&n_loops: &u64) {
        for _in in 0..n_loops {
            let _ = CPUBenchmark::factorial(20);
        }
    }

    pub fn run(&self){
        let value_style = Style::new().bright().red().bold();
        let total_calc: u64 = self.num_calculations * self.num_iterations;
        let iterations_per_core: u64 = self.num_calculations / self.num_cpu_cores;

        let bar = ProgressBar::new(self.num_iterations)
            .with_message(format!("Running {} calculations({} iterations of {} calculations) on {} threads...",
                         HumanCount(total_calc),
                         HumanCount(self.num_iterations),
                         HumanCount(self.num_calculations),
                         HumanCount(self.num_cpu_cores)));
        bar.set_style(ProgressStyle::with_template("{msg} [{elapsed}]\n{wide_bar:.cyan/blue} {pos}/{len}")
            .unwrap()
            .progress_chars("##-"));
        bar.inc(0);

        let now = Instant::now();
        for _i in 0..self.num_iterations {
            let mut results = Vec::new();
            let mut threads = Vec::new();
            for _i in 0..self.num_cpu_cores {
                threads.push(std::thread::spawn(move || CPUBenchmark::add_one_loop(&iterations_per_core)));
            }
            for thread in threads {
                results.extend(thread.join());
            }
            bar.inc(1);
        }
        bar.finish();

        let elapsed = now.elapsed();
        let calculations_per_sec: f64 = (total_calc as f64) / (elapsed.as_secs() as f64);

        println!("Total runtime: {}",
                 value_style.apply_to(HumanDuration(Duration::from_secs(elapsed.as_secs()))));
        println!("Calculations per second: {} cps.",
                 value_style.apply_to(HumanFloatCount(calculations_per_sec.round())));
    }
}

