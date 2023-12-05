use std::sync::{ Arc, Mutex };
use std::thread::JoinHandle;
use std::io::Read;

use crate::highlighter::MyHighlighter;
use crate::log::ColorLog;

pub struct LogcatWorker {
    device_name: String,
    logcat_sender: Option<std::process::Child>,
    logcat_receiver: Option<JoinHandle<()>>,
    logcat_buffer: Option<Arc<Mutex<Vec<u8>>>>,
    highlighter: MyHighlighter,
    filter: Option<String>,
    //logs: Vec<Arc<egui::Galley>>,
    logs: Vec<ColorLog>,
}

impl LogcatWorker {
    pub fn new(device_name: &str) -> Self {
        LogcatWorker {
            device_name: device_name.to_string(),
            //ip: String::new(),
            //port: String::new(),
            logcat_sender: None,
            logcat_receiver: None,
            logcat_buffer: None,
            highlighter: MyHighlighter::new(),
            filter: None,
            logs: Vec::new(),
        }
    }

    pub fn close(&mut self) {
        println!("close logcat {}", self.device_name);
        if self.logcat_sender.is_some() {
            let ret = self.logcat_sender.take().unwrap().kill();
            match ret {
                Ok(_) => {}
                Err(err) => {
                    println!("kill logcat {} error: {}", self.device_name, err);
                }
            }
            self.logcat_sender = None;
        }
        if self.logcat_receiver.is_some() {
            let ret = self.logcat_receiver.take().unwrap().join();
            match ret {
                Ok(_) => {}
                Err(_) => {
                    println!("join logcat {} error", self.device_name);
                }
            }
            self.logcat_receiver = None;
        }
        if self.logcat_buffer.is_some() {
            self.logcat_buffer = None;
        }
    }

    pub fn connect(&mut self, adb_path: &str) {
        let path = adb_path.trim().trim_matches('"');
        let output = std::process::Command
            ::new(path)
            .arg("-s")
            .arg(&self.device_name)
            .arg("logcat")
            .stdout(std::process::Stdio::piped())
            .spawn();
        if output.is_err() {
            println!("adb logcat error: {}, {}", path, output.err().unwrap());
            return;
        }

        let mut sender = output.unwrap();

        let mut stdout = sender.stdout.take().expect("!stdout");

        let buffer = Arc::new(Mutex::new(Vec::with_capacity(512)));
        self.logcat_buffer = Some(buffer.clone());

        let receiver = std::thread::spawn(move || {
            let mut line_buf = Vec::with_capacity(512);
            let mut byte = [0u8; 1];
            loop {
                match stdout.read(&mut byte) {
                    Err(err) => {
                        println!("{}] Error reading from stream: {}", line!(), err);
                        break;
                    }
                    Ok(got) => {
                        if got == 0 {
                            break;
                        }
                        line_buf.push(byte[0]);
                        if byte[0] == b'\n' {
                            let mut vec = buffer.lock().expect("!lock");
                            vec.append(&mut line_buf);
                            line_buf.clear();
                        }
                    }
                }
            }
        });
        self.logcat_sender = Some(sender);
        self.logcat_receiver = Some(receiver);
    }

    pub fn clear(&mut self, adb_path: &str) {
        let path = adb_path.trim().trim_matches('"');
        let output = std::process::Command
            ::new(path)
            .arg("-s")
            .arg(&self.device_name)
            .arg("logcat")
            .arg("-c")
            .output();
        if output.is_err() {
            println!("adb logcat error: {}, {}", path, output.err().unwrap());
            return;
        }
        self.logs.clear();
    }

    pub fn set_fliter(&mut self, filter: Option<String>) {
        self.filter = filter;
    }

    pub fn update(&mut self, ui: &mut egui::Ui) {
        if self.logcat_buffer.is_none() {
            return;
        }
        let mut buffer = self.logcat_buffer.as_ref().unwrap().lock().expect("!lock");
        if buffer.len() == 0 {
            return;
        }
        let mut vec = Vec::new();
        std::mem::swap(&mut vec, &mut buffer);
        let string = unsafe { String::from_utf8_unchecked(vec) };

        let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
            let (mut layout_job, log) = self.highlighter.highlighter(string);
            layout_job.wrap.max_width = wrap_width;

            let g = ui.fonts(|f| f.layout_job(layout_job));

            let color_log = ColorLog {
                raw: log,
                gallery: g.clone(),
                bottom: 0.0,
            };
            //println!("g rows{:?}", g.rows.len());
            return color_log;
        };

        for line in string.lines() {
            let mut log = layouter(ui, line, ui.available_width());
            let height = log.gallery.rect.height() + ui.style().spacing.item_spacing.y;
            let last_bottom = self.logs
                .last()
                .map(|l| l.bottom)
                .unwrap_or(0.0);
            log.bottom = last_bottom + height;
            self.logs.push(log);
        }
    }

    pub fn get_logs(&mut self) -> Vec<&ColorLog> {
        let mut logs_show = Vec::new();
        for log in self.logs.iter() {
            if self.filter.is_some() {
                let filter = self.filter.as_ref().unwrap();
                let text = log.raw.origin.as_str();
                if !text.contains(filter) {
                    continue;
                }
            }
            // expansive operation
            // if self.frame_count % 60 == 0 && g.job.wrap.max_width != ui.available_width() {
            //     let mut job = egui::text::LayoutJob::default();
            //     job.clone_from(&g.job);
            //     job.wrap.max_width = ui.available_width();
            //     let new_g = ui.fonts(|f| f.layout_job(job));
            //     *g = new_g;
            // }
            logs_show.push(log);
        }

        logs_show
    }
}

impl Drop for LogcatWorker {
    fn drop(&mut self) {
        self.close();
    }
}
