use crate::log::*;
pub struct MyHighlighter {}

impl MyHighlighter {
    pub fn new() -> Self {
        MyHighlighter {}
    }
    pub fn highlighter(&mut self, string: &str) -> (egui::text::LayoutJob, RawLog) {
        let mut layout_job = egui::text::LayoutJob::default();
        let mut log = RawLog {
            origin: string.to_string(),
            info: None,
        };
        //println!("string len: {}", string.len());
        let line = string;

        // if first word is not number, then it is not a time stamp
        if line.len() < 18 || !line.starts_with(|c: char| c.is_digit(10)) {
            layout_job.append(line, 0.0, egui::TextFormat {
                font_id: egui::FontId::new(14.0, egui::FontFamily::Monospace),
                color: egui::Color32::GRAY,
                ..Default::default()
            });
            //layout_job.append("\n", 0.0, egui::TextFormat::default());
            return (layout_job, log);
        }

        let l_date = &line[0..5];
        // split line by space or double space for 6 parts
        let others = &line[6..];
        let mut split_indexes = Vec::with_capacity(11);
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
            layout_job.append(line, 0.0, egui::TextFormat {
                font_id: egui::FontId::new(14.0, egui::FontFamily::Monospace),
                color: egui::Color32::GRAY,
                ..Default::default()
            });
            //layout_job.append("\n", 0.0, egui::TextFormat::default());
            return (layout_job, log);
        }

        log.info = Some(FormatedItem {
            date: 0..5,
            time: split_indexes[0]..split_indexes[1],
            pid: split_indexes[2]..split_indexes[3],
            tid: split_indexes[4]..split_indexes[5],
            level: split_indexes[6]..split_indexes[7],
            tag: split_indexes[8]..split_indexes[9],
            message: split_indexes[10]..line.len(),
        });

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
        let l_tag = &others[split_indexes[8]..split_indexes[9]].trim_end_matches(":");
        // limit tag length and extend it with spaces
        let limit = 20;
        let l_tag = format!("{: <width$}", l_tag, width = limit);
        let l_tag = l_tag.as_str();

        let l_message = &others[split_indexes[10]..];

        let data_color = egui::Color32::from_rgb(0x66, 0x99, 0x99);
        let time_color = egui::Color32::from_rgb(0x33, 0x99, 0x99);
        let pid_color = egui::Color32::from_rgb(0xcc, 0xcc, 0xcc);
        let tid_color = egui::Color32::from_rgb(0x99, 0xcc, 0x99);

        let tag_color = get_color_from_string(l_tag);
        let color = match l_level.chars().nth(0).unwrap() {
            'V' => egui::Color32::LIGHT_GRAY,
            'D' => egui::Color32::LIGHT_BLUE,
            'I' => egui::Color32::WHITE,
            'W' => egui::Color32::YELLOW,
            'E' => egui::Color32::LIGHT_RED,
            _ => egui::Color32::LIGHT_GRAY,
        };

        layout_job.append(l_date, 0.0, egui::TextFormat {
            font_id: egui::FontId::new(14.0, egui::FontFamily::Monospace),
            color: data_color,
            ..Default::default()
        });
        layout_job.append("  ", 0.0, egui::TextFormat::default());

        layout_job.append(l_time, 0.0, egui::TextFormat {
            font_id: egui::FontId::new(14.0, egui::FontFamily::Monospace),
            color: time_color,
            ..Default::default()
        });
        layout_job.append("  ", 0.0, egui::TextFormat::default());

        layout_job.append(l_pid, 0.0, egui::TextFormat {
            font_id: egui::FontId::new(14.0, egui::FontFamily::Monospace),
            color: pid_color,
            ..Default::default()
        });
        layout_job.append("  ", 0.0, egui::TextFormat::default());

        layout_job.append(l_tid, 0.0, egui::TextFormat {
            font_id: egui::FontId::new(14.0, egui::FontFamily::Monospace),
            color: tid_color,
            ..Default::default()
        });
        layout_job.append("  ", 0.0, egui::TextFormat::default());

        layout_job.append(l_tag, 0.0, egui::TextFormat {
            font_id: egui::FontId::new(14.0, egui::FontFamily::Monospace),
            italics: true,
            color: tag_color,
            ..Default::default()
        });
        layout_job.append("  ", 0.0, egui::TextFormat::default());

        layout_job.append(l_message, 0.0, egui::TextFormat {
            font_id: egui::FontId::new(14.0, egui::FontFamily::Monospace),
            color,
            ..Default::default()
        });

        //layout_job.append("\n", 0.0, egui::TextFormat::default());
        (layout_job, log)
    }
}

fn get_color_from_string(string: &str) -> egui::Color32 {
    let len = string.len();
    let point13 = len / 3;
    let point23 = point13 * 2;
    let mut chars = [0u8, 0u8, 0u8];
    // let char1 equels the average of the first third of the string
    chars[0] = (string[..point13].chars().fold(0, |acc, x| acc + (x as u32)) /
        (point13 as u32)) as u8;
    // let char2 equels the average of the second third of the string
    chars[1] = (string[point13..point23].chars().fold(0, |acc, x| acc + (x as u32)) /
        (point13 as u32)) as u8;
    // let char3 equels the average of the third third of the string
    chars[2] = (string[point23..].chars().fold(0, |acc, x| acc + (x as u32)) /
        ((len - point23) as u32)) as u8;

    egui::Color32::from_rgb(chars[0] + chars[0], chars[1] + chars[1], chars[2] + chars[2])
}
