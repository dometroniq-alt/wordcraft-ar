//! Review tab thumbnails: proofing, reading aloud, accessibility, language, comments, tracked
//! changes, comparing documents, protection and ink.

use egui::{Align2, Color32, Rect, Shape, Stroke, pos2, vec2};

use super::Entry;
use super::draw::{Dir, Draw};

pub const ENTRIES: &[Entry] = &[
    Entry::new("review.thesaurus", "Suggests words with the same meaning as the selected word so you can swap it.", |d| d.rev_thesaurus()),
    Entry::new("ui.dialog:wordCount", "Counts the pages, words, characters, paragraphs and lines of the document or the selection.", |d| {
        d.rev_word_count()
    }),
    Entry::new("review.readAloud", "Reads the text out loud from the caret, marking each word as it is spoken.", |d| d.rev_read_aloud()),
    Entry::new(
        "file.accessibility",
        "Checks the document for things a screen reader cannot handle, such as pictures without a description, and lists them.",
        |d| d.rev_accessibility(),
    ),
    Entry::new("review.translate", "Translates the selected text, or the whole document, into another language.", |d| d.rev_translate()),
    Entry::new("review.language", "Sets the language of the selected text so its spelling is checked with the right dictionary.", |d| {
        d.rev_language()
    }),
    Entry::new("review.deleteComment", "Removes the selected comment balloon from the margin.", |d| d.rev_delete_comment()),
    Entry::new("review.previousComment", "Jumps to the comment before this one in the document.", |d| d.rev_comment_nav(false)),
    Entry::new("review.nextComment", "Jumps to the next comment in the document.", |d| d.rev_comment_nav(true)),
    Entry::new("view.commentsPane", "Shows every comment of the document in a pane beside the page.", |d| d.rev_comments_pane()),
    Entry::new(
        "review.trackChanges",
        "Records every change you make: added text is underlined, deleted text is struck through, so others can see what changed.",
        |d| d.rev_track_changes(),
    ),
    Entry::new("review.markup", "Chooses how much of the tracked changes and comments is shown on the page: all, simple or none.", |d| {
        d.rev_markup()
    }),
    Entry::new("review.changes", "Opens a list of every tracked change and comment, with who made it and when.", |d| d.rev_reviewing_pane()),
    Entry::new("review.accept", "Keeps the selected change: the added text stays, the deleted text goes, and the marks disappear.", |d| {
        d.rev_decide(true)
    }),
    Entry::new("review.reject", "Undoes the selected change: the added text goes, the deleted text comes back, and the marks disappear.", |d| {
        d.rev_decide(false)
    }),
    Entry::new("review.previousChange", "Jumps to the tracked change before this one in the document.", |d| d.rev_change_nav(false)),
    Entry::new("review.nextChange", "Jumps to the next tracked change in the document.", |d| d.rev_change_nav(true)),
    Entry::new("review.compare", "Lays two versions of a document side by side and marks every difference between them.", |d| d.rev_compare()),
    Entry::new("review.blockAuthors", "Locks the selected part of the document so other authors cannot change it while you work on it.", |d| {
        d.rev_block_authors()
    }),
    Entry::new("review.restrict", "Limits what others may do in the document: read only, comments only or tracked changes only.", |d| {
        d.rev_restrict()
    }),
    Entry::new("review.hideInk", "Hides the pen and highlighter strokes drawn over the document without deleting them.", |d| d.rev_hide_ink()),
];

impl Draw<'_> {
    /// A comment balloon in `r` (a rounded box with two text lines), in accent when `hot`.
    fn rev_balloon(&self, r: Rect, hot: bool) {
        let c = if hot { self.accent() } else { self.t.border_strong };
        self.p.rect_filled(r, 4.0, if hot { self.accent().linear_multiply(0.15) } else { Color32::WHITE });
        self.p.rect_stroke(r, 4.0, Stroke::new(if hot { 1.5 } else { 1.0 }, c), egui::StrokeKind::Inside);
        self.text_lines(r.shrink(4.0), false, &[r.width() - 12.0, (r.width() - 8.0) * 0.6], 6.0, self.grey());
    }

    /// An inserted word: green with an underline.
    fn rev_inserted(&self, x: f32, y: f32, w: f32) {
        self.word(x, y, w, self.green());
        self.line(pos2(x, y + 4.5), pos2(x + w, y + 4.5), self.green(), 1.0);
    }

    /// A deleted word: red with a line through it.
    fn rev_deleted(&self, x: f32, y: f32, w: f32) {
        self.word(x, y, w, self.red().linear_multiply(0.6));
        self.line(pos2(x - 1.0, y), pos2(x + w + 1.0, y), self.red(), 1.5);
    }

    /// Ordinary lines in `area` with an inserted word on row 1 and a deleted word on row 3, and
    /// a change bar in the margin beside each.
    fn rev_marked_paragraph(&self, area: Rect) {
        let w = area.width();
        self.text_lines(area, false, &[w, 0.0, w * 0.96, 0.0, w * 0.92, w * 0.5], 10.0, self.grey());
        let y1 = area.min.y + 4.0 + 10.0;
        self.line(pos2(area.min.x, y1), pos2(area.min.x + w * 0.3, y1), self.grey(), 1.5);
        self.rev_inserted(area.min.x + w * 0.34, y1, w * 0.26);
        self.line(pos2(area.min.x + w * 0.64, y1), pos2(area.max.x, y1), self.grey(), 1.5);
        let y3 = area.min.y + 4.0 + 30.0;
        self.line(pos2(area.min.x, y3), pos2(area.min.x + w * 0.2, y3), self.grey(), 1.5);
        self.rev_deleted(area.min.x + w * 0.24, y3, w * 0.3);
        self.line(pos2(area.min.x + w * 0.58, y3), pos2(area.min.x + w * 0.9, y3), self.grey(), 1.5);
        for y in [y1, y3] {
            self.line(pos2(area.min.x - 5.0, y - 4.0), pos2(area.min.x - 5.0, y + 4.0), self.accent(), 1.5);
        }
    }

    /// A padlock centred at `c`: a filled body with a shackle arc.
    fn rev_lock(&self, c: egui::Pos2) {
        let body = Rect::from_center_size(c + vec2(0.0, 6.0), vec2(20.0, 15.0));
        self.p.rect_filled(body, 2.0, self.accent());
        let r = 6.0;
        let top = c + vec2(0.0, -4.0);
        let n = 12;
        let pts: Vec<_> = (0..=n)
            .map(|i| std::f32::consts::PI + std::f32::consts::PI * i as f32 / n as f32)
            .map(|ang| top + vec2(ang.cos(), ang.sin()) * r)
            .collect();
        self.p.add(Shape::line(pts, Stroke::new(2.5, self.accent())));
        self.line(top + vec2(-r, 0.0), top + vec2(-r, 4.0), self.accent(), 2.5);
        self.line(top + vec2(r, 0.0), top + vec2(r, 4.0), self.accent(), 2.5);
        self.p.circle_filled(c + vec2(0.0, 6.0), 2.0, Color32::WHITE);
    }

    /// A selected word on the page, and a list of words that mean the same.
    fn rev_thesaurus(&self) {
        let a = self.page(8.0, 8.0, 76.0, 88.0);
        self.paragraph(a, Dir::Ltr, 7, self.grey());
        let y = a.min.y + 4.0 + 20.0;
        let w = Rect::from_min_size(pos2(a.min.x + 12.0, y - 4.0), vec2(18.0, 8.0));
        self.selection(w);
        self.label(w.center(), "big", 7.0, self.ink(), Align2::CENTER_CENTER);
        let pane = self.pane(true, 78.0, 0);
        for (i, s) in ["large", "huge", "vast", "major", "sizable"].iter().enumerate() {
            let yy = pane.min.y + 22.0 + i as f32 * 13.0;
            self.label(pos2(pane.min.x + 8.0, yy), s, 7.0, if i == 0 { self.accent() } else { self.ink() }, Align2::LEFT_CENTER);
        }
        self.arrow_right(pos2(w.max.x + 2.0, y), pos2(pane.min.x - 2.0, y));
    }

    /// The Word Count dialog: a few counts in a box.
    fn rev_word_count(&self) {
        let inner = self.dialog("Word Count", 0);
        for (i, (k, v)) in [("Pages", "3"), ("Words", "1 240"), ("Characters", "6 802"), ("Paragraphs", "38")].iter().enumerate() {
            let y = inner.min.y + 4.0 + i as f32 * 11.0;
            self.label(pos2(inner.min.x, y), k, 6.5, self.grey(), Align2::LEFT_CENTER);
            self.label(pos2(inner.max.x, y), v, 6.5, self.ink(), Align2::RIGHT_CENTER);
        }
    }

    /// A page being read: the spoken word highlighted, a speaker beside it.
    fn rev_read_aloud(&self) {
        let a = self.page(8.0, 8.0, 84.0, 88.0);
        self.paragraph(a, Dir::Ltr, 7, self.grey());
        let y = a.min.y + 4.0 + 20.0;
        let w = Rect::from_min_size(pos2(a.min.x + 20.0, y - 4.0), vec2(22.0, 8.0));
        self.highlight(w);
        self.line(pos2(w.min.x + 1.0, y), pos2(w.max.x - 1.0, y), self.ink(), 1.5);
        self.speaker(self.at(132.0, 46.0));
    }

    /// A page with a picture that lacks a description, and the pane that lists such issues.
    fn rev_accessibility(&self) {
        let a = self.page(8.0, 8.0, 80.0, 88.0);
        self.text_lines(a, false, &[60.0, 56.0], 7.0, self.grey());
        let pic = Rect::from_min_size(a.min + vec2(0.0, 16.0), vec2(36.0, 26.0));
        self.picture(pic);
        self.warning(pic.right_top() + vec2(-4.0, 4.0));
        self.text_lines(Rect::from_min_max(pos2(a.min.x, a.min.y + 48.0), a.max), false, &[64.0, 60.0, 40.0], 7.0, self.grey());
        let pane = self.pane(true, 72.0, 3);
        self.p.circle_filled(pos2(pane.min.x + 8.0, pane.min.y + 20.0), 2.5, Color32::from_rgb(220, 150, 20));
        self.p.circle_filled(pos2(pane.min.x + 8.0, pane.min.y + 31.0), 2.5, Color32::from_rgb(220, 150, 20));
        self.p.circle_filled(pos2(pane.min.x + 8.0, pane.min.y + 42.0), 2.5, self.green());
    }

    /// Selected text with a language tag.
    fn rev_language(&self) {
        let a = self.page_center();
        self.selection(self.band(a, 1..=2));
        self.paragraph(a, Dir::Ltr, 5, self.grey());
        let c = pos2(a.center().x, a.max.y - 12.0);
        self.p.rect_filled(Rect::from_center_size(c, vec2(44.0, 16.0)), 3.0, self.accent());
        self.label(c, "ع / EN", 8.0, Color32::WHITE, Align2::CENTER_CENTER);
    }

    /// The page, the margin and a balloon joined to the commented words.
    fn rev_page_with_balloon(&self, hot: bool) -> Rect {
        let a = self.page(8.0, 8.0, 84.0, 88.0);
        self.paragraph(a, Dir::Ltr, 7, self.grey());
        let y = a.min.y + 4.0 + 20.0;
        let w = Rect::from_min_size(pos2(a.min.x + 16.0, y - 4.0), vec2(24.0, 8.0));
        self.highlight(w);
        self.line(pos2(w.min.x + 1.0, y), pos2(w.max.x - 1.0, y), self.ink(), 1.5);
        let b = Rect::from_min_size(self.at(104.0, 22.0), vec2(64.0, 26.0));
        self.line(w.right_center(), b.left_center(), self.accent(), 1.0);
        self.rev_balloon(b, hot);
        b
    }

    fn rev_delete_comment(&self) {
        let b = self.rev_page_with_balloon(true);
        self.cross(b.right_bottom() + vec2(-4.0, 2.0));
    }

    /// Three balloons in the margin; the arrow jumps to the previous or the next one.
    fn rev_comment_nav(&self, next: bool) {
        let a = self.page(8.0, 8.0, 84.0, 88.0);
        self.paragraph(a, Dir::Ltr, 7, self.grey());
        let ys = [12.0, 42.0, 72.0];
        let hot = if next { 2 } else { 0 };
        for (i, y) in ys.iter().enumerate() {
            let b = Rect::from_min_size(self.at(120.0, *y), vec2(48.0, 20.0));
            self.rev_balloon(b, i == hot);
            self.line(pos2(a.min.x + 20.0 + i as f32 * 10.0, a.min.y + 4.0 + i as f32 * 20.0 + 10.0), b.left_center(), self.t.border, 1.0);
        }
        let x = self.at(108.0, 0.0).x;
        if next {
            self.arrow_down(pos2(x, self.at(0.0, 52.0).y), pos2(x, self.at(0.0, 82.0).y));
        } else {
            self.arrow_up(pos2(x, self.at(0.0, 52.0).y), pos2(x, self.at(0.0, 22.0).y));
        }
    }

    /// The page and a pane full of balloons.
    fn rev_comments_pane(&self) {
        let a = self.page(8.0, 8.0, 76.0, 88.0);
        self.paragraph(a, Dir::Ltr, 7, self.grey());
        self.highlight(Rect::from_min_size(pos2(a.min.x + 10.0, a.min.y + 20.0), vec2(22.0, 8.0)));
        self.highlight(Rect::from_min_size(pos2(a.min.x + 24.0, a.min.y + 50.0), vec2(18.0, 8.0)));
        let pane = self.pane(true, 80.0, 0);
        for i in 0..3 {
            let b = Rect::from_min_size(pos2(pane.min.x + 5.0, pane.min.y + 17.0 + i as f32 * 26.0), vec2(pane.width() - 10.0, 22.0));
            self.rev_balloon(b, i == 0);
        }
    }

    /// Typed text becomes marked text: additions underlined in green, deletions struck in red.
    fn rev_track_changes(&self) {
        self.before_after(|d, a| d.paragraph(a, Dir::Ltr, 6, d.grey()), |d, a| d.rev_marked_paragraph(a));
    }

    /// A marked page with the markup choice above it.
    fn rev_markup(&self) {
        let a = self.page_center();
        self.rev_marked_paragraph(Rect::from_min_max(pos2(a.min.x + 4.0, a.min.y + 16.0), a.max));
        let c = pos2(a.center().x, a.min.y + 4.0);
        self.chip(c, "All Markup", 60.0);
        let t = c + vec2(26.0, 0.0);
        self.p.add(Shape::convex_polygon(vec![t + vec2(-3.0, -2.0), t + vec2(3.0, -2.0), t + vec2(0.0, 2.0)], self.ink(), Stroke::NONE));
    }

    /// The list of changes on the left, the marked page on the right.
    fn rev_reviewing_pane(&self) {
        let pane = self.pane(false, 72.0, 5);
        for (i, c) in [self.green(), self.red(), self.accent(), self.green(), self.red()].iter().enumerate() {
            self.p.circle_filled(pos2(pane.min.x + 8.0, pane.min.y + 20.0 + i as f32 * 11.0), 2.5, *c);
        }
        let a = self.page(84.0, 8.0, 84.0, 88.0);
        self.rev_marked_paragraph(Rect::from_min_max(pos2(a.min.x + 6.0, a.min.y), a.max));
    }

    /// A marked change becomes plain text: kept (`accept`) or undone.
    fn rev_decide(&self, accept: bool) {
        self.before_after(
            |d, a| d.rev_marked_paragraph(Rect::from_min_max(pos2(a.min.x + 6.0, a.min.y), a.max)),
            |d, a| {
                let w = a.width();
                d.paragraph(a, Dir::Ltr, 6, d.grey());
                let y1 = a.min.y + 4.0 + 10.0;
                d.p.rect_filled(Rect::from_min_size(pos2(a.min.x + w * 0.34, y1 - 3.0), vec2(w * 0.26, 6.0)), 1.0, Color32::WHITE);
                if accept {
                    d.word(a.min.x + w * 0.34, y1, w * 0.26, d.ink());
                }
                let y3 = a.min.y + 4.0 + 30.0;
                d.p.rect_filled(Rect::from_min_size(pos2(a.min.x + w * 0.24, y3 - 3.0), vec2(w * 0.3, 6.0)), 1.0, Color32::WHITE);
                if !accept {
                    d.word(a.min.x + w * 0.24, y3, w * 0.3, d.ink());
                }
                let c = pos2(a.max.x - 4.0, a.max.y - 4.0);
                if accept {
                    d.check(c);
                } else {
                    d.cross(c);
                }
            },
        );
    }

    /// Two marked changes on the page; the arrow jumps to the previous or the next one.
    fn rev_change_nav(&self, next: bool) {
        let a = self.page_center();
        let area = Rect::from_min_max(pos2(a.min.x + 6.0, a.min.y), a.max);
        self.rev_marked_paragraph(area);
        let w = area.width();
        let y1 = area.min.y + 4.0 + 10.0;
        let y3 = area.min.y + 4.0 + 30.0;
        let target = if next {
            Rect::from_min_size(pos2(area.min.x + w * 0.24, y3 - 4.0), vec2(w * 0.3, 8.0))
        } else {
            Rect::from_min_size(pos2(area.min.x + w * 0.34, y1 - 4.0), vec2(w * 0.26, 8.0))
        };
        self.p.rect_stroke(target.expand(2.0), 2.0, Stroke::new(1.5, self.accent()), egui::StrokeKind::Outside);
        let x = area.max.x + 6.0;
        if next {
            self.arrow_down(pos2(x, y1), pos2(x, y3));
        } else {
            self.arrow_up(pos2(x, y3), pos2(x, y1));
        }
    }

    /// Two versions side by side, the lines that differ marked.
    fn rev_compare(&self) {
        let left = self.page(8.0, 8.0, 70.0, 88.0);
        let right = self.page(98.0, 8.0, 70.0, 88.0);
        for a in [left, right] {
            self.paragraph(a, Dir::Ltr, 7, self.grey());
        }
        for (a, row, c) in [(left, 2.0, self.red()), (right, 2.0, self.green()), (right, 5.0, self.green())] {
            let y = a.min.y + 4.0 + row * 10.0;
            self.line(pos2(a.min.x, y), pos2(a.max.x * 0.4 + a.min.x * 0.6, y), c, 2.0);
            self.line(pos2(a.min.x - 5.0, y - 4.0), pos2(a.min.x - 5.0, y + 4.0), c, 2.0);
        }
        self.label(self.at(88.0, 52.0), "≠", 12.0, self.accent(), Align2::CENTER_CENTER);
    }

    /// Left-to-right text under a big "A" becomes right-to-left text under a big "ع".
    fn rev_translate(&self) {
        self.before_after(
            |d, a| {
                d.glyph(pos2(a.center().x, a.min.y + 16.0), "A", 24.0, d.grey());
                d.paragraph(Rect::from_min_max(pos2(a.min.x, a.min.y + 32.0), a.max), Dir::Ltr, 4, d.grey());
            },
            |d, a| {
                d.glyph(pos2(a.center().x, a.min.y + 16.0), "ع", 24.0, d.accent());
                d.paragraph(Rect::from_min_max(pos2(a.min.x, a.min.y + 32.0), a.max), Dir::Rtl, 4, d.accent());
            },
        );
    }

    /// A selected block of the page with a small lock on it; the rest stays free.
    fn rev_block_authors(&self) {
        let a = self.page_center();
        let band = self.band(a, 2..=4);
        self.selection(band);
        self.paragraph(a, Dir::Ltr, 7, self.grey());
        self.p.rect_stroke(band, 1.0, Stroke::new(1.5, self.accent()), egui::StrokeKind::Outside);
        self.rev_lock(pos2(band.max.x + 12.0, band.center().y - 4.0));
    }

    /// A page under a lock.
    fn rev_restrict(&self) {
        let a = self.page_lr();
        self.rev_lock(pos2(a.center().x, a.center().y + 2.0));
    }

    /// A pen stroke over the text fades out.
    fn rev_hide_ink(&self) {
        let stroke = |d: &Draw<'_>, a: Rect, c: Color32| {
            let n = 24;
            let pts: Vec<_> = (0..=n)
                .map(|i| {
                    let t = i as f32 / n as f32;
                    pos2(a.min.x + 4.0 + t * (a.width() - 8.0), a.min.y + 30.0 + (t * 12.0).sin() * 6.0)
                })
                .collect();
            d.p.add(Shape::line(pts, Stroke::new(3.0, c)));
        };
        self.before_after(
            |d, a| {
                d.paragraph(a, Dir::Ltr, 6, d.grey());
                stroke(d, a, d.accent());
            },
            |d, a| {
                d.paragraph(a, Dir::Ltr, 6, d.grey());
                stroke(d, a, d.accent().linear_multiply(0.2));
            },
        );
    }
}
