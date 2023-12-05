use std::sync::Arc;

pub struct RawLog {
    pub origin: String,

    pub info: Option<FormatedItem>,
}
pub struct ColorLog {
    pub raw: RawLog,

    pub gallery: Arc<egui::Galley>,

    pub bottom: f32,
}

pub struct FormatedItem {
    pub date: std::ops::Range<usize>,
    pub time: std::ops::Range<usize>,
    pub pid: std::ops::Range<usize>,
    pub tid: std::ops::Range<usize>,
    pub level: std::ops::Range<usize>,
    pub tag: std::ops::Range<usize>,
    pub message: std::ops::Range<usize>,
}
