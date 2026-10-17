//! References tab thumbnails: the table of contents, footnotes and endnotes, citations and the
//! bibliography, captions, the index and the table of authorities. Each one a little film: the
//! contents rows land one by one, the note number appears and its note grows, the tag drops
//! onto the word, the update arrow turns and the page numbers change.

use egui::{Align2, Color32, Rect, Shape, Stroke, pos2, vec2};

use super::Entry;
use super::draw::{Dir, Draw};

pub const ENTRIES: &[Entry] = &[
    Entry::new(
        "references.toc",
        "Makes a list of the document's headings at the start, each with the page it is on, like the contents page of a book. Example: «١. المقدمة ..... ٢» then «٢. الواجبات ..... ٥».",
        |d| {
            d.ref_toc_page(false);
        },
    ),
    Entry::new(
        "ui.dialog:toc",
        "Opens a box where you set up the table of contents yourself: how many heading levels to list, whether to show page numbers and the dotted line before them. Example: list levels 1 and 2 only, without dots.",
        |d| d.ref_toc_dialog(),
    ),
    Entry::new(
        "references.removeToc",
        "Deletes the table of contents from the document; the headings themselves stay in the text. Example: the contents page goes, and chapter 1 still begins on page 2.",
        |d| d.ref_remove_toc(),
    ),
    Entry::new(
        "references.addText",
        "Makes the paragraph the caret is in a heading of the level you pick, so it appears in the table of contents; «Do Not Show» takes it out again. Example: the plain line «الواجبات» becomes a level 1 heading and is listed.",
        |d| d.ref_add_text(),
    ),
    Entry::new(
        "references.updateToc",
        "Rebuilds the table of contents so the headings you added and the page numbers that moved are right again. Example: after adding a chapter, «الواجبات ..... ٥» becomes «الواجبات ..... ٧».",
        |d| {
            let a = d.ref_toc_page(true);
            d.ref_refresh(pos2(a.max.x + 2.0, a.min.y + 2.0));
        },
    ),
    Entry::new(
        "references.footnote",
        "Puts a small raised number after the word and lets you write a note about it at the bottom of the same page. Example: «الآمر¹» in the text and «¹ آمر الوحدة الطبية» at the foot of the page.",
        |d| d.ref_footnote(),
    ),
    Entry::new(
        "references.endnote",
        "Puts a small raised number after the word and collects the note at the very end of the document instead of on the page. Example: «الآمر¹» in the text and the note on the last page under Notes.",
        |d| d.ref_endnote(),
    ),
    Entry::new(
        "references.nextFootnote",
        "Jumps the caret from one note number in the text to the next one, so you can check the notes in order. Example: from note ¹ on page 2 to note ² on page 3.",
        |d| d.ref_next_footnote(),
    ),
    Entry::new(
        "references.notes",
        "Opens a list of every footnote and endnote beside the page, so you can read and edit them all in one place. Example: notes 1, 2 and 3 listed with their text.",
        |d| d.ref_show_notes(),
    ),
    Entry::new(
        "references.researcher",
        "Would search the internet for sources to quote, but WordCraft works without the internet, so it only tells you that. Example: press it and a message says the search is not available.",
        |d| d.ref_researcher(),
    ),
    Entry::new(
        "references.citation",
        "Inserts a short reference to a source right where you are typing, in brackets, naming who wrote it and when. Example: «... كما بيّنت الدراسة (Ali, 2024)» at the end of the sentence.",
        |d| d.ref_citation(),
    ),
    Entry::new(
        "references.sources",
        "Opens your list of books and articles so you can add one, correct one or remove one; a source you add once can be cited anywhere. Example: add «علي، ٢٠٢٤، الطب الميداني» and cite it in three places.",
        |d| d.ref_sources(),
    ),
    Entry::new(
        "ui.dialog:createSource",
        "Opens a form where you type a new source: its author, title, year and publisher; it then joins your list, ready to cite. Example: fill in «علي / الطب الميداني / ٢٠٢٤» and press OK.",
        |d| d.ref_create_source(),
    ),
    Entry::new(
        "references.citationStyle",
        "Chooses how citations and the references list are written: the author's name and the year, or a number in brackets, in one of the known styles. Example: APA writes (Ali, 2024); IEEE writes [1].",
        |d| d.ref_citation_style(),
    ),
    Entry::new(
        "references.citationStyle:APA",
        "APA style: the author and the year in brackets, and the references list sorted by author. Example: (Ali, 2024) in the text; «Ali, H. (2024). Field medicine.» in the list.",
        |d| d.ref_style("APA", "", "(Ali, 2024)", "A → Z", "References", ["", ""]),
    ),
    Entry::new(
        "references.citationStyle:Chicago",
        "Chicago style: the author and the year without a comma, and the references list sorted by author. Example: (Ali 2024) in the text; «Ali, Hasan. 2024. Field Medicine.» in the list.",
        |d| d.ref_style("Chicago", "", "(Ali 2024)", "A → Z", "References", ["", ""]),
    ),
    Entry::new(
        "references.citationStyle:GB/T 7714",
        "The Chinese standard GB/T 7714: a number in square brackets, and the list in the order the sources were cited. Example: [1] in the text; «[1] Ali H. Field medicine[M]. 2024.» in the list.",
        |d| d.ref_style("GB/T 7714", "", "[1]", "1, 2, 3", "References", ["[1]", "[2]"]),
    ),
    Entry::new(
        "references.citationStyle:GOST (name sort)",
        "The Russian standard GOST: a number in square brackets, and the list sorted by the author's name. Example: [1] in the text; «1. Ali H. Field medicine. 2024.» in the list, A to Z.",
        |d| d.ref_style("GOST", "name sort", "[1]", "A → Z", "References", ["1.", "2."]),
    ),
    Entry::new(
        "references.citationStyle:GOST (title sort)",
        "The Russian standard GOST: a number in square brackets, and the list sorted by the title of each source. Example: [1] in the text; the list ordered «Anatomy…» before «Field medicine…».",
        |d| d.ref_style("GOST", "title sort", "[1]", "Title A → Z", "References", ["1.", "2."]),
    ),
    Entry::new(
        "references.citationStyle:Harvard (Anglia)",
        "Harvard style: the author and the year in brackets, and the references list sorted by author. Example: (Ali, 2024) in the text; «Ali, H., 2024. Field medicine.» in the list.",
        |d| d.ref_style("Harvard", "Anglia", "(Ali, 2024)", "A → Z", "References", ["", ""]),
    ),
    Entry::new(
        "references.citationStyle:IEEE",
        "IEEE style, used in engineering: a number in square brackets, and the list in the order the sources were cited. Example: [1] in the text; «[1] H. Ali, “Field medicine,” 2024.» in the list.",
        |d| d.ref_style("IEEE", "", "[1]", "1, 2, 3", "References", ["[1]", "[2]"]),
    ),
    Entry::new(
        "references.citationStyle:ISO 690 (author-date)",
        "The international standard ISO 690 with the author and the year in brackets, and the list sorted by author. Example: (Ali, 2024) in the text; «ALI, H., 2024. Field medicine.» in the list.",
        |d| d.ref_style("ISO 690", "author-date", "(Ali, 2024)", "A → Z", "References", ["", ""]),
    ),
    Entry::new(
        "references.citationStyle:ISO 690 (numerical)",
        "The international standard ISO 690 with a number in round brackets, and the list in the order the sources were cited. Example: (1) in the text; «1. ALI, H. Field medicine. 2024.» in the list.",
        |d| d.ref_style("ISO 690", "numerical", "(1)", "1, 2, 3", "References", ["1.", "2."]),
    ),
    Entry::new(
        "references.citationStyle:MLA",
        "MLA style, used in the humanities: the author and the page number in brackets, and a Works Cited list sorted by author. Example: (Ali 12) in the text; «Ali, Hasan. “Field Medicine.” 2024.» in the list.",
        |d| d.ref_style("MLA", "", "(Ali 12)", "A → Z", "Works Cited", ["", ""]),
    ),
    Entry::new(
        "references.citationStyle:SIST02",
        "The Japanese standard SIST02: a number in round brackets, and the list in the order the sources were cited. Example: (1) in the text; «1) Ali H. Field medicine. 2024.» in the list.",
        |d| d.ref_style("SIST02", "", "(1)", "1, 2, 3", "References", ["1)", "2)"]),
    ),
    Entry::new(
        "references.citationStyle:Turabian",
        "Turabian style, Chicago's form for students: the author and the year, and a Bibliography sorted by author. Example: (Ali 2024) in the text; «Ali, Hasan. Field Medicine. 2024.» under Bibliography.",
        |d| d.ref_style("Turabian", "", "(Ali 2024)", "A → Z", "Bibliography", ["", ""]),
    ),
    Entry::new(
        "references.bibliography",
        "Adds a References list at the end of the document naming every source you cited, in the chosen style, and keeps it up to date. Example: a «References» heading followed by «Ali, H. (2024). Field medicine.».",
        |d| d.ref_bibliography(),
    ),
    Entry::new(
        "references.caption",
        "Writes a numbered label under the selected picture or table; the numbers follow on by themselves. Example: «شكل ١» under the first picture, «شكل ٢» under the next.",
        |d| d.ref_caption(),
    ),
    Entry::new(
        "references.tableOfFigures",
        "Makes a list of all the captions in the document (figures, tables) with their page numbers, like a contents page for the pictures. Example: «شكل ١ ..... ٢» then «شكل ٢ ..... ٣».",
        |d| {
            d.ref_figures_page(false);
        },
    ),
    Entry::new(
        "references.updateFigures",
        "Rebuilds the list of figures so new captions and page numbers that moved are right again. Example: after adding a picture, «شكل ٣» joins the list and the pages after it shift.",
        |d| {
            let a = d.ref_figures_page(true);
            d.ref_refresh(pos2(a.max.x + 2.0, a.min.y + 2.0));
        },
    ),
    Entry::new(
        "references.markEntry",
        "Tags the selected word so it is listed in the index at the end with its page numbers. Example: tag «الإخلاء» and the index shows «الإخلاء ..... ٤، ٩».",
        |d| d.ref_mark(false),
    ),
    Entry::new(
        "references.index",
        "Builds an alphabetical index at the end of the document: every tagged word and the pages it appears on. Example: «الإخلاء ٤، ٩» then «الإسعاف ٣».",
        |d| {
            d.ref_index_page(false);
        },
    ),
    Entry::new(
        "references.updateIndex",
        "Rebuilds the index so words you tagged later and page numbers that moved are right again. Example: a word tagged after the index was built now appears in it.",
        |d| {
            let a = d.ref_index_page(true);
            d.ref_refresh(pos2(a.max.x + 2.0, a.min.y + 2.0));
        },
    ),
    Entry::new(
        "references.markCitation",
        "Tags the selected legal reference (a law, a case, a decision) so it is listed in the table of authorities. Example: tag «قانون الخدمة رقم ٣» and it is listed with its pages.",
        |d| d.ref_mark(true),
    ),
    Entry::new(
        "references.tableOfAuthorities",
        "Builds a list of the laws, cases and decisions you tagged, grouped by kind, with the pages they appear on. Example: under Statutes, «قانون الخدمة ..... ٩».",
        |d| d.ref_authorities(),
    ),
];

impl Draw<'_> {
    /// How far row `i` of `n` has landed in the film: 0 before its turn, 1 once it is there
    /// (always 1 in a still drawing).
    fn ref_step(&self, i: usize, n: usize) -> f32 {
        let n = n.max(1) as f32;
        let from = 0.1 + 0.6 * i as f32 / n;
        self.reveal(from, from + 0.15)
    }

    /// One entry of a table: a text line `w` wide stepped in by `indent`, a dotted leader and a
    /// page number at the right edge of `area`.
    fn ref_leader_entry(&self, area: Rect, y: f32, indent: f32, w: f32, num: &str, c: Color32) {
        let x0 = area.min.x + indent;
        self.line(pos2(x0, y), pos2(x0 + w, y), c, 1.5);
        let mut x = x0 + w + 4.0;
        while x < area.max.x - 10.0 {
            self.p.circle_filled(pos2(x, y + 1.0), 0.6, self.grey());
            x += 3.0;
        }
        self.label(pos2(area.max.x, y), num, 6.0, c, Align2::RIGHT_CENTER);
    }

    /// A circular "update" arrow centred at `c`; it turns while the film plays.
    fn ref_refresh(&self, c: egui::Pos2) {
        self.p.circle_filled(c, 11.0, self.accent());
        let r = 6.0;
        let n = 14;
        let turn = if self.phase >= 1.0 { 0.0 } else { self.phase * std::f32::consts::TAU };
        let (a0, a1) = (-1.2_f32 + turn, 3.6_f32 + turn);
        let pts: Vec<_> = (0..=n).map(|i| a0 + (a1 - a0) * i as f32 / n as f32).map(|ang| c + vec2(ang.cos(), ang.sin()) * r).collect();
        self.p.add(Shape::line(pts, Stroke::new(1.8, Color32::WHITE)));
        let tip = c + vec2(a0.cos(), a0.sin()) * r;
        let rot = |v: egui::Vec2| vec2(v.x * turn.cos() - v.y * turn.sin(), v.x * turn.sin() + v.y * turn.cos());
        self.p.add(Shape::convex_polygon(
            vec![tip + rot(vec2(0.0, -3.5)), tip + rot(vec2(3.5, 1.0)), tip + rot(vec2(-3.0, 2.0))],
            Color32::WHITE,
            Stroke::NONE,
        ));
    }

    /// A small raised mark (a note number) at `pos`, faded by `k`.
    fn ref_superscript(&self, pos: egui::Pos2, s: &str, k: f32) {
        if k > 0.0 {
            self.label(pos + vec2(1.0, -4.0), s, 6.0, self.accent().linear_multiply(k), Align2::LEFT_CENTER);
        }
    }

    /// The rows of a table of contents in `area`: when `update`, every row is there and the
    /// page numbers of the last rows change mid-film; otherwise the rows land one by one.
    fn ref_toc_rows(&self, a: Rect, update: bool) {
        self.label(pos2(a.min.x, a.min.y + 4.0), "Contents", 7.5, self.accent(), Align2::LEFT_CENTER);
        let rows = [
            (0.0, 26.0, "1", "1"),
            (6.0, 20.0, "2", "2"),
            (6.0, 22.0, "4", "4"),
            (0.0, 28.0, "6", "7"),
            (6.0, 18.0, "7", "8"),
            (0.0, 24.0, "10", "12"),
        ];
        let swap = self.reveal(0.45, 0.6);
        for (i, (indent, w, old, new)) in rows.iter().enumerate() {
            let y = a.min.y + 16.0 + i as f32 * 11.0;
            let c = if *indent == 0.0 { self.ink() } else { self.grey() };
            if update {
                let changed = old != new;
                let num = if swap >= 1.0 || !changed { new } else { old };
                let c = if changed && swap >= 1.0 && self.phase < 1.0 { self.accent() } else { c };
                self.ref_leader_entry(a, y, *indent, *w, num, c);
            } else {
                let k = self.ref_step(i, rows.len());
                if k > 0.0 {
                    self.ref_leader_entry(a, y, *indent, *w * k, new, c.linear_multiply(k));
                }
            }
        }
    }

    /// A table of contents on the centred page; returns the page's text area.
    fn ref_toc_page(&self, update: bool) -> Rect {
        let a = self.page_center();
        self.ref_toc_rows(a, update);
        a
    }

    /// The Custom Table of Contents box: two settings and a preview of the rows it makes.
    fn ref_toc_dialog(&self) {
        let inner = self.dialog("Contents", 2);
        let k = self.reveal(0.3, 0.7);
        for (i, (w, num)) in [(20.0, "2"), (16.0, "5")].iter().enumerate() {
            let y = inner.min.y + 36.0 + i as f32 * 9.0;
            self.ref_leader_entry(inner.shrink2(vec2(2.0, 0.0)), y, i as f32 * 5.0, *w * k, num, self.ink().linear_multiply(k.max(0.2)));
        }
    }

    /// The contents page becomes an ordinary first page; the headings stay in the text.
    fn ref_remove_toc(&self) {
        self.before_after(
            |d, a| d.ref_toc_rows(a, true),
            |d, a| {
                d.heading_then_text(a, Dir::Ltr, 6, d.accent());
            },
        );
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

    /// The raised number appears in the text, then the note grows under a short rule at the
    /// foot of the page.
    fn ref_footnote(&self) {
        let a = self.page_center();
        self.paragraph(a, Dir::Ltr, 4, self.grey());
        let y = a.min.y + 4.0 + 10.0;
        self.ref_superscript(pos2(a.min.x + a.width() * 0.94, y), "1", self.reveal(0.1, 0.3));
        let rule_y = a.max.y - 16.0;
        self.line(pos2(a.min.x, rule_y), pos2(a.min.x + 20.0, rule_y), self.ink(), 1.0);
        let k = self.reveal(0.35, 0.8);
        if k > 0.0 {
            self.label(pos2(a.min.x, rule_y + 7.0), "1", 6.0, self.accent(), Align2::LEFT_CENTER);
            self.line(pos2(a.min.x + 7.0, rule_y + 7.0), pos2(a.min.x + 7.0 + (a.width() - 7.0) * k, rule_y + 7.0), self.accent(), 1.5);
            self.line(pos2(a.min.x + 7.0, rule_y + 13.0), pos2(a.min.x + 7.0 + 23.0 * k, rule_y + 13.0), self.accent(), 1.5);
        }
    }

    /// The mark on the first page, the arrow to the last page, the note there.
    fn ref_endnote(&self) {
        let first = self.page(8.0, 8.0, 66.0, 88.0);
        self.paragraph(first, Dir::Ltr, 6, self.grey());
        self.ref_superscript(pos2(first.min.x + first.width() * 0.94, first.min.y + 14.0), "i", self.reveal(0.05, 0.25));
        let arrow = self.reveal(0.25, 0.45);
        if arrow > 0.0 {
            self.arrow_right(self.at(78.0, 52.0), self.at(78.0 + 20.0 * arrow, 52.0));
        }
        let last = self.page(102.0, 8.0, 66.0, 88.0);
        self.label(pos2(last.min.x, last.min.y + 4.0), "Notes", 7.0, self.accent(), Align2::LEFT_CENTER);
        self.line(pos2(last.min.x, last.min.y + 10.0), pos2(last.min.x + 20.0, last.min.y + 10.0), self.ink(), 1.0);
        let k = self.reveal(0.5, 0.85);
        if k > 0.0 {
            self.label(pos2(last.min.x, last.min.y + 18.0), "i", 6.0, self.accent(), Align2::LEFT_CENTER);
            self.line(
                pos2(last.min.x + 7.0, last.min.y + 18.0),
                pos2(last.min.x + 7.0 + (last.width() - 7.0) * k, last.min.y + 18.0),
                self.accent(),
                1.5,
            );
            self.line(pos2(last.min.x + 7.0, last.min.y + 24.0), pos2(last.min.x + 7.0 + 19.0 * k, last.min.y + 24.0), self.accent(), 1.5);
        }
    }

    /// Two note marks in the text; the caret jumps from the first to the second.
    fn ref_next_footnote(&self) {
        let a = self.page_center();
        self.paragraph(a, Dir::Ltr, 7, self.grey());
        let y1 = a.min.y + 4.0 + 10.0;
        let y2 = a.min.y + 4.0 + 40.0;
        let x1 = a.min.x + a.width() * 0.5;
        let x2 = a.min.x + a.width() * 0.3;
        self.ref_superscript(pos2(x1, y1), "1", 1.0);
        self.ref_superscript(pos2(x2, y2), "2", 1.0);
        let k = self.reveal(0.15, 0.55);
        if k < 0.5 {
            self.caret(pos2(x1 + 6.0, y1 - 5.0), 10.0);
        } else {
            self.caret(pos2(x2 + 6.0, y2 - 5.0), 10.0);
        }
        if k > 0.0 {
            let from = pos2(x1 + 3.0, y1 + 4.0);
            let to = pos2(x2 + 3.0, y2 - 8.0);
            self.arrow_down(from, from + (to - from) * k);
        }
    }

    /// The page with its marks and a pane that lists the notes, one after the other.
    fn ref_show_notes(&self) {
        let a = self.page(8.0, 8.0, 80.0, 88.0);
        self.paragraph(a, Dir::Ltr, 7, self.grey());
        self.ref_superscript(pos2(a.min.x + a.width() * 0.9, a.min.y + 4.0), "1", 1.0);
        self.ref_superscript(pos2(a.min.x + a.width() * 0.6, a.min.y + 34.0), "2", 1.0);
        let pane = self.pane(true, 72.0, 0);
        for (i, n) in ["1", "2", "3"].iter().enumerate() {
            let k = self.ref_step(i, 3);
            if k <= 0.0 {
                continue;
            }
            let y = pane.min.y + 20.0 + i as f32 * 11.0;
            self.label(pos2(pane.min.x + 2.0, y), n, 6.0, self.accent().linear_multiply(k), Align2::LEFT_CENTER);
            self.line(pos2(pane.min.x + 10.0, y), pos2(pane.min.x + 10.0 + 50.0 * k, y), self.grey(), 1.5);
        }
    }

    /// A magnifier slides over the page, then the cloud is crossed out: no online search here.
    fn ref_researcher(&self) {
        let a = self.page(8.0, 8.0, 76.0, 88.0);
        self.paragraph(a, Dir::Ltr, 6, self.grey());
        let slide = self.reveal(0.05, 0.4);
        self.magnifier(self.at(98.0 + 14.0 * slide, 40.0));
        let c = self.at(146.0, 78.0);
        for (dx, r) in [(-8.0, 7.0), (0.0, 10.0), (9.0, 7.0)] {
            self.p.circle_filled(c + vec2(dx, 0.0), r, self.t.hover);
        }
        let k = self.reveal(0.5, 0.8);
        if k > 0.0 {
            self.line(c + vec2(-16.0, 12.0), c + vec2(-16.0 + 32.0 * k, 12.0 - 24.0 * k), self.red(), 2.0);
        }
    }

    /// A citation chip pops into a line of text.
    fn ref_citation(&self) {
        let a = self.page_center();
        self.text_lines(a, false, &[a.width(), a.width() * 0.94, 16.0], 10.0, self.grey());
        self.ref_citation_chip(pos2(a.min.x + 18.0, a.min.y + 24.0), "(Ali, 2024)", self.reveal(0.2, 0.5));
        self.text_lines(
            Rect::from_min_max(pos2(a.min.x, a.min.y + 30.0), a.max),
            false,
            &[a.width() * 0.9, a.width() * 0.96, a.width() * 0.5],
            10.0,
            self.grey(),
        );
    }

    /// `text` in an accent chip whose top-left is `at`; it fades in and settles by `k`.
    fn ref_citation_chip(&self, at: egui::Pos2, text: &str, k: f32) {
        if k <= 0.0 {
            return;
        }
        let w = (text.chars().count() as f32 * 3.6 + 6.0).max(14.0);
        let r = Rect::from_min_size(at - vec2(0.0, (1.0 - k) * 5.0), vec2(w, 10.0));
        self.p.rect_filled(r, 2.0, self.accent().linear_multiply(0.2 * k));
        self.p.rect_stroke(r, 2.0, self.stroke(self.accent().linear_multiply(k)), egui::StrokeKind::Inside);
        self.label(r.center(), text, 6.0, self.accent().linear_multiply(k), Align2::CENTER_CENTER);
    }

    /// A dialog that lists the sources one by one, with an add button.
    fn ref_sources(&self) {
        let inner = self.dialog("Sources", 0);
        for (i, name) in ["Ali 2024", "Sara 2021", "Omar 2019"].iter().enumerate() {
            let k = self.ref_step(i, 3);
            if k <= 0.0 {
                continue;
            }
            let y = inner.min.y + 4.0 + i as f32 * 11.0;
            self.label(pos2(inner.min.x, y), name, 6.5, self.ink().linear_multiply(k), Align2::LEFT_CENTER);
            self.line(pos2(inner.min.x + 40.0, y), pos2(inner.min.x + 40.0 + (inner.width() - 64.0) * k, y), self.grey(), 1.5);
        }
        if self.reveal(0.75, 0.9) > 0.0 {
            self.button(pos2(inner.max.x - 10.0, inner.max.y - 8.0), "+", false);
        }
    }

    /// The Create Source form: three labelled fields being filled in.
    fn ref_create_source(&self) {
        let inner = self.dialog("New Source", 0);
        for (i, (name, w)) in [("Author", 22.0), ("Title", 40.0), ("Year", 14.0)].iter().enumerate() {
            let y = inner.min.y + i as f32 * 14.0;
            self.label(pos2(inner.min.x, y + 5.0), name, 6.0, self.grey(), Align2::LEFT_CENTER);
            let f = Rect::from_min_size(pos2(inner.min.x + 34.0, y), vec2(inner.width() - 34.0, 10.0));
            self.p.rect_filled(f, 1.0, self.t.input);
            self.p.rect_stroke(f, 1.0, self.stroke(self.t.input_border), egui::StrokeKind::Inside);
            // The value is typed in, field by field.
            let k = self.ref_step(i, 3);
            if k > 0.0 {
                self.line(pos2(f.min.x + 3.0, y + 5.0), pos2(f.min.x + 3.0 + w * k, y + 5.0), self.ink(), 1.5);
            }
        }
    }

    /// Three style names, the chosen one marked, and the citation they produce.
    fn ref_citation_style(&self) {
        for (i, s) in ["APA", "MLA", "IEEE"].iter().enumerate() {
            let c = self.at(40.0, 24.0 + i as f32 * 22.0);
            if i == 0 {
                self.badge(c, s, self.accent());
            } else {
                self.chip(c, s, 44.0);
            }
        }
        let arrow = self.reveal(0.0, 0.25);
        if arrow > 0.0 {
            self.arrow_right(self.at(72.0, 24.0), self.at(72.0 + 20.0 * arrow, 24.0));
        }
        let a = self.page(98.0, 8.0, 70.0, 88.0);
        self.text_lines(a, false, &[a.width(), 10.0], 10.0, self.grey());
        self.ref_citation_chip(pos2(a.min.x + 12.0, a.min.y + 9.0), "(Ali, 2024)", self.reveal(0.25, 0.55));
        self.text_lines(Rect::from_min_max(pos2(a.min.x, a.min.y + 20.0), a.max), false, &[a.width() * 0.9, a.width() * 0.5], 10.0, self.grey());
    }

    /// One citation style: its name (and sub-name), how its list is ordered, the citation it
    /// writes in the text and the list it makes at the end (`markers` before each entry, empty
    /// for author styles).
    fn ref_style(&self, name: &str, sub: &str, cite: &str, order: &str, list_title: &str, markers: [&str; 2]) {
        self.badge(self.at(40.0, 22.0), name, self.accent());
        if !sub.is_empty() {
            self.label(self.at(40.0, 37.0), sub, 6.5, self.ink(), Align2::CENTER_CENTER);
        }
        self.label(self.at(40.0, 54.0), order, 6.5, self.grey(), Align2::CENTER_CENTER);
        let arrow = self.reveal(0.0, 0.25);
        if arrow > 0.0 {
            self.arrow_right(self.at(70.0, 52.0), self.at(70.0 + 14.0 * arrow, 52.0));
        }
        let a = self.page(90.0, 8.0, 80.0, 88.0);
        self.text_lines(a, false, &[a.width(), 12.0], 9.0, self.grey());
        self.ref_citation_chip(pos2(a.min.x + 14.0, a.min.y + 9.0), cite, self.reveal(0.25, 0.5));
        self.text_lines(Rect::from_min_max(pos2(a.min.x, a.min.y + 18.0), a.max), false, &[a.width() * 0.9, a.width() * 0.5], 9.0, self.grey());
        // The list at the foot of the page.
        let y0 = a.min.y + 44.0;
        self.label(pos2(a.min.x, y0), list_title, 6.5, self.accent(), Align2::LEFT_CENTER);
        for (i, m) in markers.iter().enumerate() {
            let k = self.reveal(0.5 + i as f32 * 0.2, 0.65 + i as f32 * 0.2);
            if k <= 0.0 {
                continue;
            }
            let y = y0 + 10.0 + i as f32 * 12.0;
            let mut x = a.min.x;
            if !m.is_empty() {
                self.label(pos2(a.min.x, y), m, 6.0, self.ink(), Align2::LEFT_CENTER);
                x += 12.0;
            }
            self.line(pos2(x, y), pos2(x + (a.max.x - x) * k, y), self.ink(), 1.5);
            self.line(pos2(a.min.x + 8.0, y + 5.0), pos2(a.min.x + 8.0 + a.width() * 0.5 * k, y + 5.0), self.grey(), 1.5);
        }
    }

    /// A "References" heading and hanging entries that appear one after the other.
    fn ref_bibliography(&self) {
        let a = self.page_center();
        self.label(pos2(a.min.x, a.min.y + 4.0), "References", 7.5, self.accent(), Align2::LEFT_CENTER);
        for i in 0..3 {
            let k = self.ref_step(i, 3);
            if k <= 0.0 {
                continue;
            }
            let y = a.min.y + 18.0 + i as f32 * 22.0;
            self.line(pos2(a.min.x, y), pos2(a.min.x + a.width() * k, y), self.ink(), 1.5);
            self.line(pos2(a.min.x + 8.0, y + 8.0), pos2(a.min.x + 8.0 + (a.width() * 0.7 - 8.0) * k, y + 8.0), self.grey(), 1.5);
        }
    }

    /// A picture; its numbered label slides in underneath.
    fn ref_caption(&self) {
        let a = self.page_center();
        let pic = Rect::from_min_size(pos2(a.min.x + 8.0, a.min.y + 2.0), vec2(44.0, 30.0));
        self.picture(pic);
        let k = self.reveal(0.2, 0.6);
        if k > 0.0 {
            self.label(
                pos2(a.center().x, pic.max.y + 7.0 + (1.0 - k) * 6.0),
                "Figure 1",
                7.0,
                self.accent().linear_multiply(k),
                Align2::CENTER_CENTER,
            );
        }
        self.paragraph(Rect::from_min_max(pos2(a.min.x, pic.max.y + 14.0), a.max), Dir::Ltr, 3, self.grey());
    }

    /// A table of figures on the centred page; returns its text area. When `update`, the rows
    /// are there and a new figure's page numbers change; otherwise they land one by one.
    fn ref_figures_page(&self, update: bool) -> Rect {
        let a = self.page_center();
        self.label(pos2(a.min.x, a.min.y + 4.0), "Figures", 7.5, self.accent(), Align2::LEFT_CENTER);
        let rows = [("Fig. 1", "2", "2"), ("Fig. 2", "3", "3"), ("Fig. 3", "4", "5"), ("Fig. 4", "7", "8"), ("Fig. 5", "8", "9")];
        let swap = self.reveal(0.45, 0.6);
        for (i, (name, old, new)) in rows.iter().enumerate() {
            let y = a.min.y + 18.0 + i as f32 * 12.0;
            if update {
                let changed = old != new;
                let num = if swap >= 1.0 || !changed { new } else { old };
                let c = if changed && swap >= 1.0 && self.phase < 1.0 { self.accent() } else { self.grey() };
                self.label(pos2(a.min.x, y), name, 6.0, self.ink(), Align2::LEFT_CENTER);
                self.ref_leader_entry(a, y, 22.0, 12.0, num, c);
            } else {
                let k = self.ref_step(i, rows.len());
                if k > 0.0 {
                    self.label(pos2(a.min.x, y), name, 6.0, self.ink().linear_multiply(k), Align2::LEFT_CENTER);
                    self.ref_leader_entry(a, y, 22.0, 12.0 * k, new, self.grey().linear_multiply(k));
                }
            }
        }
        a
    }

    /// A selected word; a tag chip drops down onto it: an index entry or a legal citation.
    fn ref_mark(&self, legal: bool) {
        let a = self.page_center();
        self.paragraph(a, Dir::Ltr, 7, self.grey());
        let y = a.min.y + 4.0 + 30.0;
        let w = Rect::from_min_size(pos2(a.min.x + 14.0, y - 4.0), vec2(if legal { 28.0 } else { 20.0 }, 8.0));
        self.selection(w);
        self.line(pos2(w.min.x + 1.0, y), pos2(w.max.x - 1.0, y), self.ink(), 1.5);
        let k = self.reveal(0.15, 0.55);
        if k <= 0.0 {
            return;
        }
        let c = pos2(w.center().x + 16.0, y - 18.0 - (1.0 - k) * 16.0);
        self.line(w.center_top(), c + vec2(0.0, 7.0), self.accent().linear_multiply(k), 1.0);
        let (text, cw) = if legal { ("§ Case", 36.0) } else { ("Index", 32.0) };
        let r = Rect::from_center_size(c, vec2(cw, 12.0));
        self.p.rect_filled(r, 3.0, self.accent().linear_multiply(k.max(0.4)));
        self.label(c, text, 6.5, Color32::WHITE, Align2::CENTER_CENTER);
    }

    /// An index in two columns on the centred page; returns its text area. When `update`, the
    /// rows are there and two page numbers change; otherwise they land one by one.
    fn ref_index_page(&self, update: bool) -> Rect {
        let a = self.page_center();
        let cw = (a.width() - 6.0) / 2.0;
        let nums = [("3", "3"), ("7", "7"), ("12", "13"), ("4", "4"), ("9", "10")];
        let swap = self.reveal(0.45, 0.6);
        for (col, letter) in ["A", "B"].iter().enumerate() {
            let x = a.min.x + col as f32 * (cw + 6.0);
            let colr = Rect::from_min_size(pos2(x, a.min.y), vec2(cw, a.height()));
            self.label(pos2(x, a.min.y + 4.0), letter, 7.0, self.accent(), Align2::LEFT_CENTER);
            for (i, (old, new)) in nums.iter().enumerate() {
                let y = a.min.y + 15.0 + i as f32 * 11.0;
                let indent = if i % 3 == 2 { 4.0 } else { 0.0 };
                if update {
                    let changed = old != new && col == 1;
                    let num = if swap >= 1.0 || !changed { new } else { old };
                    let c = if changed && swap >= 1.0 && self.phase < 1.0 { self.accent() } else { self.ink() };
                    self.ref_leader_entry(colr, y, indent, 10.0 - indent, num, c);
                } else {
                    let k = self.ref_step(col * nums.len() + i, 2 * nums.len());
                    if k > 0.0 {
                        self.ref_leader_entry(colr, y, indent, (10.0 - indent) * k, new, self.ink().linear_multiply(k));
                    }
                }
            }
        }
        a
    }

    /// Legal sources grouped under "Cases" and "Statutes", appearing one by one.
    fn ref_authorities(&self) {
        let a = self.page_center();
        let mut y = a.min.y + 4.0;
        let mut row = 0;
        for (head, nums) in [("Cases", ["2", "5", "6"]), ("Statutes", ["9", "11", "11"])] {
            self.label(pos2(a.min.x, y), head, 7.0, self.accent(), Align2::LEFT_CENTER);
            y += 10.0;
            for num in nums {
                let k = self.ref_step(row, 6);
                if k > 0.0 {
                    self.ref_leader_entry(a, y, 4.0, 22.0 * k, num, self.ink().linear_multiply(k));
                }
                y += 9.0;
                row += 1;
            }
            y += 4.0;
        }
    }
}
