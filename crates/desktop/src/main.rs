use eframe::egui;
use rrsvp_core::reader::ReadingLoop;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([640.0, 172.0])
            .with_title("RRSVP Desktop"),
        ..Default::default()
    };
    eframe::run_native(
        "RRSVP",
        options,
        Box::new(|_cc| Ok(Box::new(RsvpApp::new()))),
    )
}

struct RsvpApp {
    reader: ReadingLoop,
    playing: bool,
    last_update: std::time::Instant,
}

impl RsvpApp {
    fn new() -> Self {
        let mut reader = ReadingLoop::new();
        reader.begin(0);
        Self {
            reader,
            playing: false,
            last_update: std::time::Instant::now(),
        }
    }

    fn now_ms(&self) -> u32 {
        self.last_update.elapsed().as_millis() as u32
    }
}

impl eframe::App for RsvpApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now_ms = self.now_ms();

        if self.playing {
            if self.reader.update(now_ms, true) {
                ctx.request_repaint();
            }
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(egui::Color32::BLACK))
            .show(ctx, |ui| {
                let available = ui.available_rect_before_wrap();

                // ── Header ────────────────────────────────────────────────────
                let pct = if self.reader.word_count() > 0 {
                    (self.reader.current_index() * 100) / self.reader.word_count()
                } else {
                    0
                };
                ui.horizontal(|ui| {
                    ui.colored_label(
                        egui::Color32::DARK_GRAY,
                        format!("{:>3}%  {} WPM", pct, self.reader.wpm()),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button(if self.playing { "⏸" } else { "▶" }).clicked() {
                            self.playing = !self.playing;
                            if self.playing {
                                self.reader.start(now_ms);
                            }
                        }
                    });
                });

                ui.separator();

                // ── Word display ──────────────────────────────────────────────
                let word = self.reader.current_word().to_string();
                let center_y = available.height() / 2.0 + 10.0;

                ui.allocate_ui_at_rect(
                    egui::Rect::from_min_size(
                        egui::pos2(0.0, center_y - 20.0),
                        egui::vec2(available.width(), 40.0),
                    ),
                    |ui| {
                        ui.with_layout(
                            egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                            |ui| {
                                if !word.is_empty() {
                                    // Highlight the ORP character in red.
                                    let orp = orp_index(&word);
                                    let mut job = egui::text::LayoutJob::default();
                                    for (i, ch) in word.chars().enumerate() {
                                        let color = if i == orp {
                                            egui::Color32::from_rgb(220, 80, 30)
                                        } else {
                                            egui::Color32::WHITE
                                        };
                                        job.append(
                                            &ch.to_string(),
                                            0.0,
                                            egui::TextFormat {
                                                font_id: egui::FontId::proportional(32.0),
                                                color,
                                                ..Default::default()
                                            },
                                        );
                                    }
                                    ui.label(job);
                                }
                            },
                        );
                    },
                );

                ui.separator();

                // ── Footer ────────────────────────────────────────────────────
                ui.horizontal(|ui| {
                    if ui.small_button("−WPM").clicked() {
                        self.reader.adjust_wpm(-1);
                    }
                    if ui.small_button("+WPM").clicked() {
                        self.reader.adjust_wpm(1);
                    }
                    if ui.small_button("⟨ Sentence").clicked() {
                        self.reader.rewind_sentence();
                    }
                    if ui.small_button("Word ⟩").clicked() {
                        self.reader.scrub(1);
                    }
                    ui.colored_label(
                        egui::Color32::DARK_GRAY,
                        format!(
                            "{} / {}",
                            self.reader.current_index() + 1,
                            self.reader.word_count()
                        ),
                    );
                });
            });

        // Request repaint at ~60fps when playing.
        if self.playing {
            ctx.request_repaint_after(std::time::Duration::from_millis(16));
        }
    }
}

/// ORP: index of the "optimal recognition point" character.
fn orp_index(word: &str) -> usize {
    let len = word.chars().count();
    if len == 0 { return 0; }
    ((len.saturating_sub(1)) / 4).min(len - 1)
}
