use std::{cmp, env, fs, thread};
use std::fs::metadata;
use indicatif::{DecimalBytes, HumanBytes, HumanCount, HumanDuration, ProgressBar, ProgressStyle};
use std::io::{BufWriter, Write, BufReader, Read, BufRead};
use std::time::{Duration, Instant};
use console::Style;

pub struct DiskBenchmark {
    path: String,
    size: u64,
    num_iterations: i32,
}

impl DiskBenchmark {
    pub fn new(path: String, size: u64, num_iterations: i32) -> Self {
        Self {path, size, num_iterations}
    }

    pub fn run(&self){
        self.run_write();
        println!();

        thread::sleep(Duration::from_secs(5));
        self.run_read();
    }

    fn delete_temp_file(&self) -> bool {
        if metadata(&self.path).is_ok() {
            return match fs::remove_file(&self.path) {
                Ok(_) => {
                    true
                }
                Err(e) => {
                    false
                }
            }
        }

        true
    }

    fn run_write(&self){
        let value_style = Style::new().bright().green().bold().underlined();
        let bar = ProgressBar::new(self.num_iterations as u64)
            .with_message(format!("Writing {} of size {} {} times... ",
                                  self.path,
                                  DecimalBytes(self.size),
            self.num_iterations));
        bar.set_style(ProgressStyle::with_template("{msg} [{elapsed}]\n{wide_bar:.cyan/blue} {pos}/{len}")
            .unwrap()
            .progress_chars("##-"));
        bar.enable_steady_tick(Duration::from_secs(1));
        bar.inc(0);


        const BUF_SIZE: usize = 8 * 1024;
        let random_bytes: Vec<u8> = vec![1; BUF_SIZE];
        let mut total_elapsed = 0u64;

        for _ in 0..self.num_iterations {
            #[cfg(target_os = "windows")]
            if !crate::win32::Win32::clear_standby_list()
            {
                println!("Unable to clear file cache. Result may not be accurate.");
            }

            self.delete_temp_file();

            let now = Instant::now();
            let mut remaining_size = self.size;
            let mut f = BufWriter::with_capacity(BUF_SIZE, fs::File::create(&self.path)
                .unwrap());
            while remaining_size > 0 {
                f.write(&random_bytes).unwrap();
                if remaining_size >= BUF_SIZE as u64 {
                    remaining_size -= BUF_SIZE as u64;
                } else {
                    remaining_size = 0;
                }
            }
            total_elapsed += now.elapsed().as_secs();
            bar.inc(1);
        }

        bar.finish();
        let average= (self.size * self.num_iterations as u64) / cmp::max(total_elapsed, 1);

        println!("Write took {} on average.",
                 value_style.apply_to(format!("{}/s",DecimalBytes(average))));
    }

    fn run_read(&self) {
        const BUF_SIZE: usize = 1000 * 1024;
        let value_style = Style::new().bright().green().bold().underlined();
        let bar = ProgressBar::new(self.num_iterations as u64)
            .with_message(format!("Reading {} of size {} {} times...",
                                  self.path,
                                  DecimalBytes(self.size),
                                self.num_iterations));
        bar.set_style(ProgressStyle::with_template("{msg} [{elapsed}]\n{wide_bar:.cyan/blue} {decimal_bytes}/{decimal_total_bytes}")
            .unwrap()
            .progress_chars("##-"));
        bar.enable_steady_tick(Duration::from_secs(1));
        bar.inc(0);

        let mut read_data =  vec![0; BUF_SIZE];
        let mut total_elapsed = 0u64;

        for _ in 0..self.num_iterations {
            #[cfg(target_os = "windows")]
            if !crate::win32::Win32::clear_standby_list()
            {
                println!("Unable to clear file cache. Result may not be accurate.");
            }

            let now = Instant::now();
            let mut f = BufReader::with_capacity(BUF_SIZE, fs::File::open(&self.path)
                .unwrap());
            let mut size = f.read(read_data.as_mut_slice()).unwrap();
            while size > 0 {
                size = f.read(read_data.as_mut_slice()).unwrap();
            }
            total_elapsed += now.elapsed().as_secs();
            bar.inc(1);
        }

        bar.finish();
        let average= (self.size * self.num_iterations as u64) / cmp::max(total_elapsed, 1);

        println!("Read took {} on average.",
                 value_style.apply_to(format!("{}/s",DecimalBytes(average))));

        self.delete_temp_file();
    }
}