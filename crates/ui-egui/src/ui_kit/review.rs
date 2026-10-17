//! Review tab thumbnails: proofing, reading aloud, accessibility, language, comments, tracked
//! changes, comparing documents, protection and ink. Each one a little film: the synonyms list
//! fills, the spoken word moves along the line, the balloon is crossed out, the marks fade when
//! No Markup is chosen, the lock drops onto the page.

use egui::{Align2, Color32, Rect, Shape, Stroke, pos2, vec2};

use super::Entry;
use super::draw::{Dir, Draw};

pub const ENTRIES: &[Entry] = &[
    Entry::new(
        "review.thesaurus",
        "Shows words that mean the same as the selected word, so you can pick a better one and swap it in. Example: select «كبير» and choose «ضخم» or «واسع» from the list.",
        |d| d.rev_thesaurus(),
    ),
    Entry::new(
        "ui.dialog:wordCount",
        "Counts how many pages, words, letters, paragraphs and lines the document has, or the part you selected. Example: 3 pages, 1 240 words, 38 paragraphs.",
        |d| d.rev_word_count(),
    ),
    Entry::new(
        "review.readAloud",
        "Reads the text out loud from where the caret is, lighting up each word as it is spoken, so you can check it by ear. Example: start at a paragraph and hear a missing word.",
        |d| d.rev_read_aloud(),
    ),
    Entry::new(
        "file.accessibility",
        "Checks the document for things a blind reader's screen reader cannot handle, such as a picture with no description, and lists them to fix. Example: «Picture 1: add a description».",
        |d| d.rev_accessibility(),
    ),
    Entry::new(
        "review.translate",
        "Translates the selected text, or the whole document, into another language. Example: select an English paragraph and get it in Arabic.",
        |d| d.rev_translate(),
    ),
    Entry::new(
        "review.language",
        "Tells the spelling checker which language the selected text is in, so it uses the right dictionary. Example: mark an English sentence inside an Arabic letter as English.",
        |d| d.rev_language(),
    ),
    Entry::new(
        "review.deleteComment",
        "Removes the selected comment balloon from the margin; the text it pointed at stays. Example: delete the note «راجع التاريخ» once the date is fixed.",
        |d| d.rev_delete_comment(),
    ),
    Entry::new(
        "review.previousComment",
        "Jumps back to the comment before this one in the document. Example: from the third comment to the second.",
        |d| d.rev_comment_nav(false),
    ),
    Entry::new("review.nextComment", "Jumps forward to the next comment in the document. Example: from the first comment to the second.", |d| {
        d.rev_comment_nav(true)
    }),
    Entry::new(
        "view.commentsPane",
        "Shows every comment of the document in a list beside the page, so you can read and answer them in order. Example: three comments listed, the first one open.",
        |d| d.rev_comments_pane(),
    ),
    Entry::new(
        "review.trackChanges",
        "Records every change you make from now on: added text is underlined, deleted text is struck through, so others can see what changed and accept or reject it. Example: «الطبيب» deleted and «الضابط الطبيب» added, both marked.",
        |d| d.rev_track_changes(),
    ),
    Entry::new(
        "review.markup",
        "Chooses how much of the tracked changes and comments is shown on the page: all of them, a simple line in the margin, or none. Example: switch to No Markup to read the letter clean.",
        |d| d.rev_markup("All Markup", 1.0),
    ),
    Entry::new(
        "review.markup:all",
        "Shows every tracked change on the page: added text underlined, deleted text struck through, and the comment balloons. Example: the edited word appears twice, old and new.",
        |d| d.rev_markup("All Markup", 1.0),
    ),
    Entry::new(
        "review.markup:noMarkup",
        "Hides the change marks and shows the page as it will look with every change accepted; the changes are still recorded. Example: only the new word shows, with no underline.",
        |d| d.rev_markup("No Markup", 0.0),
    ),
    Entry::new(
        "review.changes",
        "Opens a list of every tracked change and comment, with who made it and when, so you can go through them one by one. Example: «added «الضابط» — أحمد — اليوم».",
        |d| d.rev_reviewing_pane(),
    ),
    Entry::new(
        "review.accept",
        "Keeps the selected change: the added text stays, the deleted text goes, and the marks on it disappear; the other changes stay marked. Example: accept «الضابط الطبيب» and it becomes ordinary text.",
        |d| d.rev_decide(true, false),
    ),
    Entry::new(
        "review.acceptAll",
        "Keeps every change in the document at once: all the added text stays, all the deleted text goes, and every mark disappears. Example: a letter with ten corrections becomes a clean final copy.",
        |d| d.rev_decide(true, true),
    ),
    Entry::new(
        "review.reject",
        "Undoes the selected change: the added text goes, the deleted text comes back, and the marks on it disappear; the other changes stay marked. Example: reject «الضابط الطبيب» and «الطبيب» is back.",
        |d| d.rev_decide(false, false),
    ),
    Entry::new(
        "review.rejectAll",
        "Undoes every change in the document at once, putting the text back as it was before the corrections. Example: a letter with ten corrections returns to its first version.",
        |d| d.rev_decide(false, true),
    ),
    Entry::new(
        "review.previousChange",
        "Jumps back to the tracked change before this one in the document. Example: from the deleted word to the added word above it.",
        |d| d.rev_change_nav(false),
    ),
    Entry::new(
        "review.nextChange",
        "Jumps forward to the next tracked change in the document. Example: from the added word to the deleted word below it.",
        |d| d.rev_change_nav(true),
    ),
    Entry::new(
        "review.compare",
        "Puts two versions of a document side by side and marks every line that differs between them. Example: compare yesterday's letter with today's and see the two changed lines.",
        |d| d.rev_compare(),
    ),
    Entry::new(
        "review.blockAuthors",
        "Locks the selected part of the document so other people cannot change it while you work on it. Example: lock the signature block so only you can edit it.",
        |d| d.rev_block_authors(),
    ),
    Entry::new(
        "review.restrict",
        "Limits what other people may do in the document: only read it, only add comments, or only make tracked changes. Example: send the letter so others can comment but not edit.",
        |d| d.rev_restrict(),
    ),
    Entry::new(
        "review.hideInk",
        "Hides the pen and highlighter strokes drawn over the document without deleting them; press again to show them. Example: hide the red circles before printing.",
        |d| d.rev_hide_ink(),
    ),
];

impl Draw<'_> {
    /// How far item `i` of `n` has landed in the film: 0 before its turn, 1 once it is there
    /// (always 1 in a still drawing).
    fn rev_step(&self, i: usize, n: usize) -> f32 {
        let n = n.max(1) as f32;
        let from = 0.1 + 0.6 * i as f32 / n;
        self.reveal(from, from + 0.15)
    }

    /// A comment balloon in `r` (a rounded box with two text lines), in accent when `hot`.
    fn rev_balloon(&self, r: Rect, hot: bool) {
        let c = if hot { self.accent() } else { self.t.border_strong };
        self.p.rect_filled(r, 4.0, if hot { self.accent().linear_multiply(0.15) } else { Color32::WHITE });
        self.p.rect_stroke(r, 4.0, Stroke::new(if hot { 1.5 } else { 1.0 }, c), egui::StrokeKind::Inside);
        self.text_lines(r.shrink(4.0), false, &[r.width() - 12.0, (r.width() - 8.0) * 0.6], 6.0, self.grey());
    }

    /// Ordinary lines in `area` with an inserted word on row 1 and a deleted word on row 3, and
    /// a change bar in the margin beside each. `marks` fades the marks: 1 shows them, 0 shows
    /// the plain words of the page as it would read with every change accepted.
    fn rev_marked_paragraph(&self, area: Rect, marks: f32) {
        let w = area.width();
        self.text_lines(area, false, &[w, 0.0, w * 0.96, 0.0, w * 0.92, w * 0.5], 10.0, self.grey());
        let y1 = area.min.y + 4.0 + 10.0;
        self.line(pos2(area.min.x, y1), pos2(area.min.x + w * 0.3, y1), self.grey(), 1.5);
        if marks < 1.0 {
            self.word(area.min.x + w * 0.34, y1, w * 0.26, self.ink().linear_multiply(1.0 - marks));
        }
        if marks > 0.0 {
            self.word(area.min.x + w * 0.34, y1, w * 0.26, self.green().linear_multiply(marks));
            self.line(pos2(area.min.x + w * 0.34, y1 + 4.5), pos2(area.min.x + w * 0.6, y1 + 4.5), self.green().linear_multiply(marks), 1.0);
        }
        self.line(pos2(area.min.x + w * 0.64, y1), pos2(area.max.x, y1), self.grey(), 1.5);
        let y3 = area.min.y + 4.0 + 30.0;
        self.line(pos2(area.min.x, y3), pos2(area.min.x + w * 0.2, y3), self.grey(), 1.5);
        if marks > 0.0 {
            self.word(area.min.x + w * 0.24, y3, w * 0.3, self.red().linear_multiply(0.6 * marks));
            self.line(pos2(area.min.x + w * 0.24 - 1.0, y3), pos2(area.min.x + w * 0.54 + 1.0, y3), self.red().linear_multiply(marks), 1.5);
        }
        self.line(pos2(area.min.x + w * 0.58, y3), pos2(area.min.x + w * 0.9, y3), self.grey(), 1.5);
        if marks > 0.0 {
            for y in [y1, y3] {
                self.line(pos2(area.min.x - 5.0, y - 4.0), pos2(area.min.x - 5.0, y + 4.0), self.accent().linear_multiply(marks), 1.5);
            }
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

    /// A selected word on the page, and a list of words that mean the same, filling in.
    fn rev_thesaurus(&self) {
        let a = self.page(8.0, 8.0, 76.0, 88.0);
        self.paragraph(a, Dir::Ltr, 7, self.grey());
        let y = a.min.y + 4.0 + 20.0;
        let w = Rect::from_min_size(pos2(a.min.x + 12.0, y - 4.0), vec2(18.0, 8.0));
        self.selection(w);
        self.label(w.center(), "big", 7.0, self.ink(), Align2::CENTER_CENTER);
        let pane = self.pane(true, 78.0, 0);
        let arrow = self.reveal(0.0, 0.2);
        if arrow > 0.0 {
            let (x0, x1) = (w.max.x + 2.0, pane.min.x - 2.0);
            self.arrow_right(pos2(x0, y), pos2(x0 + (x1 - x0) * arrow, y));
        }
        for (i, s) in ["large", "huge", "vast", "major", "sizable"].iter().enumerate() {
            let k = self.rev_step(i, 5);
            if k <= 0.0 {
                continue;
            }
            let yy = pane.min.y + 22.0 + i as f32 * 13.0;
            let c = if i == 0 { self.accent() } else { self.ink() };
            self.label(pos2(pane.min.x + 8.0, yy), s, 7.0, c.linear_multiply(k), Align2::LEFT_CENTER);
        }
    }

    /// The Word Count dialog: the counts tick up to their values.
    fn rev_word_count(&self) {
        let inner = self.dialog("Word Count", 0);
        let count = self.reveal(0.1, 0.7);
        for (i, (k, v, n)) in
            [("Pages", "3", 3.0), ("Words", "1 240", 1240.0), ("Characters", "6 802", 6802.0), ("Paragraphs", "38", 38.0)].iter().enumerate()
        {
            let y = inner.min.y + 4.0 + i as f32 * 11.0;
            self.label(pos2(inner.min.x, y), k, 6.5, self.grey(), Align2::LEFT_CENTER);
            let shown = if count >= 1.0 { (*v).to_string() } else { format!("{}", (n * count) as u32) };
            self.label(pos2(inner.max.x, y), &shown, 6.5, self.ink(), Align2::RIGHT_CENTER);
        }
    }

    /// A page being read: the spoken word's highlight moves along the line to the word.
    fn rev_read_aloud(&self) {
        let a = self.page(8.0, 8.0, 84.0, 88.0);
        self.paragraph(a, Dir::Ltr, 7, self.grey());
        let y = a.min.y + 4.0 + 20.0;
        let k = self.reveal(0.1, 0.7);
        let w = Rect::from_min_size(pos2(a.min.x + 20.0 * k, y - 4.0), vec2(22.0, 8.0));
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
        if self.reveal(0.15, 0.3) > 0.0 {
            self.warning(pic.right_top() + vec2(-4.0, 4.0));
        }
        self.text_lines(Rect::from_min_max(pos2(a.min.x, a.min.y + 48.0), a.max), false, &[64.0, 60.0, 40.0], 7.0, self.grey());
        let pane = self.pane(true, 72.0, 3);
        for (i, c) in [Color32::from_rgb(220, 150, 20), Color32::from_rgb(220, 150, 20), self.green()].into_iter().enumerate() {
            if self.rev_step(i, 3) > 0.0 {
                self.p.circle_filled(pos2(pane.min.x + 8.0, pane.min.y + 20.0 + i as f32 * 11.0), 2.5, c);
            }
        }
    }

    /// Selected text; the language tag drops in under it.
    fn rev_language(&self) {
        let a = self.page_center();
        self.selection(self.band(a, 1..=2));
        self.paragraph(a, Dir::Ltr, 5, self.grey());
        let k = self.reveal(0.2, 0.55);
        if k <= 0.0 {
            return;
        }
        let c = pos2(a.center().x, a.max.y - 12.0 - (1.0 - k) * 10.0);
        self.p.rect_filled(Rect::from_center_size(c, vec2(44.0, 16.0)), 3.0, self.accent().linear_multiply(k.max(0.3)));
        self.label(c, "ع / EN", 8.0, Color32::WHITE, Align2::CENTER_CENTER);
    }

    /// The page, the margin and a balloon joined to the commented words; `fade` dims the balloon.
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

    /// The balloon, then the cross lands on it.
    fn rev_delete_comment(&self) {
        let b = self.rev_page_with_balloon(true);
        let k = self.reveal(0.3, 0.55);
        if k > 0.0 {
            let c = b.right_bottom() + vec2(-4.0, 2.0);
            self.p.circle_filled(c, 11.0 + (1.0 - k) * 6.0, self.red().linear_multiply(k.max(0.3)));
            self.line(c + vec2(-5.0, -5.0), c + vec2(5.0, 5.0), Color32::WHITE, 2.5);
            self.line(c + vec2(-5.0, 5.0), c + vec2(5.0, -5.0), Color32::WHITE, 2.5);
        }
    }

    /// Three balloons in the margin; the arrow grows to the previous or the next one, which
    /// then lights up.
    fn rev_comment_nav(&self, next: bool) {
        let a = self.page(8.0, 8.0, 84.0, 88.0);
        self.paragraph(a, Dir::Ltr, 7, self.grey());
        let ys = [12.0, 42.0, 72.0];
        let k = self.reveal(0.1, 0.5);
        let hot = if k < 1.0 {
            1
        } else if next {
            2
        } else {
            0
        };
        for (i, y) in ys.iter().enumerate() {
            let b = Rect::from_min_size(self.at(120.0, *y), vec2(48.0, 20.0));
            self.rev_balloon(b, i == hot);
            self.line(pos2(a.min.x + 20.0 + i as f32 * 10.0, a.min.y + 4.0 + i as f32 * 20.0 + 10.0), b.left_center(), self.t.border, 1.0);
        }
        let x = self.at(108.0, 0.0).x;
        if k <= 0.0 {
            return;
        }
        if next {
            self.arrow_down(pos2(x, self.at(0.0, 52.0).y), pos2(x, self.at(0.0, 52.0 + 30.0 * k).y));
        } else {
            self.arrow_up(pos2(x, self.at(0.0, 52.0).y), pos2(x, self.at(0.0, 52.0 - 30.0 * k).y));
        }
    }

    /// The page and a pane whose balloons slide in one after the other.
    fn rev_comments_pane(&self) {
        let a = self.page(8.0, 8.0, 76.0, 88.0);
        self.paragraph(a, Dir::Ltr, 7, self.grey());
        self.highlight(Rect::from_min_size(pos2(a.min.x + 10.0, a.min.y + 20.0), vec2(22.0, 8.0)));
        self.highlight(Rect::from_min_size(pos2(a.min.x + 24.0, a.min.y + 50.0), vec2(18.0, 8.0)));
        let pane = self.pane(true, 80.0, 0);
        for i in 0..3 {
            let k = self.rev_step(i, 3);
            if k <= 0.0 {
                continue;
            }
            let b =
                Rect::from_min_size(pos2(pane.min.x + 5.0 + (1.0 - k) * 30.0, pane.min.y + 17.0 + i as f32 * 26.0), vec2(pane.width() - 10.0, 22.0));
            let clipped = self.p.with_clip_rect(pane.intersect(self.p.clip_rect()));
            self.with_painter(&clipped).rev_balloon(b, i == 0);
        }
    }

    /// Typed text becomes marked text: additions underlined in green, deletions struck in red.
    fn rev_track_changes(&self) {
        self.before_after(|d, a| d.paragraph(a, Dir::Ltr, 6, d.grey()), |d, a| d.rev_marked_paragraph(a, 1.0));
    }

    /// A page with the markup choice above it; the marks fade to `marks` (1 = all shown,
    /// 0 = none) as the film plays.
    fn rev_markup(&self, choice: &str, marks: f32) {
        let a = self.page_center();
        let k = self.reveal(0.25, 0.65);
        let shown = 1.0 + (marks - 1.0) * k;
        self.rev_marked_paragraph(Rect::from_min_max(pos2(a.min.x + 4.0, a.min.y + 16.0), a.max), shown);
        let c = pos2(a.center().x, a.min.y + 4.0);
        self.chip(c, choice, 60.0);
        let t = c + vec2(26.0, 0.0);
        self.p.add(Shape::convex_polygon(vec![t + vec2(-3.0, -2.0), t + vec2(3.0, -2.0), t + vec2(0.0, 2.0)], self.ink(), Stroke::NONE));
    }

    /// The list of changes on the left filling in, the marked page on the right.
    fn rev_reviewing_pane(&self) {
        let pane = self.pane(false, 72.0, 5);
        for (i, c) in [self.green(), self.red(), self.accent(), self.green(), self.red()].iter().enumerate() {
            if self.rev_step(i, 5) > 0.0 {
                self.p.circle_filled(pos2(pane.min.x + 8.0, pane.min.y + 20.0 + i as f32 * 11.0), 2.5, *c);
            }
        }
        let a = self.page(84.0, 8.0, 84.0, 88.0);
        self.rev_marked_paragraph(Rect::from_min_max(pos2(a.min.x + 6.0, a.min.y), a.max), 1.0);
    }

    /// A marked change becomes plain text: kept (`accept`) or undone. One change (the selected
    /// one, in a selection band) or, with `all`, both of them.
    fn rev_decide(&self, accept: bool, all: bool) {
        self.before_after(
            |d, a| {
                let area = Rect::from_min_max(pos2(a.min.x + 6.0, a.min.y), a.max);
                if !all {
                    d.selection(d.band(area, 1..=1));
                }
                d.rev_marked_paragraph(area, 1.0);
            },
            |d, a| {
                let area = Rect::from_min_max(pos2(a.min.x + 6.0, a.min.y), a.max);
                let w = area.width();
                if all {
                    d.paragraph(area, Dir::Ltr, 6, d.grey());
                } else {
                    // The selected change is on row 1; row 3 keeps its mark.
                    d.rev_marked_paragraph(area, 1.0);
                    let band = d.band(area, 1..=1);
                    d.p.rect_filled(Rect::from_min_max(pos2(area.min.x - 7.0, band.min.y), pos2(area.max.x + 2.0, band.max.y)), 0.0, Color32::WHITE);
                    d.line(pos2(area.min.x, band.center().y), pos2(area.max.x, band.center().y), d.grey(), 1.5);
                }
                let y1 = area.min.y + 4.0 + 10.0;
                d.p.rect_filled(Rect::from_min_size(pos2(area.min.x + w * 0.34, y1 - 3.0), vec2(w * 0.26, 6.0)), 1.0, Color32::WHITE);
                if accept {
                    d.word(area.min.x + w * 0.34, y1, w * 0.26, d.ink());
                }
                if all {
                    let y3 = area.min.y + 4.0 + 30.0;
                    d.p.rect_filled(Rect::from_min_size(pos2(area.min.x + w * 0.24, y3 - 3.0), vec2(w * 0.3, 6.0)), 1.0, Color32::WHITE);
                    if !accept {
                        d.word(area.min.x + w * 0.24, y3, w * 0.3, d.ink());
                    }
                }
                let c = pos2(a.max.x - 4.0, a.max.y - 4.0);
                if accept {
                    d.check(c);
                } else {
                    d.cross(c);
                }
                if all {
                    d.label(c + vec2(-16.0, 0.0), "×2", 7.0, d.ink(), Align2::CENTER_CENTER);
                }
            },
        );
    }

    /// Two marked changes on the page; the arrow grows to the previous or the next one.
    fn rev_change_nav(&self, next: bool) {
        let a = self.page_center();
        let area = Rect::from_min_max(pos2(a.min.x + 6.0, a.min.y), a.max);
        self.rev_marked_paragraph(area, 1.0);
        let w = area.width();
        let y1 = area.min.y + 4.0 + 10.0;
        let y3 = area.min.y + 4.0 + 30.0;
        let k = self.reveal(0.1, 0.5);
        let target = if (k >= 1.0) == next {
            Rect::from_min_size(pos2(area.min.x + w * 0.24, y3 - 4.0), vec2(w * 0.3, 8.0))
        } else {
            Rect::from_min_size(pos2(area.min.x + w * 0.34, y1 - 4.0), vec2(w * 0.26, 8.0))
        };
        self.p.rect_stroke(target.expand(2.0), 2.0, Stroke::new(1.5, self.accent()), egui::StrokeKind::Outside);
        let x = area.max.x + 6.0;
        if k <= 0.0 {
            return;
        }
        if next {
            self.arrow_down(pos2(x, y1), pos2(x, y1 + (y3 - y1) * k));
        } else {
            self.arrow_up(pos2(x, y3), pos2(x, y3 - (y3 - y1) * k));
        }
    }

    /// Two versions side by side, the lines that differ marked one after the other.
    fn rev_compare(&self) {
        let left = self.page(8.0, 8.0, 70.0, 88.0);
        let right = self.page(98.0, 8.0, 70.0, 88.0);
        for a in [left, right] {
            self.paragraph(a, Dir::Ltr, 7, self.grey());
        }
        for (i, (a, row, c)) in [(left, 2.0, self.red()), (right, 2.0, self.green()), (right, 5.0, self.green())].into_iter().enumerate() {
            let k = self.rev_step(i, 3);
            if k <= 0.0 {
                continue;
            }
            let y = a.min.y + 4.0 + row * 10.0;
            let x1 = a.max.x * 0.4 + a.min.x * 0.6;
            self.line(pos2(a.min.x, y), pos2(a.min.x + (x1 - a.min.x) * k, y), c, 2.0);
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

    /// A selected block of the page; a small lock drops onto it; the rest stays free.
    fn rev_block_authors(&self) {
        let a = self.page_center();
        let band = self.band(a, 2..=4);
        self.selection(band);
        self.paragraph(a, Dir::Ltr, 7, self.grey());
        self.p.rect_stroke(band, 1.0, Stroke::new(1.5, self.accent()), egui::StrokeKind::Outside);
        let k = self.reveal(0.2, 0.55);
        if k > 0.0 {
            self.rev_lock(pos2(band.max.x + 12.0, band.center().y - 4.0 - (1.0 - k) * 14.0));
        }
    }

    /// A page; the lock drops onto it.
    fn rev_restrict(&self) {
        let a = self.page_lr();
        let k = self.reveal(0.2, 0.55);
        if k > 0.0 {
            self.rev_lock(pos2(a.center().x, a.center().y + 2.0 - (1.0 - k) * 16.0));
        }
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
