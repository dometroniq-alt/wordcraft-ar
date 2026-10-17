//! Home tab thumbnails: the clipboard, character formatting, lists and alignment, finding and
//! selecting. Mostly before/after pairs: the left page as the text is, the right page after
//! the button, with what changed in the accent colour.

use egui::{Align2, Color32, Rect, Shape, Stroke, pos2, vec2};

use super::Entry;
use super::draw::{Dir, Draw};

pub const ENTRIES: &[Entry] = &[
    Entry::new("edit.paste", "Puts the text you cut or copied in at the caret.", |d| d.hm_paste()),
    Entry::new("edit.cut", "Removes the selected text and keeps it on the clipboard for pasting.", |d| d.hm_clip(true)),
    Entry::new("edit.copy", "Keeps a copy of the selected text on the clipboard; the text stays where it is.", |d| d.hm_clip(false)),
    Entry::new("edit.formatPainter", "Copies the look of the selected text (font, size, colour) and paints it onto the text you click.", |d| {
        d.hm_format_painter()
    }),
    Entry::new("format.bold", "Makes the selected text bold: thicker, darker strokes.", |d| {
        d.before_after(|d, a| d.text_lines(a, true, &[52.0, 46.0, 30.0], 12.0, d.grey()), |d, a| d.hm_thick_lines(a))
    }),
    Entry::new("format.italic", "Slants the selected text to the right, as in italic type.", |d| {
        d.before_after(|d, a| d.hm_words_row(a, false), |d, a| d.hm_words_row(a, true))
    }),
    Entry::new("format.underline", "Draws a line under the selected text.", |d| {
        d.before_after(|d, a| d.hm_words_row(a, false), |d, a| d.hm_words_marked(a, 5.0))
    }),
    Entry::new("format.strikethrough", "Draws a line through the middle of the selected text.", |d| {
        d.before_after(|d, a| d.hm_words_row(a, false), |d, a| d.hm_words_marked(a, 0.0))
    }),
    Entry::new("format.subscript", "Makes the selected text small and lowers it below the line, as in H₂O.", |d| d.hm_script(false)),
    Entry::new("format.superscript", "Makes the selected text small and raises it above the line, as in x².", |d| d.hm_script(true)),
    Entry::new("format.outline", "Gives the selected text a decorative look, such as an outline or a shadow.", |d| d.hm_text_effects()),
    Entry::new("format.highlight", "Paints a colour behind the selected text, like a highlighter pen.", |d| {
        d.before_after(
            |d, a| d.paragraph(a, Dir::Rtl, 5, d.grey()),
            |d, a| {
                let b = d.band(a, 1..=2);
                d.highlight(Rect::from_min_max(pos2(b.min.x + 14.0, b.min.y), b.max));
                d.paragraph(a, Dir::Rtl, 5, d.grey());
            },
        )
    }),
    Entry::new("format.color", "Changes the colour of the selected text itself.", |d| {
        d.before_after(
            |d, a| d.paragraph(a, Dir::Rtl, 5, d.grey()),
            |d, a| {
                d.paragraph(a, Dir::Rtl, 5, d.grey());
                let f = a.width();
                d.text_lines(Rect::from_min_max(pos2(a.min.x, a.min.y + 10.0), a.max), true, &[f * 0.94, f * 0.98], 10.0, d.red());
            },
        )
    }),
    Entry::new("para.bullets", "Starts a list with a dot (•) in front of every paragraph.", |d| {
        d.before_after(|d, a| d.paragraph(a, Dir::Rtl, 4, d.grey()), |d, a| d.list(a, Dir::Rtl, &["•", "•", "•", "•"], d.accent()))
    }),
    Entry::new("para.numbering", "Starts a numbered list: 1. 2. 3. in front of every paragraph, renumbered as you add items.", |d| {
        d.before_after(|d, a| d.paragraph(a, Dir::Rtl, 4, d.grey()), |d, a| d.list(a, Dir::Rtl, &["1.", "2.", "3.", "4."], d.accent()))
    }),
    Entry::new("para.multilevel", "Starts a list with levels: 1. for main items and a. b. for the items under them, stepped in.", |d| {
        d.before_after(|d, a| d.paragraph(a, Dir::Rtl, 4, d.grey()), |d, a| d.hm_multilevel(a))
    }),
    Entry::new("para.outdent", "Moves the paragraph one step back out towards the margin.", |d| {
        d.before_after(|d, a| d.hm_indented(a, 12.0), |d, a| d.hm_indented(a, 0.0));
        d.arrow_right(d.at(112.0, 86.0), d.at(132.0, 86.0));
    }),
    Entry::new("para.indent", "Moves the paragraph one step in, away from the margin.", |d| {
        d.before_after(|d, a| d.hm_indented(a, 0.0), |d, a| d.hm_indented(a, 12.0));
        d.arrow_left(d.at(160.0, 86.0), d.at(140.0, 86.0));
    }),
    Entry::new("para.sort", "Puts the selected paragraphs in alphabetical or numerical order.", |d| {
        d.before_after(|d, a| d.list(a, Dir::Rtl, &["ج", "أ", "د", "ب"], d.grey()), |d, a| d.list(a, Dir::Rtl, &["أ", "ب", "ج", "د"], d.accent()))
    }),
    Entry::new("view.marks", "Shows the hidden marks: ¶ at the end of every paragraph and a dot for every space.", |d| d.hm_marks()),
    Entry::new("para.alignLeft", "Lines the paragraph up along the left margin; the right edge stays ragged.", |d| d.hm_aligned(0)),
    Entry::new("para.alignCenter", "Centres every line of the paragraph between the margins.", |d| d.hm_aligned(1)),
    Entry::new("para.alignRight", "Lines the paragraph up along the right margin; the left edge stays ragged.", |d| d.hm_aligned(2)),
    Entry::new("para.justify", "Stretches every line to reach both margins, so the paragraph has straight edges.", |d| d.hm_aligned(3)),
    Entry::new("para.lineSpacing", "Sets how far apart the lines of the paragraph are, and the gap before and after it.", |d| {
        d.before_after(
            |d, a| d.text_lines(a, true, &[52.0, 48.0, 50.0, 46.0, 30.0], 8.0, d.grey()),
            |d, a| {
                d.text_lines(a, true, &[42.0, 38.0, 40.0, 36.0, 24.0], 14.0, d.grey());
                d.measure_v(pos2(a.min.x + 3.0, a.min.y + 4.0), pos2(a.min.x + 3.0, a.min.y + 60.0));
            },
        )
    }),
    Entry::new("para.shading", "Fills the background of the whole paragraph with a colour.", |d| {
        d.before_after(
            |d, a| d.paragraph(a, Dir::Rtl, 6, d.grey()),
            |d, a| {
                d.p.rect_filled(d.band(a, 1..=3), 1.0, tint(d.accent()));
                d.paragraph(a, Dir::Rtl, 6, d.grey());
            },
        )
    }),
    Entry::new("para.borders", "Draws lines around the paragraph: a box, or only the top or bottom edge.", |d| {
        d.before_after(
            |d, a| d.paragraph(a, Dir::Rtl, 6, d.grey()),
            |d, a| {
                d.paragraph(a, Dir::Rtl, 6, d.grey());
                d.p.rect_stroke(d.band(a, 1..=3), 0.0, Stroke::new(1.5, d.accent()), egui::StrokeKind::Outside);
            },
        )
    }),
    Entry::new("edit.find", "Finds every place a word or phrase occurs and marks each one.", |d| d.hm_find()),
    Entry::new("ui.dialog:replace", "Finds a word or phrase and swaps every occurrence for other text.", |d| d.hm_replace()),
    Entry::new("select.all", "Selects the whole document at once.", |d| {
        let a = d.page_center();
        d.selection(d.band(a, 0..=6));
        d.paragraph(a, Dir::Rtl, 7, d.ink());
    }),
    Entry::new("tools.dictate", "Types what you say into the microphone, at the caret, as you speak.", |d| d.hm_dictate()),
    Entry::new("review.spelling", "Checks the spelling and grammar and stops at each mistake with suggestions.", |d| d.hm_spelling()),
];

/// A light tint of the accent for shading and translucent marks.
fn tint(c: Color32) -> Color32 {
    c.linear_multiply(0.3)
}

impl Draw<'_> {
    /// A clipboard (a board with a clip) whose centre is `c`, holding a few accent lines.
    fn hm_clipboard(&self, c: egui::Pos2) {
        let board = Rect::from_center_size(c, vec2(30.0, 38.0));
        self.p.rect_filled(board, 2.0, self.t.hover);
        self.p.rect_stroke(board, 2.0, self.stroke(self.t.border_strong), egui::StrokeKind::Inside);
        let clip = Rect::from_center_size(pos2(c.x, board.min.y), vec2(12.0, 6.0));
        self.p.rect_filled(clip, 1.0, self.t.text);
        self.text_lines(board.shrink2(vec2(5.0, 6.0)), true, &[20.0, 16.0, 18.0], 8.0, self.t.accent);
    }

    fn hm_paste(&self) {
        self.hm_clipboard(self.at(34.0, 50.0));
        self.arrow_right(self.at(58.0, 50.0), self.at(80.0, 50.0));
        let page = self.page(90.0, 8.0, 70.0, 88.0);
        self.text_lines(page, true, &[50.0, 44.0], 10.0, self.grey());
        self.caret(pos2(page.max.x - 2.0, page.min.y + 20.0), 10.0);
        self.text_lines(Rect::from_min_max(pos2(page.min.x, page.min.y + 24.0), page.max), true, &[46.0, 52.0, 40.0], 10.0, self.accent());
        self.text_lines(Rect::from_min_max(pos2(page.min.x, page.min.y + 54.0), page.max), true, &[48.0, 30.0], 10.0, self.grey());
    }

    /// Cut (`remove`) or Copy: a selection on the left page, the clipboard holding it on the
    /// right; after a cut the lines are gone from the page.
    fn hm_clip(&self, remove: bool) {
        let before = self.page(6.0, 8.0, 66.0, 88.0);
        self.selection(self.band(before, 2..=3));
        self.paragraph(before, Dir::Rtl, 7, self.grey());
        self.arrow_right(self.at(76.0, 52.0), self.at(92.0, 52.0));
        let after = self.page(96.0, 8.0, 66.0, 88.0);
        if remove {
            self.text_lines(after, true, &[50.0, 47.0, 46.0, 48.0, 28.0], 10.0, self.grey());
        } else {
            self.paragraph(after, Dir::Rtl, 7, self.grey());
        }
        self.hm_clipboard(self.at(150.0, 78.0));
    }

    fn hm_format_painter(&self) {
        self.before_after(
            |d, a| {
                d.heading_then_text(a, Dir::Rtl, 2, d.accent());
                d.paragraph(Rect::from_min_max(pos2(a.min.x, a.min.y + 40.0), a.max), Dir::Rtl, 3, d.grey());
            },
            |d, a| {
                d.heading_then_text(a, Dir::Rtl, 2, d.accent());
                d.heading_then_text(Rect::from_min_max(pos2(a.min.x, a.min.y + 40.0), a.max), Dir::Rtl, 2, d.accent());
            },
        );
        // A brush: a handle and a wide tip, over the arrow.
        self.line(self.at(80.0, 28.0), self.at(94.0, 42.0), self.t.text, 2.5);
        self.line(self.at(94.0, 42.0), self.at(100.0, 48.0), self.accent(), 5.0);
    }

    /// Thick accent lines (bold).
    fn hm_thick_lines(&self, a: Rect) {
        for (i, w) in [52.0, 46.0, 30.0].iter().enumerate() {
            let y = a.min.y + 4.0 + i as f32 * 12.0;
            self.line(pos2(a.max.x - w, y), pos2(a.max.x, y), self.accent(), 3.0);
        }
    }

    /// Three rows of "words" (blocks); slanted parallelograms when `italic`.
    fn hm_words_row(&self, a: Rect, italic: bool) {
        let rows: [&[f32]; 3] = [&[14.0, 10.0, 18.0], &[12.0, 16.0, 10.0], &[16.0, 12.0]];
        let c = if italic { self.accent() } else { self.grey() };
        for (r, ws) in rows.iter().enumerate() {
            let y = a.min.y + 6.0 + r as f32 * 14.0;
            let mut x = a.max.x;
            for w in ws.iter() {
                let (x0, x1) = (x - w, x);
                if italic {
                    let pts = vec![pos2(x0 + 2.0, y - 3.0), pos2(x1 + 2.0, y - 3.0), pos2(x1 - 2.0, y + 3.0), pos2(x0 - 2.0, y + 3.0)];
                    self.p.add(Shape::convex_polygon(pts, c, Stroke::NONE));
                } else {
                    self.word(x0, y, *w, c);
                }
                x = x0 - 5.0;
            }
        }
    }

    /// The same words with an accent line `dy` below their middle (underline at 5, strike at 0).
    fn hm_words_marked(&self, a: Rect, dy: f32) {
        self.hm_words_row(a, false);
        let rows: [f32; 3] = [42.0, 38.0, 28.0];
        for (r, w) in rows.iter().enumerate() {
            let y = a.min.y + 6.0 + r as f32 * 14.0 + dy;
            self.line(pos2(a.max.x - w, y), pos2(a.max.x, y), self.accent(), 1.5);
        }
    }

    /// `X2` on one baseline, then the 2 small and raised (`up`) or lowered.
    fn hm_script(&self, up: bool) {
        self.before_after(
            |d, a| {
                let c = a.center();
                d.glyph(c + vec2(-6.0, 0.0), "X", 24.0, d.ink());
                d.glyph(c + vec2(10.0, 0.0), "2", 24.0, d.grey());
            },
            |d, a| {
                let c = a.center();
                d.glyph(c + vec2(-6.0, 0.0), "X", 24.0, d.ink());
                let dy = if up { -9.0 } else { 7.0 };
                d.glyph(c + vec2(9.0, dy), "2", 12.0, d.accent());
            },
        );
    }

    /// An outlined letter beside a letter with a shadow.
    fn hm_text_effects(&self) {
        self.before_after(
            |d, a| d.glyph(a.center(), "A", 30.0, d.ink()),
            |d, a| {
                let c = a.center();
                d.glyph(c + vec2(2.0, 2.0), "A", 30.0, d.grey());
                for off in [vec2(-1.2, 0.0), vec2(1.2, 0.0), vec2(0.0, -1.2), vec2(0.0, 1.2)] {
                    d.glyph(c + off, "A", 30.0, d.accent());
                }
                d.glyph(c, "A", 30.0, Color32::WHITE);
            },
        );
    }

    /// A two-level list: 1. then a. b. stepped in, then 2.
    fn hm_multilevel(&self, a: Rect) {
        let marks = ["1.", "a.", "b.", "2."];
        let levels = [0.0, 1.0, 1.0, 0.0];
        for (i, m) in marks.iter().enumerate() {
            let y = a.min.y + 5.0 + i as f32 * 12.0;
            let x = a.max.x - levels[i] * 10.0;
            self.label(pos2(x, y), m, 7.0, self.accent(), Align2::RIGHT_CENTER);
            self.line(pos2(a.min.x, y), pos2(x - 14.0, y), self.grey(), 1.5);
        }
    }

    /// A paragraph stepped in from the right margin by `indent`, between two ordinary ones.
    fn hm_indented(&self, a: Rect, indent: f32) {
        self.text_lines(a, true, &[50.0, 30.0], 10.0, self.grey());
        let mid = Rect::from_min_max(pos2(a.min.x, a.min.y + 22.0), pos2(a.max.x - indent, a.max.y));
        let c = if indent > 0.0 { self.accent() } else { self.grey() };
        self.text_lines(mid, true, &[mid.width(), mid.width() * 0.9, mid.width() * 0.5], 10.0, c);
        self.text_lines(Rect::from_min_max(pos2(a.min.x, a.min.y + 54.0), a.max), true, &[48.0, 26.0], 10.0, self.grey());
    }

    /// Pilcrows at the end of every paragraph and dots between the words.
    fn hm_marks(&self) {
        let a = self.page_center();
        let rows: [&[f32]; 4] = [&[14.0, 10.0, 18.0], &[12.0, 16.0], &[16.0, 12.0, 10.0], &[12.0, 18.0]];
        for (r, ws) in rows.iter().enumerate() {
            let y = a.min.y + 8.0 + r as f32 * 16.0;
            let mut x = a.max.x;
            for (i, w) in ws.iter().enumerate() {
                self.word(x - w, y, *w, self.grey());
                x -= w + 6.0;
                if i + 1 < ws.len() {
                    self.p.circle_filled(pos2(x + 3.0, y), 1.2, self.accent());
                }
            }
            self.label(pos2(x + 1.0, y), "¶", 9.0, self.accent(), Align2::RIGHT_CENTER);
        }
    }

    /// One page whose lines sit on the left (0), centre (1), right (2) or both margins (3), with
    /// an accent guide along the straight edge(s).
    fn hm_aligned(&self, how: u8) {
        let a = self.page_center();
        let full = a.width();
        let widths = [1.0, 0.8, 0.95, 0.7, 0.9, 0.6];
        for (i, f) in widths.iter().enumerate() {
            let y = a.min.y + 6.0 + i as f32 * 12.0;
            let w = if how == 3 { full } else { full * f };
            let x0 = match how {
                0 | 3 => a.min.x,
                1 => a.center().x - w / 2.0,
                _ => a.max.x - w,
            };
            self.line(pos2(x0, y), pos2(x0 + w, y), self.grey(), 1.5);
        }
        let (top, bottom) = (a.min.y, a.min.y + 66.0);
        if how == 0 || how == 3 {
            self.line(pos2(a.min.x - 3.0, top), pos2(a.min.x - 3.0, bottom), self.accent(), 1.5);
        }
        if how == 2 || how == 3 {
            self.line(pos2(a.max.x + 3.0, top), pos2(a.max.x + 3.0, bottom), self.accent(), 1.5);
        }
        if how == 1 {
            let x = a.center().x;
            let mut y = top;
            while y < bottom {
                self.line(pos2(x, y), pos2(x, y + 3.0), self.accent(), 1.5);
                y += 6.0;
            }
        }
    }

    fn hm_find(&self) {
        let a = self.page_rl();
        for (row, dx, w) in [(1usize, 20.0, 16.0), (3, 42.0, 16.0), (5, 8.0, 16.0)] {
            let y = a.min.y + 4.0 + row as f32 * 10.0;
            self.highlight(Rect::from_min_max(pos2(a.max.x - dx - w, y - 4.0), pos2(a.max.x - dx, y + 4.0)));
            self.line(pos2(a.max.x - dx - w, y), pos2(a.max.x - dx, y), self.ink(), 1.5);
        }
        self.magnifier(self.at(140.0, 70.0));
    }

    fn hm_replace(&self) {
        let inner = self.dialog("Replace", 2);
        // The two fields hold a word each; an arrow between them says "becomes".
        self.word(inner.min.x + 40.0, inner.min.y + 5.0, 22.0, self.grey());
        self.word(inner.min.x + 40.0, inner.min.y + 19.0, 26.0, self.accent());
        self.arrow_down(pos2(inner.max.x - 14.0, inner.min.y + 2.0), pos2(inner.max.x - 14.0, inner.min.y + 20.0));
        self.button(pos2(inner.center().x, inner.max.y - 10.0), "Replace All", false);
    }

    /// A red wave under a word, and a tick for the text that is fine.
    fn hm_spelling(&self) {
        let a = self.page_rl();
        let y = a.min.y + 24.0;
        let (x0, x1) = (a.max.x - 44.0, a.max.x - 22.0);
        self.p.rect_filled(Rect::from_min_max(pos2(x0, y - 4.0), pos2(x1, y + 4.0)), 0.0, Color32::WHITE);
        self.line(pos2(x0, y), pos2(x1, y), self.ink(), 1.5);
        let pts: Vec<egui::Pos2> = (0..=10).map(|i| pos2(x0 + i as f32 * 2.2, y + 4.0 + if i % 2 == 0 { 0.0 } else { 2.0 })).collect();
        self.p.add(Shape::line(pts, Stroke::new(1.2, self.red())));
        self.check(self.at(140.0, 74.0));
    }

    /// A microphone (capsule on a stand) with sound arcs, and the words landing on the page.
    fn hm_dictate(&self) {
        let c = self.at(34.0, 40.0);
        let cap = Rect::from_center_size(c, vec2(14.0, 26.0));
        self.p.rect_filled(cap, 7.0, self.accent());
        // The holder arc and the stand.
        let pts: Vec<egui::Pos2> = (0..=12)
            .map(|i| {
                let ang = i as f32 / 12.0 * std::f32::consts::PI;
                c + vec2(11.0 * ang.cos(), 4.0 + 11.0 * ang.sin())
            })
            .collect();
        self.p.add(Shape::line(pts, Stroke::new(2.0, self.ink())));
        self.line(c + vec2(0.0, 15.0), c + vec2(0.0, 24.0), self.ink(), 2.0);
        self.line(c + vec2(-8.0, 24.0), c + vec2(8.0, 24.0), self.ink(), 2.0);
        // Sound arcs to the right of the capsule.
        for r in [16.0, 22.0] {
            let pts: Vec<egui::Pos2> = (0..=8)
                .map(|i| {
                    let ang = -0.5 + i as f32 / 8.0;
                    c + vec2(r * ang.cos(), r * ang.sin())
                })
                .collect();
            self.p.add(Shape::line(pts, Stroke::new(1.5, self.grey())));
        }
        self.arrow_right(self.at(66.0, 50.0), self.at(84.0, 50.0));
        let a = self.page(90.0, 8.0, 70.0, 88.0);
        self.text_lines(a, true, &[50.0, 46.0], 10.0, self.grey());
        self.text_lines(Rect::from_min_max(pos2(a.min.x, a.min.y + 22.0), a.max), true, &[48.0, 52.0, 30.0], 10.0, self.accent());
        self.caret(pos2(a.max.x - 32.0, a.min.y + 39.0), 10.0);
    }
}
