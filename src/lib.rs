
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
