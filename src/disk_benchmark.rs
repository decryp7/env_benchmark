use std::{alloc, cmp, env, fs, ptr, slice, thread};
use std::alloc::Layout;
use std::ffi::CString;
use std::fs::{metadata, File, OpenOptions};
use indicatif::{DecimalBytes, HumanBytes, HumanCount, HumanDuration, ProgressBar, ProgressStyle};
use std::io::{BufWriter, Write, BufReader, Read, BufRead};
#[cfg(target_os = "macos")]
use std::os::fd::FromRawFd;
#[cfg(target_os = "linux")]
use std::os::linux::fs::MetadataExt;
use std::path::Path;
use std::time::{Duration, Instant, SystemTime};
use console::Style;
use libc::{c_int, close, fileno};
use parse_size::parse_size;

trait OpenOptionsExt {
    fn disable_buffering(&mut self) -> &mut Self;
}

impl OpenOptionsExt for OpenOptions {
    #[cfg(target_os = "linux")]
    fn disable_buffering(&mut self) -> &mut Self {
        use std::os::unix::fs::OpenOptionsExt;
        self.custom_flags(libc::O_DIRECT)
    }

    #[cfg(target_os = "macos")]
    fn disable_buffering(&mut self) -> &mut Self {
        self
    }

    #[cfg(windows)]
    fn disable_buffering(&mut self) -> &mut Self {
        use std::os::windows::fs::OpenOptionsExt;
        self.custom_flags(winapi::um::winbase::FILE_FLAG_WRITE_THROUGH | winapi::um::winbase::FILE_FLAG_NO_BUFFERING)
    }
}

#[cfg(target_os = "macos")]
pub struct MacDirectIO {
    fd: c_int,
    file: File
}

#[cfg(target_os = "macos")]
impl Drop for MacDirectIO {
    fn drop(&mut self) {
        unsafe {
            let i = libc::close(self.fd);
        }
    }
}

#[cfg(target_os = "macos")]
impl MacDirectIO {
    pub fn open(path: String) -> Self {
        unsafe {
            let path = CString::new(&*path).unwrap();
            let flags = libc::O_RDWR | libc::O_CREAT;
            let mode: c_int = 0o644;
            let fd = libc::open(path.as_ptr(), flags, mode);
            let r = libc::fcntl(fd, libc::F_NOCACHE, 1);
            let f = File::from_raw_fd(fd);
            Self { fd, file: f}
        }
    }
}

#[cfg(target_os = "macos")]
impl Read for MacDirectIO {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.file.read(buf)
    }
}

#[cfg(target_os = "macos")]
impl Write for MacDirectIO {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }
}

// `O_DIRECT` requires all reads and writes
// to be aligned to the block device's block
// size.
pub struct Aligned {
    ptr: ptr::NonNull<u8>,
    layout: Layout
}

impl Aligned {
    pub fn new(size: usize, alignment: usize) -> Self {
        let layout = alloc::Layout::from_size_align(size, alignment).unwrap();
        let ptr = ptr::NonNull::new(unsafe {alloc::alloc(layout)}).unwrap();
        Self { ptr, layout }
    }

    pub fn array<'a>(&self) -> &'a mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.ptr.cast::<u8>().as_ptr(), self.layout.size()) }
    }
}

impl Drop for Aligned {
    fn drop(&mut self) {
        unsafe {
            alloc::dealloc(self.ptr.cast::<u8>().as_ptr(), self.layout);
        }
    }
}

pub struct DiskBenchmark {
    path: String,
    size: u64,
    num_iterations: u32,
    buffer_size: usize,
    alignment_size: usize
}

impl DiskBenchmark {
    pub fn new(path: String, size: u64, num_iterations: u32, buffer_size: u64) -> Self {
        let bs = buffer_size - buffer_size % 1024;
        let s = size - size % 1024;
        let p = Path::new(&path)
            .join(format!("{}.diskbenchmark", SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis()))
            .to_str()
            .unwrap()
            .to_string();

        #[cfg(not(target_os = "linux"))]
        let a = 4096;

        #[cfg(target_os = "linux")]
        let a = metadata(path).unwrap().st_blksize();

        Self {path: p, size: s, num_iterations, buffer_size: bs as usize, alignment_size: a as usize }
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

        let aligned = Aligned::new(self.buffer_size, 4096);
        let random_bytes = aligned.array();
        let mut total_elapsed = 0u64;

        for _ in 0..self.num_iterations {
            // #[cfg(target_os = "windows")]
            // if !crate::win32::Win32::clear_standby_list()
            // {
            //     println!("Unable to clear file cache. Result may not be accurate.");
            // }

            self.delete_temp_file();

            #[cfg(target_os = "macos")]
            let mut file = MacDirectIO::open(self.path.clone());

            #[cfg(not(target_os = "macos"))]
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .disable_buffering()
                .open(&self.path)
                .unwrap();

            let now = Instant::now();
            let mut remaining_size = self.size;
            while remaining_size > 0 {
                file.write_all(random_bytes).unwrap();
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

        let aligned = Aligned::new(self.buffer_size, 4096);
        let read_data = aligned.array();
        let mut total_elapsed = 0u64;

        for _ in 0..self.num_iterations {
            // #[cfg(target_os = "windows")]
            // if !crate::win32::Win32::clear_standby_list()
            // {
            //     println!("Unable to clear file cache. Result may not be accurate.");
            // }

            #[cfg(target_os = "macos")]
            let mut file = MacDirectIO::open(self.path.clone());

            #[cfg(not(target_os = "macos"))]
            let mut file = OpenOptions::new()
                .read(true)
                .disable_buffering()
                .open(&self.path)
                .unwrap();

            let now = Instant::now();


            let mut size = file.read(read_data).unwrap();
            while size > 0 {
                size = file.read(read_data).unwrap();
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