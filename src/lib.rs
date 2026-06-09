use std::thread;
use std::io::Write;
use std::sync::mpsc::{self, Receiver, Sender};

#[derive(Debug, Clone)]
pub enum BarStyle {
    SimpleBar,
    Bar,
    RotatingSlash,
}

#[derive(Debug, Clone)]
pub struct ProgressBar {
    style: BarStyle,

    value_range: (i32, i32),
    current_value: i32,

    taskname: String,

    tx: Sender<i32>,
}

#[derive(Debug, Clone)]
pub enum ProgressBarError {
    OutOfRange {
        range: (i32, i32),
        current: i32
    },
    InvalidDefinition((i32, i32)),
}

pub type ProgressBarResult = Result<ProgressBar, ProgressBarError>;

impl ProgressBar {
    pub fn new(taskname: &str, style: BarStyle, val: i32, range: (i32, i32)) -> ProgressBarResult {
        let (ttx, rrx): (Sender<i32>, Receiver<i32>) = mpsc::channel();

        if range.0 > range.1 {
            return Err(ProgressBarError::InvalidDefinition(range));
        }

        let pb = ProgressBar{
            style,

            current_value: val,
            value_range: range,

            taskname: taskname.to_string(),
            tx: ttx,
        };

        let mpb = pb.clone();

        thread::spawn(move || {
            let terminal_width = termize::dimensions_stdout().expect("failed to get terminal size").0;
            let pb_width = terminal_width - " 100% [] ".len() - mpb.taskname.len();

            loop {

                let mut str = String::new();
                let current_value = rrx.recv().expect("failed to get value") as usize;

                let total_range = (current_value as f64 + 1f64 - mpb.get_min() as f64) / (mpb.get_max() - mpb.get_min()) as f64;
                let progress_ratio = total_range.clamp(0.0, 1.0);

                let filled = (progress_ratio * pb_width as f64).round() as usize;

                str.push_str(&"=".repeat(filled));
                str.push_str(&"-".repeat(pb_width - filled));

                match mpb.style {
                    BarStyle::SimpleBar => {
                        print!("{} [{}] {:3}% \r", mpb.taskname, str, ((current_value as f64 / (pb.value_range.1 as f64 - 1f64)) * 100f64) as i32);
                    }
                    _ => {}
                }

                if current_value == (mpb.get_max() - 1) as usize {
                    println!();
                    break;
                }

                std::io::stdout().flush().expect("Failed to flush stdout");
            }
        });

        Ok(pb)
    }

    pub fn set_val(&mut self, val: i32) {
        self.tx.send(self.current_value).expect("receiving between threads failed");
        self.current_value = val;
    }

    pub fn get_val(&self) -> i32 {
        self.current_value
    }

    pub fn get_range(&self) -> (i32, i32) {
        self.value_range
    }

    pub fn get_min(&self) -> i32 {
        self.value_range.0
    }

    pub fn get_max(&self) -> i32 {
        self.value_range.1
    }
}