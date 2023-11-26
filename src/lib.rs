use std::io::Read;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::thread;

/// Pipe streams are blocking, we need separate threads to monitor them without blocking the primary thread.
pub fn child_stream_to_vec<R>(mut stream: R) -> Arc<Mutex<Vec<u8>>>
where
    R: Read + Send + 'static,
{
    let out = Arc::new(Mutex::new(Vec::with_capacity(512 * 1024)));
    let vec = out.clone();
    thread::Builder::new()
        .name("child_stream_to_vec".into())
        .spawn(move || {
            let mut vec_buf: Vec<u8> = Vec::with_capacity(256);
            loop {
                let mut buf = [0];
                match stream.read(&mut buf) {
                    Err(err) => {
                        println!("{}] Error reading from stream: {}", line!(), err);
                        break;
                    }
                    Ok(got) => {
                        if got == 0 {
                            break;
                        }
                        vec_buf.push(buf[0]);
                        if vec_buf.len() >= 256 {
                            let mut vec = vec.lock().expect("!lock");
                            vec.append(&mut vec_buf);
                            vec_buf.clear();
                        }
                    }
                }
            }
        })
        .expect("!thread");
    out
}

pub fn get_color_from_string(string: &str) -> egui::Color32 {
    let char1 = string.chars().nth(0).unwrap();
    let char2 = string.chars().nth(1).unwrap();
    let char3 = string.chars().nth(2).unwrap();
    egui::Color32::from_rgb(
        100u8 + char1 as u8,
        100u8 + char2 as u8,
        100u8 + char3 as u8,
    )
}

pub fn get_adb_devices(adb_path: &str) -> Vec<String> {
    let output = std::process::Command::new(adb_path).arg("devices").output();
    if output.is_err() {
        println!("adb devices error: {}, {}", adb_path, output.err().unwrap());
        return Vec::new();
    }
    let output = output.unwrap();
    // convert output to string
    let output = String::from_utf8_lossy(&output.stdout);
    // split output by new line
    let output = output.split('\n').collect::<Vec<&str>>();
    // add devices to adb_devices
    let mut adb_devices = Vec::new();
    for line in output {
        if line.starts_with("List of") {
            continue;
        }
        if line.len() < 1 {
            continue;
        }
        let split = line.split('\t').collect::<Vec<&str>>();
        if split.len() < 2 {
            continue;
        }
        adb_devices.push(split[0].to_string());
    }
    adb_devices
}

pub fn get_adb_logcat_handler(adb_path: &str, device: &str) -> Option<Arc<Mutex<Vec<u8>>>> {
    let output = std::process::Command::new(adb_path)
        .arg("-s")
        .arg(device)
        .arg("logcat")
        .stdout(Stdio::piped())
        .spawn();
    if output.is_err() {
        println!("adb logcat error: {}, {}", adb_path, output.err().unwrap());
        return None;
    }
    Some(child_stream_to_vec(
        output.unwrap().stdout.take().expect("!stdout"),
    ))
}

pub fn clear_adb_logcat(adb_path: &str, device: &str) {
    let output = std::process::Command::new(adb_path)
        .arg("-s")
        .arg(device)
        .arg("logcat")
        .arg("-c")
        .output();
    if output.is_err() {
        println!("adb logcat error: {}, {}", adb_path, output.err().unwrap());
        return;
    }
}
