use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use eframe::egui;
use eframe::App;

fn main() {
    let mut native_options = eframe::NativeOptions::default();
    native_options.viewport.inner_size = Some(egui::Vec2::new(1280.0, 720.0));
    let ret = eframe::run_native(
        "ADBX",
        native_options,
        Box::new(|cc| Box::new(MyEguiApp::new(cc))),
    );
    match ret {
        Ok(_) => {}
        Err(err) => {
            println!("Error: {}", err);
        }
    }
}

struct MyEguiApp {
    adb_path: String,
    adb_devices: Vec<String>,
    adb_logcat_out_handle: Option<Arc<Mutex<Vec<u8>>>>,
    adb_logcat_out_buffer: String,
    selected_device: usize,
    time_point: SystemTime,
    frame_count: usize,
    last_fps: usize,

    layouter: Box<dyn Fn(&egui::Ui, &str, f32) -> Arc<egui::Galley>>,

    fliter_buffer: String,
    fliter: Option<String>,
}

impl MyEguiApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.

        // set adb path
        let adb_path = "D:/Temp/platform-tools/adb.exe";
        // get adb devices
        let adb_devices = adbx::get_adb_devices(adb_path);

        let layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
            let mut layout_job: egui::text::LayoutJob = egui::text::LayoutJob::default();
            layout_job.wrap.max_width = wrap_width;

            // for each line

            for line in string.lines() {
                if line.len() < 18 {
                    continue;
                }
                // if first word is not number, then it is not a time stamp
                if !line.starts_with(|c: char| c.is_digit(10)) {
                    layout_job.append(
                        line,
                        0.0,
                        egui::TextFormat {
                            font_id: egui::FontId::new(14.0, egui::FontFamily::Monospace),
                            color: egui::Color32::GRAY,
                            ..Default::default()
                        },
                    );
                    layout_job.append("\n", 0.0, egui::TextFormat::default());
                    continue;
                }

                let l_date = &line[0..5];
                // split line by space or double space for 6 parts
                let others = &line[6..];
                let mut split_indexes = Vec::new();
                let mut last_is_space = true;
                for (i, c) in others.chars().enumerate() {
                    if c != ' ' {
                        if last_is_space {
                            split_indexes.push(i);
                        }
                        last_is_space = false;
                    } else {
                        if !last_is_space {
                            split_indexes.push(i);
                        }
                        last_is_space = true;
                    }
                    if split_indexes.len() == 11 {
                        break;
                    }
                }
                if split_indexes.len() < 11 {
                    layout_job.append(
                        line,
                        0.0,
                        egui::TextFormat {
                            font_id: egui::FontId::new(14.0, egui::FontFamily::Monospace),
                            color: egui::Color32::GRAY,
                            ..Default::default()
                        },
                    );
                    layout_job.append("\n", 0.0, egui::TextFormat::default());
                    continue;
                }

                let l_time = &others[split_indexes[0]..split_indexes[1]];
                let l_pid = &others[split_indexes[2]..split_indexes[3]];
                // ensure pid length is 5
                let l_pid = format!("{: <width$}", l_pid, width = 5);
                let l_pid = l_pid.as_str();
                let l_tid = &others[split_indexes[4]..split_indexes[5]];
                // ensure tid length is 5
                let l_tid = format!("{: <width$}", l_tid, width = 5);
                let l_tid = l_tid.as_str();

                let l_level = &others[split_indexes[6]..split_indexes[7]];
                let l_tag = &others[split_indexes[8]..split_indexes[9]];
                // limit tag length and extend it with spaces
                let limit = 20;
                let l_tag = format!("{: <width$}", l_tag, width = limit);
                let l_tag = l_tag.as_str();

                let l_message = &others[split_indexes[10]..];

                let data_color = egui::Color32::from_rgb(0x66, 0x99, 0x99);
                let time_color = egui::Color32::from_rgb(0x33, 0x99, 0x99);
                let pid_color = egui::Color32::from_rgb(0xcc, 0xcc, 0xcc);
                let tid_color = egui::Color32::from_rgb(0x99, 0xcc, 0x99);

                let tag_color = adbx::get_color_from_string(l_tag);
                let color = match l_level.chars().nth(0).unwrap() {
                    'V' => egui::Color32::LIGHT_GRAY,
                    'D' => egui::Color32::LIGHT_BLUE,
                    'I' => egui::Color32::WHITE,
                    'W' => egui::Color32::YELLOW,
                    'E' => egui::Color32::LIGHT_RED,
                    _ => egui::Color32::LIGHT_GRAY,
                };

                layout_job.append(
                    l_date,
                    0.0,
                    egui::TextFormat {
                        font_id: egui::FontId::new(14.0, egui::FontFamily::Monospace),
                        color: data_color,
                        ..Default::default()
                    },
                );
                layout_job.append("  ", 0.0, egui::TextFormat::default());

                layout_job.append(
                    l_time,
                    0.0,
                    egui::TextFormat {
                        font_id: egui::FontId::new(14.0, egui::FontFamily::Monospace),
                        color: time_color,
                        ..Default::default()
                    },
                );
                layout_job.append("  ", 0.0, egui::TextFormat::default());

                layout_job.append(
                    l_pid,
                    0.0,
                    egui::TextFormat {
                        font_id: egui::FontId::new(14.0, egui::FontFamily::Monospace),
                        color: pid_color,
                        ..Default::default()
                    },
                );
                layout_job.append("  ", 0.0, egui::TextFormat::default());

                layout_job.append(
                    l_tid,
                    0.0,
                    egui::TextFormat {
                        font_id: egui::FontId::new(14.0, egui::FontFamily::Monospace),
                        color: tid_color,
                        ..Default::default()
                    },
                );
                layout_job.append("  ", 0.0, egui::TextFormat::default());

                layout_job.append(
                    l_tag,
                    0.0,
                    egui::TextFormat {
                        font_id: egui::FontId::new(14.0, egui::FontFamily::Monospace),
                        italics: true,
                        color: tag_color,
                        ..Default::default()
                    },
                );
                layout_job.append("  ", 0.0, egui::TextFormat::default());

                layout_job.append(
                    l_message,
                    0.0,
                    egui::TextFormat {
                        font_id: egui::FontId::new(14.0, egui::FontFamily::Monospace),
                        color,
                        ..Default::default()
                    },
                );

                layout_job.append("\n", 0.0, egui::TextFormat::default());
            }

            ui.fonts(|f| f.layout_job(layout_job))
        };

        MyEguiApp {
            adb_path: adb_path.to_string(),
            adb_devices,
            adb_logcat_out_handle: None,
            adb_logcat_out_buffer: String::with_capacity(1024 * 1024 * 4),
            selected_device: 0,
            time_point: SystemTime::now(),
            frame_count: 0,
            last_fps: 0,

            layouter: Box::new(layouter),

            fliter_buffer: String::new(),
            fliter: None,
        }
    }

    pub fn check_adb_devices(&mut self) -> bool {
        let last_device = self
            .adb_devices
            .get(self.selected_device)
            .unwrap_or(&String::new())
            .clone();

        self.adb_devices = adbx::get_adb_devices(&self.adb_path);

        let new_device = self.adb_devices.get(self.selected_device);

        if last_device.is_empty() && new_device.is_none() {
            println!("device not found");
            return false;
        }
        if &last_device != new_device.unwrap() {
            println!("device changed");
            self.adb_logcat_out_buffer.clear();
            self.selected_device = 0;
            self.adb_logcat_out_handle = None;
            return false;
        }
        true
    }
}

impl App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.frame_count += 1;
        let time_elapsed = SystemTime::now().duration_since(self.time_point).unwrap();
        if time_elapsed.as_secs_f32() > 1.0 {
            self.last_fps = self.frame_count;
            self.time_point = SystemTime::now();
            self.frame_count = 0;
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Grid::new("adb_grid")
                .striped(true)
                .min_col_width(10.0)
                .max_col_width(500.0)
                .spacing(egui::vec2(10.0, 10.0))
                .show(ui, |ui| {
                    // ui.heading("Hello World!");
                    // show the current frame every second
                    ui.label(format!("Frame: {}", self.last_fps));
                    ui.label("adb path:");
                    ui.text_edit_singleline(&mut self.adb_path);
                });

            egui::Grid::new("device_grid")
                .striped(true)
                .spacing(egui::vec2(10.0, 10.0))
                .show(ui, |ui| {
                    // show a button
                    if ui.button("Refresh Devices").clicked() {
                        println!("> {} devices", &self.adb_path);
                        self.check_adb_devices();
                    }

                    if self.adb_devices.len() <= 0 {
                        ui.label("No device found");
                        return;
                    }

                    // draw a combo box to select device
                    let devices = self.adb_devices.clone();
                    for (i, device) in devices.iter().enumerate() {
                        if ui
                            .selectable_label(self.selected_device == i, device)
                            .clicked()
                        {
                            self.check_adb_devices();
                            if i != self.selected_device {
                                self.selected_device = i;
                                self.adb_logcat_out_buffer.clear();
                                self.adb_logcat_out_handle = None;
                            }
                        }
                    }
                });

            egui::Grid::new("logcat_grid")
                .min_col_width(10.0)
                .max_col_width(100.0)
                .striped(true)
                .spacing(egui::vec2(10.0, 10.0))
                .show(ui, |ui| {
                    // show a button to call adb logcat
                    if ui.button("Show Logcat").clicked() {
                        if !self.check_adb_devices() {
                            return;
                        }
                        // if out is none, call adb logcat
                        if self.adb_logcat_out_handle.is_none() {
                            // run adb logcat
                            self.adb_logcat_out_handle = adbx::get_adb_logcat_handler(
                                &self.adb_path,
                                &self.adb_devices[self.selected_device],
                            );
                            // print command
                            println!("> {} -s {} logcat", &self.adb_path, &self.adb_devices[self.selected_device]);
                        }
                    }
                    // call logcat -c
                    if ui.button("Clear Logcat").clicked() {
                        if !self.check_adb_devices() {
                            return;
                        }
                        // run adb logcat
                        adbx::clear_adb_logcat(
                            &self.adb_path,
                            &self.adb_devices[self.selected_device],
                        );
                        self.adb_logcat_out_buffer.clear();
                        // print command
                        println!(
                            "> {} -s {} logcat -c",
                            &self.adb_path, &self.adb_devices[self.selected_device]
                        );
                    }
                    // show a text edit to fliter logcat
                    // ui.text_edit_singleline(&mut self.fliter_buffer);
                    // if ui.button("Fliter").clicked() {
                    //     if self.fliter_buffer.is_empty() {
                    //         self.fliter = None;
                    //     } else {
                    //         self.fliter = Some(self.fliter_buffer.clone());
                    //     }
                    // }
                });

            if self.adb_logcat_out_handle.is_some() {
                // show the output of adb logcat
                let out = self.adb_logcat_out_handle.as_ref().unwrap();
                let mut vec = out.lock().expect("!lock");
                //println!("vec.len(): {}", vec.len());
                let text = unsafe { String::from_utf8_unchecked(vec.clone()) };
                self.adb_logcat_out_buffer.push_str(&text);
                //println!("buffer len: {}", self.adb_logcat_out_buffer.len());
                vec.clear();
            }

            egui::ScrollArea::vertical()
                .auto_shrink([true, true])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.adb_logcat_out_buffer)
                            .desired_width(ui.available_width())
                            .layouter(&mut self.layouter),
                    );
                });
        });
    }
}
