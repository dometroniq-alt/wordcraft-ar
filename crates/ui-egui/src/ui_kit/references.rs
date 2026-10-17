//! References tab thumbnails: the table of contents, footnotes and endnotes, citations and the
//! bibliography, captions, the index and the table of authorities.

use egui::{Align2, Rect, Shape, Stroke, pos2, vec2};

use super::Entry;
use super::draw::{Dir, Draw};

pub const ENTRIES: &[Entry] = &[
    Entry::new("references.toc", "Builds a table of contents from the document's headings, with the page number of each.", |d| {
        d.ref_toc_page();
    }),
    Entry::new("references.addText", "Marks the selected paragraph as a heading of a chosen level so it appears in the table of contents.", |d| {
        d.ref_add_text()
    }),
    Entry::new("references.updateToc", "Rebuilds the table of contents so new headings and changed page numbers are current.", |d| {
        let a = d.ref_toc_page();
        d.ref_refresh(pos2(a.max.x + 2.0, a.min.y + 2.0));
    }),
    Entry::new("references.footnote", "Puts a small number after the word and a note explaining it at the bottom of the page.", |d| d.ref_footnote()),
    Entry::new("references.endnote", "Puts a small number after the word and collects the note on a last page at the end of the document.", |d| {
        d.ref_endnote()
    }),
    Entry::new("references.nextFootnote", "Jumps the caret to the next footnote mark in the text.", |d| d.ref_next_footnote()),
    Entry::new("references.notes", "Shows the footnotes and endnotes of the document in a list so you can read and edit them.", |d| {
        d.ref_show_notes()
    }),
    Entry::new("references.researcher", "Would search the web for sources to quote; WordCraft works offline, so it only says so.", |d| {
        d.ref_researcher()
    }),
    Entry::new("references.citation", "Inserts a short reference such as (Ali, 2024) at the caret, naming the source you quote.", |d| {
        d.ref_citation()
    }),
    Entry::new("references.sources", "Keeps the list of books and articles you cite: add, edit or remove a source.", |d| d.ref_sources()),
    Entry::new("references.citationStyle", "Chooses the format of the citations and the bibliography (APA, MLA, Chicago…).", |d| {
        d.ref_citation_style()
    }),
    Entry::new("references.bibliography", "Adds a References list at the end that names every source you cited.", |d| d.ref_bibliography()),
    Entry::new("references.caption", "Writes a numbered label such as Figure 1 under the selected picture or table.", |d| d.ref_caption()),
    Entry::new("references.tableOfFigures", "Builds a list of all the captions (figures, tables) with their page numbers.", |d| {
        d.ref_figures_page();
    }),
    Entry::new("references.updateFigures", "Rebuilds the table of figures so new captions and changed page numbers are current.", |d| {
        let a = d.ref_figures_page();
        d.ref_refresh(pos2(a.max.x + 2.0, a.min.y + 2.0));
    }),
    Entry::new("references.markEntry", "Tags the selected word so it is listed in the index with its page number.", |d| d.ref_mark(false)),
    Entry::new("references.index", "Builds an alphabetical index at the end: every tagged word with the pages it appears on.", |d| {
        d.ref_index_page();
    }),
    Entry::new("references.updateIndex", "Rebuilds the index so newly tagged words and changed page numbers are current.", |d| {
        let a = d.ref_index_page();
        d.ref_refresh(pos2(a.max.x + 2.0, a.min.y + 2.0));
    }),
    Entry::new("references.markCitation", "Tags the selected legal citation (a case, a statute) for the table of authorities.", |d| d.ref_mark(true)),
    Entry::new(
        "references.tableOfAuthorities",
        "Builds a list of the cases, statutes and other legal sources cited, grouped by kind, with their pages.",
        |d| d.ref_authorities(),
    ),
];

impl Draw<'_> {
    /// One entry of a table: a text line `w` wide stepped in by `indent`, a dotted leader and a
    /// page number at the right edge of `area`.
    fn ref_leader_entry(&self, area: Rect, y: f32, indent: f32, w: f32, num: &str, c: egui::Color32) {
        let x0 = area.min.x + indent;
        self.line(pos2(x0, y), pos2(x0 + w, y), c, 1.5);
        let mut x = x0 + w + 4.0;
        while x < area.max.x - 10.0 {
            self.p.circle_filled(pos2(x, y + 1.0), 0.6, self.grey());
            x += 3.0;
        }
        self.label(pos2(area.max.x, y), num, 6.0, c, Align2::RIGHT_CENTER);
    }

    /// A circular "update" arrow centred at `c`.
    fn ref_refresh(&self, c: egui::Pos2) {
        self.p.circle_filled(c, 11.0, self.accent());
        let r = 6.0;
        let n = 14;
        let (a0, a1) = (-1.2_f32, 3.6_f32);
        let pts: Vec<_> = (0..=n).map(|i| a0 + (a1 - a0) * i as f32 / n as f32).map(|ang| c + vec2(ang.cos(), ang.sin()) * r).collect();
        self.p.add(Shape::line(pts, Stroke::new(1.8, egui::Color32::WHITE)));
        let tip = c + vec2(a0.cos(), a0.sin()) * r;
        self.p.add(Shape::convex_polygon(
            vec![tip + vec2(0.0, -3.5), tip + vec2(3.5, 1.0), tip + vec2(-3.0, 2.0)],
            egui::Color32::WHITE,
            Stroke::NONE,
        ));
    }

    /// A small raised mark (a note number) at `pos`.
    fn ref_superscript(&self, pos: egui::Pos2, s: &str) {
        self.label(pos + vec2(1.0, -4.0), s, 6.0, self.accent(), Align2::LEFT_CENTER);
    }

    /// A table of contents on the centred page; returns the page's text area.
    fn ref_toc_page(&self) -> Rect {
        let a = self.page_center();
        self.label(pos2(a.min.x, a.min.y + 4.0), "Contents", 7.5, self.accent(), Align2::LEFT_CENTER);
        let rows = [(0.0, 26.0, "1"), (6.0, 20.0, "2"), (6.0, 22.0, "4"), (0.0, 28.0, "7"), (6.0, 18.0, "8"), (0.0, 24.0, "12")];
        for (i, (indent, w, num)) in rows.iter().enumerate() {
            let y = a.min.y + 16.0 + i as f32 * 11.0;
            let c = if *indent == 0.0 { self.ink() } else { self.grey() };
            self.ref_leader_entry(a, y, *indent, *w, num, c);
        }
        a
    }

    /// A selected paragraph becomes a heading, with its level.
    fn ref_add_text(&self) {
        self.before_after(
            |d, a| {
                d.selection(d.band(a, 2..=2));
                d.paragraph(a, Dir::Ltr, 6, d.grey());
            },
            |d, a| {
                let y = a.min.y + 4.0 + 2.0 * 10.0;
                d.text_lines(a, false, &[a.width(), a.width() * 0.94], 10.0, d.grey());
                d.line(pos2(a.min.x, y), pos2(a.min.x + a.width() * 0.6, y), d.accent(), 3.0);
                d.text_lines(
                    Rect::from_min_max(pos2(a.min.x, y + 6.0), a.max),
                    false,
                    &[a.width() * 0.9, a.width() * 0.96, a.width() * 0.5],
                    10.0,
                    d.grey(),
                );
                d.chip(pos2(a.center().x, a.max.y - 6.0), "Level 1", 40.0);
            },
        );
    }

    /// A raised number in the text, and the note under a short rule at the foot of the page.
    fn ref_footnote(&self) {
        let a = self.page_center();
        self.paragraph(a, Dir::Ltr, 4, self.grey());
        let y = a.min.y + 4.0 + 10.0;
        self.ref_superscript(pos2(a.min.x + a.width() * 0.94, y), "1");
        let rule_y = a.max.y - 16.0;
        self.line(pos2(a.min.x, rule_y), pos2(a.min.x + 20.0, rule_y), self.ink(), 1.0);
        self.label(pos2(a.min.x, rule_y + 7.0), "1", 6.0, self.accent(), Align2::LEFT_CENTER);
        self.line(pos2(a.min.x + 7.0, rule_y + 7.0), pos2(a.max.x, rule_y + 7.0), self.accent(), 1.5);
        self.line(pos2(a.min.x + 7.0, rule_y + 13.0), pos2(a.min.x + 30.0, rule_y + 13.0), self.accent(), 1.5);
    }

    /// The mark on the first page, the note on the last page.
    fn ref_endnote(&self) {
        let first = self.page(8.0, 8.0, 66.0, 88.0);
        self.paragraph(first, Dir::Ltr, 6, self.grey());
        self.ref_superscript(pos2(first.min.x + first.width() * 0.94, first.min.y + 14.0), "i");
        self.arrow_right(self.at(78.0, 52.0), self.at(98.0, 52.0));
        let last = self.page(102.0, 8.0, 66.0, 88.0);
        self.label(pos2(last.min.x, last.min.y + 4.0), "Notes", 7.0, self.accent(), Align2::LEFT_CENTER);
        self.line(pos2(last.min.x, last.min.y + 10.0), pos2(last.min.x + 20.0, last.min.y + 10.0), self.ink(), 1.0);
        self.label(pos2(last.min.x, last.min.y + 18.0), "i", 6.0, self.accent(), Align2::LEFT_CENTER);
        self.line(pos2(last.min.x + 7.0, last.min.y + 18.0), pos2(last.max.x, last.min.y + 18.0), self.accent(), 1.5);
        self.line(pos2(last.min.x + 7.0, last.min.y + 24.0), pos2(last.min.x + 26.0, last.min.y + 24.0), self.accent(), 1.5);
    }

    /// Two note marks in the text; the caret jumps from the first to the second.
    fn ref_next_footnote(&self) {
        let a = self.page_center();
        self.paragraph(a, Dir::Ltr, 7, self.grey());
        let y1 = a.min.y + 4.0 + 10.0;
        let y2 = a.min.y + 4.0 + 40.0;
        let x1 = a.min.x + a.width() * 0.5;
        let x2 = a.min.x + a.width() * 0.3;
        self.ref_superscript(pos2(x1, y1), "1");
        self.ref_superscript(pos2(x2, y2), "2");
        self.caret(pos2(x2 + 6.0, y2 - 5.0), 10.0);
        self.arrow_down(pos2(x1 + 3.0, y1 + 4.0), pos2(x2 + 3.0, y2 - 8.0));
    }

    /// The page with its marks and a pane that lists the notes.
    fn ref_show_notes(&self) {
        let a = self.page(8.0, 8.0, 80.0, 88.0);
        self.paragraph(a, Dir::Ltr, 7, self.grey());
        self.ref_superscript(pos2(a.min.x + a.width() * 0.9, a.min.y + 4.0), "1");
        self.ref_superscript(pos2(a.min.x + a.width() * 0.6, a.min.y + 34.0), "2");
        let pane = self.pane(true, 72.0, 3);
        for (i, n) in ["1", "2", "3"].iter().enumerate() {
            let y = pane.min.y + 20.0 + i as f32 * 11.0;
            self.label(pos2(pane.min.x + 2.0, y), n, 6.0, self.accent(), Align2::LEFT_CENTER);
        }
    }

    /// A magnifier over a page, and a crossed-out cloud: no online search here.
    fn ref_researcher(&self) {
        let a = self.page(8.0, 8.0, 76.0, 88.0);
        self.paragraph(a, Dir::Ltr, 6, self.grey());
        self.magnifier(self.at(112.0, 40.0));
        let c = self.at(146.0, 78.0);
        for (dx, r) in [(-8.0, 7.0), (0.0, 10.0), (9.0, 7.0)] {
            self.p.circle_filled(c + vec2(dx, 0.0), r, self.t.hover);
        }
        self.line(c + vec2(-16.0, 12.0), c + vec2(16.0, -12.0), self.red(), 2.0);
    }

    /// A citation chip inside a line of text.
    fn ref_citation(&self) {
        let a = self.page_center();
        self.text_lines(a, false, &[a.width(), a.width() * 0.94, 16.0], 10.0, self.grey());
        self.ref_citation_chip(pos2(a.min.x + 18.0, a.min.y + 24.0));
        self.text_lines(
            Rect::from_min_max(pos2(a.min.x, a.min.y + 30.0), a.max),
            false,
            &[a.width() * 0.9, a.width() * 0.96, a.width() * 0.5],
            10.0,
            self.grey(),
        );
    }

    /// "(Ali, 2024)" in an accent chip whose top-left is `at`.
    fn ref_citation_chip(&self, at: egui::Pos2) {
        let r = Rect::from_min_size(at, vec2(40.0, 10.0));
        self.p.rect_filled(r, 2.0, self.accent().linear_multiply(0.2));
        self.p.rect_stroke(r, 2.0, self.stroke(self.accent()), egui::StrokeKind::Inside);
        self.label(r.center(), "(Ali, 2024)", 6.0, self.accent(), Align2::CENTER_CENTER);
    }

    /// A dialog that lists the sources, with an add button.
    fn ref_sources(&self) {
        let inner = self.dialog("Sources", 0);
        for (i, name) in ["Ali 2024", "Sara 2021", "Omar 2019"].iter().enumerate() {
            let y = inner.min.y + 4.0 + i as f32 * 11.0;
            self.label(pos2(inner.min.x, y), name, 6.5, self.ink(), Align2::LEFT_CENTER);
            self.line(pos2(inner.min.x + 40.0, y), pos2(inner.max.x - 24.0, y), self.grey(), 1.5);
        }
        self.button(pos2(inner.max.x - 10.0, inner.max.y - 8.0), "+", false);
    }

    /// Three style names, the chosen one marked, and the citation they produce.
    fn ref_citation_style(&self) {
        for (i, s) in ["APA", "MLA", "Chicago"].iter().enumerate() {
            let c = self.at(40.0, 24.0 + i as f32 * 22.0);
            if i == 0 {
                self.badge(c, s, self.accent());
            } else {
                self.chip(c, s, 44.0);
            }
        }
        self.arrow_right(self.at(72.0, 24.0), self.at(92.0, 24.0));
        let a = self.page(98.0, 8.0, 70.0, 88.0);
        self.text_lines(a, false, &[a.width(), 10.0], 10.0, self.grey());
        self.ref_citation_chip(pos2(a.min.x + 12.0, a.min.y + 9.0));
        self.text_lines(Rect::from_min_max(pos2(a.min.x, a.min.y + 20.0), a.max), false, &[a.width() * 0.9, a.width() * 0.5], 10.0, self.grey());
    }

    /// A "References" heading and hanging entries.
    fn ref_bibliography(&self) {
        let a = self.page_center();
        self.label(pos2(a.min.x, a.min.y + 4.0), "References", 7.5, self.accent(), Align2::LEFT_CENTER);
        for i in 0..3 {
            let y = a.min.y + 18.0 + i as f32 * 22.0;
            self.line(pos2(a.min.x, y), pos2(a.max.x, y), self.ink(), 1.5);
            self.line(pos2(a.min.x + 8.0, y + 8.0), pos2(a.min.x + a.width() * 0.7, y + 8.0), self.grey(), 1.5);
        }
    }

    /// A picture with its numbered label underneath.
    fn ref_caption(&self) {
        let a = self.page_center();
        let pic = Rect::from_min_size(pos2(a.min.x + 8.0, a.min.y + 2.0), vec2(44.0, 30.0));
        self.picture(pic);
        self.label(pos2(a.center().x, pic.max.y + 7.0), "Figure 1", 7.0, self.accent(), Align2::CENTER_CENTER);
        self.paragraph(Rect::from_min_max(pos2(a.min.x, pic.max.y + 14.0), a.max), Dir::Ltr, 3, self.grey());
    }

    /// A table of figures on the centred page; returns its text area.
    fn ref_figures_page(&self) -> Rect {
        let a = self.page_center();
        self.label(pos2(a.min.x, a.min.y + 4.0), "Figures", 7.5, self.accent(), Align2::LEFT_CENTER);
        for (i, (name, num)) in [("Fig. 1", "2"), ("Fig. 2", "3"), ("Fig. 3", "5"), ("Fig. 4", "8"), ("Fig. 5", "9")].iter().enumerate() {
            let y = a.min.y + 18.0 + i as f32 * 12.0;
            self.label(pos2(a.min.x, y), name, 6.0, self.ink(), Align2::LEFT_CENTER);
            self.ref_leader_entry(a, y, 22.0, 12.0, num, self.grey());
        }
        a
    }

    /// A selected word with a tag chip pointing at it: an index entry or a legal citation.
    fn ref_mark(&self, legal: bool) {
        let a = self.page_center();
        self.paragraph(a, Dir::Ltr, 7, self.grey());
        let y = a.min.y + 4.0 + 30.0;
        let w = Rect::from_min_size(pos2(a.min.x + 14.0, y - 4.0), vec2(if legal { 28.0 } else { 20.0 }, 8.0));
        self.selection(w);
        self.line(pos2(w.min.x + 1.0, y), pos2(w.max.x - 1.0, y), self.ink(), 1.5);
        let c = pos2(w.center().x + 16.0, y - 18.0);
        self.line(w.center_top(), c + vec2(0.0, 7.0), self.accent(), 1.0);
        let (text, cw) = if legal { ("§ Case", 36.0) } else { ("Index", 32.0) };
        let r = Rect::from_center_size(c, vec2(cw, 12.0));
        self.p.rect_filled(r, 3.0, self.accent());
        self.label(c, text, 6.5, egui::Color32::WHITE, Align2::CENTER_CENTER);
    }

    /// An index in two columns on the centred page; returns its text area.
    fn ref_index_page(&self) -> Rect {
        let a = self.page_center();
        let cw = (a.width() - 6.0) / 2.0;
        for (col, letter) in ["A", "B"].iter().enumerate() {
            let x = a.min.x + col as f32 * (cw + 6.0);
            let colr = Rect::from_min_size(pos2(x, a.min.y), vec2(cw, a.height()));
            self.label(pos2(x, a.min.y + 4.0), letter, 7.0, self.accent(), Align2::LEFT_CENTER);
            for (i, num) in ["3", "7", "12", "4", "9"].iter().enumerate() {
                let y = a.min.y + 15.0 + i as f32 * 11.0;
                let indent = if i % 3 == 2 { 4.0 } else { 0.0 };
                self.ref_leader_entry(colr, y, indent, 10.0 - indent, num, self.ink());
            }
        }
        a
    }

    /// Legal sources grouped under "Cases" and "Statutes".
    fn ref_authorities(&self) {
        let a = self.page_center();
        let mut y = a.min.y + 4.0;
        for (head, nums) in [("Cases", ["2", "5", "6"]), ("Statutes", ["9", "11", "11"])] {
            self.label(pos2(a.min.x, y), head, 7.0, self.accent(), Align2::LEFT_CENTER);
            y += 10.0;
            for num in nums {
                self.ref_leader_entry(a, y, 4.0, 22.0, num, self.ink());
                y += 9.0;
            }
            y += 4.0;
        }
    }
}
