mod log;
mod highlighter;
mod logcat_worker;

use std::time::SystemTime;

use eframe::egui;
use eframe::App;

use logcat_worker::LogcatWorker;

fn main() {
    let mut native_options = eframe::NativeOptions::default();
    native_options.viewport.inner_size = Some(egui::Vec2::new(1280.0, 720.0));
    native_options.follow_system_theme = false;
    let ret = eframe::run_native(
        "ADBX",
        native_options,
        Box::new(|cc| Box::new(MyEguiApp::new(cc)))
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
    selected_device: usize,
    time_point: SystemTime,
    frame_count: usize,
    last_fps: usize,
    frame_limit: usize,

    adb_logcat_worker: Option<LogcatWorker>,
    filter_buffer: String,

    demo: egui_demo_lib::DemoWindows,
}

impl MyEguiApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.

        // set adb path
        let buildin_adb_path = "platform-tools/adb.exe";
        // convert adb path to absolute path
        // let adb_path = match std::fs::canonicalize(buildin_adb_path) {
        //     Ok(path) => path.to_string_lossy().to_string(),
        //     Err(_) => buildin_adb_path.to_string(),
        // };
        let adb_path = buildin_adb_path.to_string();
        // get adb devices
        let adb_devices = adbx::get_adb_devices(adb_path.as_str());

        MyEguiApp {
            adb_path,
            adb_devices,
            selected_device: 0,
            time_point: SystemTime::now(),
            frame_count: 0,
            last_fps: 0,
            frame_limit: 60,
            adb_logcat_worker: None,
            filter_buffer: String::new(),

            demo: egui_demo_lib::DemoWindows::default(),
        }
    }

    pub fn check_adb_devices(&mut self) -> bool {
        let last_device = self.adb_devices
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
            self.selected_device = 0;
            self.adb_logcat_worker = None;
            return false;
        }
        true
    }
}

impl App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        //self.demo.ui(ctx);

        //return;

        let time_point = SystemTime::now();

        self.frame_count += 1;
        let time_elapsed = time_point.duration_since(self.time_point).unwrap();
        if time_elapsed.as_secs_f32() > 1.0 {
            self.last_fps = self.frame_count;
            self.time_point = SystemTime::now();
            self.frame_count = 0;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.frame_count % 120 == 0 {
                //self.check_adb_devices();
            }
            egui::Grid
                ::new("adb_grid")
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

            egui::Grid
                ::new("device_grid")
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
                        if ui.selectable_label(self.selected_device == i, device).clicked() {
                            self.check_adb_devices();
                            if i != self.selected_device {
                                self.selected_device = i;
                                self.adb_logcat_worker = None;
                            }
                        }
                    }
                });

            let mut scoll_to_bottom = false;

            egui::Grid
                ::new("logcat_grid")
                .min_col_width(10.0)
                .max_col_width(150.0)
                .striped(true)
                .spacing(egui::vec2(10.0, 10.0))
                .show(ui, |ui| {
                    // show a button to call adb logcat
                    if ui.button("Show Logcat").clicked() {
                        if !self.check_adb_devices() {
                            return;
                        }
                        // if out is none, call adb logcat
                        if self.adb_logcat_worker.is_none() {
                            // print command
                            println!(
                                "> {} -s {} logcat",
                                &self.adb_path,
                                &self.adb_devices[self.selected_device]
                            );
                            // run adb logcat
                            self.adb_logcat_worker = Some(
                                LogcatWorker::new(&self.adb_devices[self.selected_device])
                            );
                            self.adb_logcat_worker.as_mut().unwrap().connect(&self.adb_path);
                        }
                    }
                    // call logcat -c
                    if ui.button("Clear Logcat").clicked() {
                        if !self.check_adb_devices() || self.adb_logcat_worker.is_none() {
                            return;
                        }
                        println!(
                            "> {} -s {} logcat -c",
                            &self.adb_path,
                            &self.adb_devices[self.selected_device]
                        );
                        self.adb_logcat_worker.as_mut().unwrap().clear(&self.adb_path);
                    }
                    //show a text edit to fliter logcat
                    ui.text_edit_singleline(&mut self.filter_buffer);
                    if ui.button("Fliter").clicked() {
                        // option string if filter is empty
                        let fliter = if self.filter_buffer.is_empty() {
                            None
                        } else {
                            Some(self.filter_buffer.clone())
                        };
                        self.adb_logcat_worker.as_mut().unwrap().set_fliter(fliter);
                        println!("set fliter: {}", self.filter_buffer);
                    }
                    // show a button to scroll to bottom
                    scoll_to_bottom |= ui.button("Scroll Bottom").clicked();
                });

            ui.separator();

            if let Some(worker) = self.adb_logcat_worker.as_mut() {
                worker.show(ui, scoll_to_bottom);
            }

            ui.separator();
        });

        // if time is not up to 1/60 second, then wait
        let time_elapsed = SystemTime::now().duration_since(time_point).unwrap();
        let time_abundance = 1.0 / (self.frame_limit as f32) - time_elapsed.as_secs_f32();
        if time_abundance > 0.01 {
            //println!("time_used: {}, time_abundance: {}", time_elapsed.as_secs_f32(), time_abundance);
            //std::thread::sleep(std::time::Duration::from_secs_f32(time_abundance-0.01));
        }
        ctx.request_repaint();
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        println!("on_exit");
    }
}
