#[derive(Default)]
struct ScrollList<T> {
    rows: Vec<T>,
    pending_rows: Vec<T>,
    heights: Option<Vec<f32>>,
}

impl<T> Deref for ScrollList<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.rows
    }
}

impl<T> ScrollList<T>
where
    for<'a> &'a T: Widget,
{
    fn new(rows: Vec<T>) -> Self {
        Self {
            rows,
            pending_rows: vec![],
            heights: None,
        }
    }

    fn push(&mut self, row: T) {
        self.pending_rows.push(row);
    }

    fn mark_dirty(&mut self) {
        self.heights = None;
    }

    fn show(&mut self, ui: &mut Ui, area: ScrollArea) -> ScrollAreaOutput<()> {
        area.show_viewport(ui, |ui, viewport| {
            ui.vertical(|ui| match &mut self.heights {
                // re-render everything to recalculate heights
                None => {
                    self.rows.append(&mut self.pending_rows);

                    let spacing_y = ui.style().spacing.item_spacing.y;
                    let mut heights = Vec::with_capacity(self.rows.len());
                    let mut last = 0.0;
                    for message in &self.rows {
                        last += ui.add(message).rect.height() + spacing_y;
                        heights.push(last);
                    }
                    self.heights = Some(heights);
                }
                Some(heights) => {
                    let top = viewport.top();
                    let bottom = viewport.bottom();
                    let from = heights
                        .binary_search_by(|h| h.total_cmp(&top))
                        .unwrap_or_else(|i| i);
                    let to = heights
                        .binary_search_by(|h| h.total_cmp(&bottom))
                        .unwrap_or_else(|i| i);

                    if from != 0 {
                        ui.allocate_space(egui::vec2(0.0, heights[from - 1]));
                    }
                    for row in &self.rows[from..(to + 1).min(self.rows.len())] {
                        ui.add(row);
                    }
                    let spacing_y = ui.style().spacing.item_spacing.y;
                    let mut total = heights.last().copied().unwrap_or_default();

                    let to = to.min(self.rows.len().saturating_sub(1));
                    let remaining =
                        total - heights.get(to).copied().unwrap_or_default() - spacing_y;

                    if remaining > 0.0 {
                        ui.allocate_space(egui::vec2(0.0, remaining));
                    }

                    for row in self.pending_rows.drain(..) {
                        let r = ui.add(&row);
                        total += r.rect.height() + spacing_y;
                        heights.push(total);
                        self.rows.push(row);
                    }
                }
            });
        })
    }
}