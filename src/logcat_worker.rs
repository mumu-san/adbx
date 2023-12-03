use std::sync::{ Arc, Mutex };
use std::thread::JoinHandle;
use std::io::Read;

use crate::highlighter::MyHighlighter;

pub struct LogcatWorker {
    device_name: String,
    logcat_sender: Option<std::process::Child>,
    logcat_receiver: Option<JoinHandle<()>>,
    logcat_buffer: Option<Arc<Mutex<Vec<u8>>>>,
    highlighter: MyHighlighter,
    filter: Option<String>,
    logs: Vec<Arc<egui::Galley>>,
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
            let mut layout_job = self.highlighter.highlighter(string);
            layout_job.wrap.max_width = wrap_width;

            let g = ui.fonts(|f| f.layout_job(layout_job));
            //println!("g rows{:?}", g.rows.len());
            return g;
        };

        for line in string.lines() {
            let g = layouter(ui, line, ui.available_width());
            self.logs.push(g);
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, scoll_to_bottom: bool) {
        self.update(ui);

        let mut total_height = 0.0;
        let mut logs_show = Vec::new();
        for g in self.logs.iter() {
            if self.filter.is_some() {
                let filter = self.filter.as_ref().unwrap();
                let text = g.text();
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
            total_height += g.rect.height() + ui.style().spacing.item_spacing.y;
            logs_show.push(g.clone());
        }

        egui::ScrollArea
            ::vertical()
            .auto_shrink([false, false])
            .stick_to_bottom(true)
            .show_viewport(ui, |ui, viewport| {
                let top = viewport.top();
                let bottom = viewport.bottom();
                //println!("top: {}, bottom: {}", top, bottom);
                let mut from = 0;
                let mut to = 0;
                let mut height = 0.0;
                let mut y_min = ui.max_rect().top();
                let mut y_max = ui.max_rect().top();
                let spacing = ui.style().spacing.item_spacing.y;
                for (i, g) in logs_show.iter().enumerate() {
                    let h = g.rect.height() + spacing;
                    height += h;
                    if height < top {
                        continue;
                    }
                    if from == 0 && top != 0.0 {
                        from = i;
                        y_min += height - h;
                    }
                    if height > bottom {
                        to = i;
                        y_max += height;
                        break;
                    }
                }
                if to == 0 {
                    to = logs_show.len().saturating_sub(1);
                }
                ui.set_height(total_height - spacing);
                //println!("from: {}, to: {}", from, to);
                //println!("y_min: {}, y_max: {}, total_height: {}", y_min, y_max, total_height);
                let rect = egui::Rect::from_x_y_ranges(ui.max_rect().x_range(), y_min..=y_max);
                ui.allocate_ui_at_rect(rect, |ui| {
                    //ui.skip_ahead_auto_ids(from);
                    for (i, g) in logs_show.iter().enumerate() {
                        if i < from {
                            continue;
                        } else if i > to {
                            break;
                        }
                        let wt = egui::WidgetText::from(g.clone());
                        let label = egui::Label::new(wt);
                        ui.add(label);
                    }
                });
                if scoll_to_bottom {
                    let bottom_rect = egui::Rect::from_x_y_ranges(
                        ui.max_rect().x_range(),
                        0.0..=total_height
                    );
                    ui.scroll_to_rect(bottom_rect, Some(egui::Align::BOTTOM));
                }
            });
    }
}

impl Drop for LogcatWorker {
    fn drop(&mut self) {
        self.close();
    }
}
