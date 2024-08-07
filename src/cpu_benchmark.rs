use std::thread;
use std::time::{Duration, Instant};
use console::Style;
use indicatif::{HumanCount, HumanDuration, HumanFloatCount, ProgressBar, ProgressStyle};
use color_eyre::eyre::{Report, Result};
use dashu::base::SquareRoot;
use dashu::float::FBig;
use dashu::integer::IBig;

pub struct CPUBenchmark {
    precision: usize,
}

impl CPUBenchmark {
    pub fn new(precision: usize) -> CPUBenchmark {
        Self { precision }
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

    pub fn run(&self){
        let value_style = Style::new().bright().green().bold().underlined();

        let bar = ProgressBar::new_spinner()
            .with_message(format!("Running PI calculation with precision {}",self.precision));
        bar.set_style(ProgressStyle::with_template("{msg} [{elapsed}] {spinner}")
            .unwrap());
        bar.enable_steady_tick(Duration::from_millis(300));
        bar.inc(0);

        let now = Instant::now();
        Self::chudnovsky(self.precision).unwrap().to_decimal().value();
        let elapsed = now.elapsed();

        bar.finish_with_message(format!("PI calculation with precision {} took {}.",
                                self.precision,
                                value_style
                                    .apply_to(HumanDuration(Duration::from_secs(elapsed.as_secs())))));
        println!();

        thread::sleep(Duration::from_secs(5));
    }
}

