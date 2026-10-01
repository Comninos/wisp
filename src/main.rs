#![cfg_attr(windows, windows_subsystem = "windows")]

use std::sync::Arc;

use eframe::egui::{
    self, pos2, text::LayoutJob, vec2, Align, Align2, CentralPanel, Color32, CursorIcon, Event,
    FontData, FontDefinitions, FontFamily, FontId, Frame, Galley, Id, Key, Label, Layout,
    Modifiers, Panel, RichText, ScrollArea, Sense, Stroke, TextBuffer, TextEdit, Theme, Ui,
    ViewportBuilder, ViewportCommand,
};

const FONT: f32 = 16.0;
const FONTS: std::ops::RangeInclusive<f32> = 8.0..=48.0;
const STEP: f32 = 2.0;
const BAR: f32 = 12.0;
const HELP_FONT: f32 = BAR + 2.0;
const SIZE: [f32; 2] = [420.0, 560.0];
const START: Theme = Theme::Light;
const DIM: Color32 = Color32::GRAY;
const GAP: usize = 2;
const TYPEFACE: &[u8] = include_bytes!("../fonts/IBMPlexMono-Regular.ttf");
const ICON: &[u8] = include_bytes!("../icon/wisp.png");

const HELP: &[(&str, &str)] = &[
    ("Ctrl Shift C", "copy the whole pad"),
    ("Ctrl Shift Backspace", "clear the pad"),
    ("Ctrl + / Ctrl -", "font bigger / smaller"),
    ("Ctrl 0", "font reset"),
    ("Ctrl T", "light / dark"),
    ("Ctrl M", "minimise"),
    ("Ctrl Q", "close"),
    ("F1 / Esc", "back to the pad"),
];

// Wayland ignores window icons; there the compositor takes Icon= from wisp.desktop via the app id.
fn main() -> eframe::Result {
    let icon = eframe::icon_data::from_png_bytes(ICON).expect("icon/wisp.png is a valid PNG");
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title("wisp")
            .with_app_id("wisp")
            .with_icon(icon)
            .with_inner_size(SIZE)
            .with_always_on_top(),
        ..Default::default()
    };
    eframe::run_native(
        "wisp",
        options,
        Box::new(|cc| {
            cc.egui_ctx.options_mut(|o| o.zoom_with_keyboard = false);
            fonts(&cc.egui_ctx);
            themes(&cc.egui_ctx);
            Ok(Box::<Wisp>::default())
        }),
    )
}

// egui's bundled Hack stays behind as fallback for glyphs the typeface lacks.
fn fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    let data = Arc::new(FontData::from_static(TYPEFACE));
    fonts.font_data.insert("typeface".into(), data);
    let mono = fonts.families.entry(FontFamily::Monospace).or_default();
    mono.insert(0, "typeface".into());
    ctx.set_fonts(fonts);
}

fn themes(ctx: &egui::Context) {
    for (theme, fg, bg) in [
        (Theme::Light, Color32::BLACK, Color32::WHITE),
        (Theme::Dark, Color32::WHITE, Color32::BLACK),
    ] {
        let mut v = theme.default_visuals();
        v.panel_fill = bg;
        v.extreme_bg_color = bg;
        v.override_text_color = Some(fg);
        v.text_cursor.stroke = Stroke::new(2.0, fg);
        v.selection.bg_fill = fg.gamma_multiply(0.2);
        v.selection.stroke = Stroke::new(1.0, fg);
        ctx.set_visuals_of(theme, v);
    }
    ctx.set_theme(START);
}

struct Wisp {
    text: String,
    size: f32,
    help: bool,
}

impl Default for Wisp {
    fn default() -> Self {
        Self { text: String::new(), size: FONT, help: false }
    }
}

impl eframe::App for Wisp {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.keys(ui.ctx());
        Panel::bottom("bar").show(ui, |ui| self.bar(ui));
        CentralPanel::default().show(ui, |ui| {
            if self.help {
                let margin = self.gutter(ui).1;
                help(ui, margin);
            } else {
                self.pad(ui);
            }
        });
    }
}

impl Wisp {
    fn keys(&mut self, ctx: &egui::Context) {
        let none = Modifiers::NONE;
        let cmd = Modifiers::COMMAND;
        let shift = cmd | Modifiers::SHIFT;
        let (copy, theme, min, quit) = ctx.input_mut(|i| {
            // egui-winit turns Ctrl C into Event::Copy and drops the key, even with Shift held.
            let copy = i.modifiers.shift && i.events.iter().any(|e| matches!(e, Event::Copy));
            if copy {
                i.events.retain(|e| !matches!(e, Event::Copy));
            }
            if i.consume_key(shift, Key::Backspace) {
                self.text.clear();
            }
            if i.consume_key(cmd, Key::Plus) || i.consume_key(cmd, Key::Equals) {
                self.size = (self.size + STEP).min(*FONTS.end());
            }
            if i.consume_key(cmd, Key::Minus) {
                self.size = (self.size - STEP).max(*FONTS.start());
            }
            if i.consume_key(cmd, Key::Num0) {
                self.size = FONT;
            }
            if i.consume_key(none, Key::F1) || (self.help && i.consume_key(none, Key::Escape)) {
                self.help = !self.help;
            }
            (
                copy,
                i.consume_key(cmd, Key::T),
                i.consume_key(cmd, Key::M),
                i.consume_key(cmd, Key::Q),
            )
        });
        if copy {
            ctx.copy_text(self.text.clone());
        }
        if theme {
            ctx.set_theme(match ctx.theme() {
                Theme::Light => Theme::Dark,
                Theme::Dark => Theme::Light,
            });
        }
        if min {
            ctx.send_viewport_cmd(ViewportCommand::Minimized(true));
        }
        if quit {
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }
    }

    fn bar(&mut self, ui: &mut Ui) {
        let text = |s: &str| Label::new(RichText::new(s).font(FontId::monospace(BAR)));
        let n = self.text.chars().count();
        let count = format!("{n} char{}", if n == 1 { "" } else { "s" });
        ui.horizontal(|ui| {
            ui.add(text(&count).selectable(false));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let hint = ui.add(text("F1 help").sense(Sense::click()));
                if hint.on_hover_cursor(CursorIcon::PointingHand).clicked() {
                    self.help = !self.help;
                }
            });
        });
    }

    // Widths of the line numbers and of the whole gutter, at the pad's font size.
    fn gutter(&self, ui: &Ui) -> (f32, f32) {
        let digits = self.text.split('\n').count().to_string().len().max(2);
        let font = FontId::monospace(self.size);
        let glyph = ui.ctx().fonts_mut(|f| f.glyph_width(&font, '0'));
        (digits as f32 * glyph, (digits + GAP) as f32 * glyph)
    }

    fn pad(&mut self, ui: &mut Ui) {
        let font = FontId::monospace(self.size);
        let (numbers, inset) = self.gutter(ui);
        let height = ui.available_height();
        let fg = ui.visuals().text_color();

        ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
            ui.horizontal_top(|ui| {
                let right = ui.cursor().min.x + numbers;
                ui.add_space(inset);
                let id = Id::new("pad");
                if ui.ctx().memory(|m| m.focused().is_none()) {
                    ui.ctx().memory_mut(|m| m.request_focus(id));
                }
                let mut layouter = |ui: &Ui, text: &dyn TextBuffer, width: f32| {
                    wrap(ui, text.as_str(), &font, fg, width)
                };
                let out = TextEdit::multiline(&mut self.text)
                    .id(id)
                    .font(font.clone())
                    .layouter(&mut layouter)
                    .frame(Frame::NONE)
                    .margin(0.0)
                    .desired_width(ui.available_width() - inset)
                    .min_size(vec2(0.0, height))
                    .lock_focus(true)
                    .show(ui);

                let current = out.cursor_range.map(|c| {
                    let before = self.text.chars().take(c.primary.index.0);
                    before.filter(|&ch| ch == '\n').count()
                });
                let chars: Vec<char> = self.text.chars().collect();
                let mut line = 0;
                let mut index = 0;
                for row in &out.galley.rows {
                    if index == 0 || chars.get(index - 1) == Some(&'\n') {
                        let color = if current == Some(line) { fg } else { DIM };
                        let at = pos2(right, out.galley_pos.y + row.pos.y);
                        let n = (line + 1).to_string();
                        ui.painter().text(at, Align2::RIGHT_TOP, n, font.clone(), color);
                        line += 1;
                    }
                    index += row.glyphs.len() + usize::from(row.ends_with_newline);
                }
            });
        });
    }
}

// egui wraps a word whose trailing space crosses the edge. Pick the breaks here so only
// letters count, mark them as newlines in the layout copy, and let one space hang.
fn wrap(ui: &Ui, text: &str, font: &FontId, color: Color32, width: f32) -> Arc<Galley> {
    let layout = |text: String, width: f32| {
        ui.painter().layout_job(LayoutJob::simple(text, font.clone(), color, width))
    };
    let flat = layout(text.to_owned(), f32::INFINITY);
    let mut chars: Vec<char> = text.chars().collect();
    let mut index = 0;
    for row in &flat.rows {
        let (mut start, mut space) = (0.0, None);
        for (i, glyph) in row.glyphs.iter().enumerate() {
            if glyph.chr == ' ' {
                space = Some((index + i, glyph.max_x()));
            } else if glyph.max_x() - start > width
                && let Some((at, x)) = space.take()
            {
                chars[at] = '\n';
                start = x;
            }
        }
        index += row.glyphs.len() + usize::from(row.ends_with_newline);
    }
    let hang = ui.ctx().fonts_mut(|f| f.glyph_width(font, ' '));
    layout(chars.into_iter().collect(), width + hang)
}

fn help(ui: &mut Ui, margin: f32) {
    let font = FontId::monospace(HELP_FONT);
    let text = |s: &str| Label::new(RichText::new(s).font(font.clone())).selectable(false);
    let glyph = ui.ctx().fonts_mut(|f| f.glyph_width(&font, '0'));
    let longest = HELP.iter().map(|(key, _)| key.chars().count()).max().unwrap_or(0);
    let column = (longest + GAP) as f32 * glyph;
    ui.horizontal_top(|ui| {
        ui.add_space(margin);
        ui.vertical(|ui| {
            ui.set_max_width(ui.available_width() - margin);
            ui.spacing_mut().item_spacing.y = 8.0;
            for (key, action) in HELP {
                ui.horizontal_top(|ui| {
                    let used = ui.add(text(key)).rect.width();
                    ui.add_space(column - used);
                    ui.add(text(action).wrap());
                });
            }
        });
    });
}
