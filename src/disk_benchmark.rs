use std::{env, fs, thread};
use std::fs::metadata;
use indicatif::{HumanBytes, HumanCount, HumanDuration, ProgressBar, ProgressStyle};
use std::io::{BufWriter, Write, BufReader, Read};
use std::time::{Duration, Instant};
use console::Style;

pub struct DiskBenchmark {
    path: String,
    size: u64
}

impl DiskBenchmark {
    pub fn new(path: String, size: u64) -> Self {
        Self {path, size}
    }

    pub fn run(&self){
        if self.delete_temp_file() {
            self.run_write();
            self.run_read();
            self.delete_temp_file();
        }else{
            println!("Unable to delete {}. Skipping disk benchmark.", self.path);
        }
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
        let value_style = Style::new().bright().red().bold();
        let bar = ProgressBar::new(self.size)
            .with_message(format!("Writing {} of size {}...",
                                  self.path,
                                  HumanBytes(self.size)));
        bar.set_style(ProgressStyle::with_template("{msg} [{elapsed}]\n{wide_bar:.cyan/blue} {bytes}/{total_bytes}")
            .unwrap()
            .progress_chars("##-"));
        bar.inc(0);

        let mut content = [0; 1024];
        for i in 0..content.len() {
            content[i]  = 1;
        }

        let now = Instant::now();
        let mut remaining_size = self.size;
        let mut f = BufWriter::new(fs::File::create(&self.path)
            .unwrap());
        while(remaining_size > 0) {
            f.write(&content).unwrap();
            if remaining_size >= 1024 {
                remaining_size -= 1024;
            }else {
                remaining_size = 0;
            }
            bar.inc(1024);
        }

        bar.finish();
        let elapsed = now.elapsed();
        println!("Write took {}.",
                 value_style.apply_to(HumanDuration(Duration::from_secs(elapsed.as_secs()))));
    }

    fn run_read(&self) {
        let value_style = Style::new().bright().red().bold();
        let bar = ProgressBar::new(self.size)
            .with_message(format!("Reading {} of size {}...",
                                  self.path,
                                  HumanBytes(self.size)));
        bar.set_style(ProgressStyle::with_template("{msg} [{elapsed}]\n{wide_bar:.cyan/blue} {bytes}/{total_bytes}")
            .unwrap()
            .progress_chars("##-"));
        bar.inc(0);

        let mut content = [0; 1024];
        let now = Instant::now();
        let mut f = BufReader::new(fs::File::open(&self.path)
            .unwrap());
        let mut size = f.read(&mut content).unwrap();
        while(size > 0) {
            bar.inc(size as u64);
            size = f.read(&mut content).unwrap();
        }

        bar.finish();
        let elapsed = now.elapsed();
        println!("Read took {}.",
                 value_style.apply_to(HumanDuration(Duration::from_secs(elapsed.as_secs()))));
    }
}