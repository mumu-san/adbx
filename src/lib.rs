use std::io::Read;
use std::process::Stdio;
use std::sync::{ Arc, Mutex };
use std::thread;

/// Pipe streams are blocking, we need separate threads to monitor them without blocking the primary thread.
pub fn child_stream_to_vec<R>(mut stream: R) -> Arc<Mutex<Vec<u8>>> where R: Read + Send + 'static {
    let out = Arc::new(Mutex::new(Vec::with_capacity(512 * 1024)));
    let vec = out.clone();
    thread::Builder
        ::new()
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
                        if buf[0] == b'\n' {
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
    let len = string.len();
    let point13 = len / 3;
    let point23 = point13 * 2;
    let mut chars = [0u8, 0u8, 0u8];
    // let char1 equels the average of the first third of the string
    chars[0] = (string[..point13].chars().fold(0, |acc, x| acc + (x as u32)) / point13 as u32) as u8;
    // let char2 equels the average of the second third of the string
    chars[1] = (string[point13..point23].chars().fold(0, |acc, x| acc + (x as u32)) / point13 as u32) as u8;
    // let char3 equels the average of the third third of the string
    chars[2] = (string[point23..].chars().fold(0, |acc, x| acc + (x as u32)) / (len - point23) as u32) as u8;

    egui::Color32::from_rgb(chars[0] + chars[0], chars[1] + chars[1], chars[2] + chars[2])
}

pub fn get_adb_devices(adb_path: &str) -> Vec<String> {
    let path = adb_path.trim().trim_matches('"');
    let output = std::process::Command::new(path).arg("devices").output();
    if output.is_err() {
        println!("adb devices error: {}, {}", path, output.err().unwrap());
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
    let path = adb_path.trim().trim_matches('"');
    let output = std::process::Command
        ::new(path)
        .arg("-s")
        .arg(device)
        .arg("logcat")
        .stdout(Stdio::piped())
        .spawn();
    if output.is_err() {
        println!("adb logcat error: {}, {}", path, output.err().unwrap());
        return None;
    }
    Some(child_stream_to_vec(output.unwrap().stdout.take().expect("!stdout")))
}

pub fn clear_adb_logcat(adb_path: &str, device: &str) {
    let path = adb_path.trim().trim_matches('"');
    let output = std::process::Command
        ::new(path)
        .arg("-s")
        .arg(device)
        .arg("logcat")
        .arg("-c")
        .output();
    if output.is_err() {
        println!("adb logcat error: {}, {}", path, output.err().unwrap());
        return;
    }
}
