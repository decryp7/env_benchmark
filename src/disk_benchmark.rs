use std::{cmp, env, fs, thread};
use std::fs::{metadata, OpenOptions};
use indicatif::{DecimalBytes, HumanBytes, HumanCount, HumanDuration, ProgressBar, ProgressStyle};
use std::io::{BufWriter, Write, BufReader, Read, BufRead};
use std::time::{Duration, Instant};
use console::Style;
use parse_size::parse_size;

trait OpenOptionsExt {
    fn disable_buffering(&mut self) -> &mut Self;
}

const O_DIRECT: i32 = 0o0040000;

impl OpenOptionsExt for OpenOptions {
    #[cfg(target_os = "linux")]
    fn disable_buffering(&mut self) -> &mut Self {
        use std::os::unix::fs::OpenOptionsExt;
        self.custom_flags(O_DIRECT)
    }

    #[cfg(target_os = "macos")]
    fn disable_buffering(&mut self) -> &mut Self {
        use std::os::unix::fs::OpenOptionsExt;
        self.custom_flags(O_DIRECT)
    }

    #[cfg(windows)]
    fn disable_buffering(&mut self) -> &mut Self {
        use std::os::windows::fs::OpenOptionsExt;
        self.custom_flags(winapi::um::winbase::FILE_FLAG_WRITE_THROUGH | winapi::um::winbase::FILE_FLAG_NO_BUFFERING | winapi::um::winbase::FILE_FLAG_RANDOM_ACCESS)
    }
}

pub struct DiskBenchmark {
    path: String,
    size: u64,
    num_iterations: u32,
    buffer_size: usize,
}

impl DiskBenchmark {
    pub fn new(path: String, size: u64, num_iterations: u32, buffer_size: u64) -> Self {
        let bs = buffer_size - buffer_size % 1024;
        let s = size - size % 1024;
        Self {path, size: s, num_iterations, buffer_size: bs as usize}
    }

    pub fn run(&self){
        self.run_write();
        println!();
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

        let random_bytes: Vec<u8> = vec![1; self.buffer_size];
        let mut total_elapsed = 0u64;

        for _ in 0..self.num_iterations {
            // #[cfg(target_os = "windows")]
            // if !crate::win32::Win32::clear_standby_list()
            // {
            //     println!("Unable to clear file cache. Result may not be accurate.");
            // }

            self.delete_temp_file();

            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .disable_buffering()
                .open(&self.path)
                .unwrap();

            let now = Instant::now();
            let mut remaining_size = self.size;
            while remaining_size > 0 {
                file.write_all(&random_bytes).unwrap();
                if remaining_size >= self.buffer_size as u64 {
                    remaining_size -= self.buffer_size as u64;
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
        let value_style = Style::new().bright().green().bold().underlined();
        let bar = ProgressBar::new(self.num_iterations as u64)
            .with_message(format!("Reading {} of size {} {} times...",
                                  self.path,
                                  DecimalBytes(self.size),
                                self.num_iterations));
        bar.set_style(ProgressStyle::with_template("{msg} [{elapsed}]\n{wide_bar:.cyan/blue} {pos}/{len}")
            .unwrap()
            .progress_chars("##-"));
        bar.enable_steady_tick(Duration::from_secs(1));
        bar.inc(0);

        let mut read_data =  vec![0; self.buffer_size];
        let mut total_elapsed = 0u64;

        for _ in 0..self.num_iterations {
            // #[cfg(target_os = "windows")]
            // if !crate::win32::Win32::clear_standby_list()
            // {
            //     println!("Unable to clear file cache. Result may not be accurate.");
            // }

            let now = Instant::now();
            let mut file = OpenOptions::new()
                .read(true)
                .disable_buffering()
                .open(&self.path)
                .unwrap();

            let mut size = file.read(read_data.as_mut_slice()).unwrap();
            while size > 0 {
                size = file.read(read_data.as_mut_slice()).unwrap();
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