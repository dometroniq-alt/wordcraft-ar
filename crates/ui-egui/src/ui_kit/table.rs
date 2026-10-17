//! Table Design and Table Layout tabs thumbnails: one small table per scene, with the row,
//! column or cell the button touches in the accent colour (or selected, or crossed out in red).
//! Each one a little film: the shading fills in, the border lines draw themselves, the pencil
//! draws the table, the columns shrink to their contents, the sum appears under its numbers.

use egui::{Align2, Color32, Rect, Stroke, pos2, vec2};

use super::Entry;
use super::draw::Draw;

pub const ENTRIES: &[Entry] = &[
    // Table Design.
    Entry::new(
        "table.shading",
        "Colours the background of the selected cells, to mark a row or make a heading stand out. Example: shade the totals row light grey.",
        |d| d.tb_shading(),
    ),
    Entry::new(
        "table.borders",
        "Chooses which edges of the selected cells get a line and which do not: all of them, the outside only, the inside only, none, the top or the bottom. Example: outside borders only, for a clean frame.",
        |d| d.tb_borders("all"),
    ),
    Entry::new(
        "table.borders:all",
        "Draws a line on every edge of the selected cells, around the outside and between the cells. Example: a full grid with every cell boxed.",
        |d| d.tb_borders("all"),
    ),
    Entry::new(
        "table.borders:outside",
        "Draws a line around the outside of the selected cells only; the lines between the cells are removed. Example: one frame around the whole table, nothing inside.",
        |d| d.tb_borders("outside"),
    ),
    Entry::new(
        "table.borders:inside",
        "Draws the lines between the selected cells only, with no frame around the outside. Example: rows and columns separated, but no outer box.",
        |d| d.tb_borders("inside"),
    ),
    Entry::new(
        "table.borders:none",
        "Removes every line from the selected cells; the text stays where it is. Example: a layout table that prints with no lines at all.",
        |d| d.tb_borders("none"),
    ),
    Entry::new("table.borders:top", "Draws a line along the top edge of the selected cells only. Example: a line above the totals row.", |d| {
        d.tb_borders("top")
    }),
    Entry::new("table.borders:bottom", "Draws a line along the bottom edge of the selected cells only. Example: a line under the header row.", |d| {
        d.tb_borders("bottom")
    }),
    Entry::new(
        "table.borderPainter",
        "Paints a line onto any cell edge you click or drag along with the mouse, in the chosen line style. Example: drag along the bottom of the header row to make that line thick.",
        |d| d.tb_border_painter(),
    ),
    Entry::new(
        "ui.dialog:newTableStyle",
        "Opens a box where you make your own table style (its lines, shading and header look) to reuse on other tables. Example: save «جدول الواجبات» with a grey header and thin lines.",
        |d| d.tb_new_table_style(),
    ),
    // Table Layout.
    Entry::new(
        "table.selectCell",
        "Selects the whole cell the caret is in, so you can format it or delete it. Example: select one cell and shade it.",
        |d| d.tb_select(0),
    ),
    Entry::new(
        "table.selectColumn",
        "Selects the whole column the caret is in, from the top of the table to the bottom. Example: select the «الرتبة» column and make it bold.",
        |d| d.tb_select(1),
    ),
    Entry::new(
        "table.selectRow",
        "Selects the whole row the caret is in, from its first cell to its last. Example: select the totals row and shade it.",
        |d| d.tb_select(2),
    ),
    Entry::new(
        "table.selectTable",
        "Selects the whole table, every row and every column. Example: select the table and centre it on the page.",
        |d| d.tb_select(3),
    ),
    Entry::new(
        "table.viewGridlines",
        "Shows faint dotted lines around cells that have no borders, on screen only, so you can see where the cells are; they do not print. Example: a table with no lines still shows its cells.",
        |d| d.tb_view_gridlines(),
    ),
    Entry::new(
        "table.properties",
        "Opens a box to set the table's width, where it sits on the page, the height of its rows and the width of its cells. Example: make the table 15 cm wide and centred.",
        |d| d.tb_table_properties(),
    ),
    Entry::new(
        "table.draw",
        "Lets you draw a table with the mouse: first a box for the table, then lines inside it for the cells. Example: draw a box, then two lines across it to make three rows.",
        |d| d.tb_draw_table(),
    ),
    Entry::new(
        "table.eraser",
        "Rubs out a line between two cells so they become one cell. Example: erase the line between two header cells to make one wide header.",
        |d| d.tb_eraser(),
    ),
    Entry::new(
        "table.deleteCells",
        "Removes the cell the caret is in; the cells beside it shift to close the gap. Example: delete one empty cell in the middle of a row.",
        |d| d.tb_delete(0),
    ),
    Entry::new(
        "table.deleteColumn",
        "Removes the whole column the caret is in; the other columns close up. Example: delete the unused «ملاحظات» column.",
        |d| d.tb_delete(1),
    ),
    Entry::new(
        "table.deleteRow",
        "Removes the whole row the caret is in; the rows below it move up. Example: delete the row of a person who has left.",
        |d| d.tb_delete(2),
    ),
    Entry::new(
        "table.deleteTable",
        "Removes the whole table and everything in it from the document. Example: the table goes and the paragraphs around it join up.",
        |d| d.tb_delete(3),
    ),
    Entry::new(
        "table.insertRowAbove",
        "Adds a new empty row just above the row the caret is in. Example: add a row above the first person to put a new name first.",
        |d| d.tb_insert_row(true),
    ),
    Entry::new(
        "table.insertRowBelow",
        "Adds a new empty row just below the row the caret is in. Example: add a row under the last person for one more name.",
        |d| d.tb_insert_row(false),
    ),
    Entry::new(
        "table.insertColumnLeft",
        "Adds a new empty column just to the left of the column the caret is in. Example: add a column for the rank beside the name.",
        |d| d.tb_insert_column(true),
    ),
    Entry::new(
        "table.insertColumnRight",
        "Adds a new empty column just to the right of the column the caret is in. Example: add a «ملاحظات» column at the end.",
        |d| d.tb_insert_column(false),
    ),
    Entry::new(
        "table.merge",
        "Joins the selected cells into one larger cell; their text joins too. Example: two header cells become one that spans both columns.",
        |d| d.tb_merge_cells(),
    ),
    Entry::new(
        "table.split",
        "Divides the cell at the caret into several smaller cells, as many across and down as you choose. Example: one cell becomes two side by side.",
        |d| d.tb_split_cells(),
    ),
    Entry::new(
        "table.splitTable",
        "Cuts the table in two at the row the caret is in, with an empty paragraph between the halves. Example: split a long table so a heading can go in the middle.",
        |d| d.tb_split_table(),
    ),
    Entry::new(
        "table.autofit",
        "Sizes the columns in one of three ways: to fit their contents, to fill the window, or to fixed widths that never change. Example: fit the contents so short columns stop wasting space.",
        |d| d.tb_autofit(),
    ),
    Entry::new(
        "table.autofit:contents",
        "Makes each column just wide enough for the longest text in it: short columns shrink, long ones grow. Example: a column of ranks narrows to the width of «نقيب».",
        |d| d.tb_autofit_mode(0),
    ),
    Entry::new(
        "table.autofit:window",
        "Stretches the table to the full width between the page margins, widening every column to fill it. Example: a narrow table grows to the whole line.",
        |d| d.tb_autofit_mode(1),
    ),
    Entry::new(
        "table.autofit:fixed",
        "Keeps every column at its set width whatever you type; long text wraps onto a second line inside the cell instead of widening it. Example: the «الاسم» column stays 4 cm wide.",
        |d| d.tb_autofit_mode(2),
    ),
    Entry::new("table.distributeRows", "Makes all the selected rows the same height. Example: three rows of different heights become equal.", |d| {
        d.tb_distribute_rows()
    }),
    Entry::new(
        "table.distributeColumns",
        "Makes all the selected columns the same width. Example: a wide column and two narrow ones become three equal ones.",
        |d| d.tb_distribute_columns(),
    ),
    Entry::new(
        "table.sort",
        "Puts the rows in order by one column: A to Z, or the smallest number first. Example: sort by the names column so «أحمد» comes before «باسم».",
        |d| d.tb_sort_rows(),
    ),
    Entry::new(
        "table.repeatHeader",
        "Repeats the header row at the top of every page the table runs onto, so the columns are always labelled. Example: page 2 of the table starts with the same «الاسم | الرتبة» row.",
        |d| d.tb_repeat_header(),
    ),
    Entry::new(
        "table.toText",
        "Turns the table back into ordinary paragraphs, one line per row. Example: a three-row table becomes three lines of text.",
        |d| d.tb_to_text(),
    ),
    Entry::new(
        "table.formula",
        "Puts a calculation in the cell, such as the sum of the numbers above it, and it updates itself when the numbers change. Example: under ٤، ٧ and ٢ the formula shows ١٣.",
        |d| d.tb_formula(),
    ),
];

/// The thumbnail's one centred table.
const TABLE: (f32, f32, f32, f32) = (38.0, 20.0, 100.0, 64.0);

impl Draw<'_> {
    fn tb_table_rect(&self) -> Rect {
        Rect::from_min_size(self.at(TABLE.0, TABLE.1), vec2(TABLE.2, TABLE.3))
    }

    /// How far item `i` of `n` has landed in the film: 0 before its turn, 1 once it is there
    /// (always 1 in a still drawing).
    fn tb_step(&self, i: usize, n: usize) -> f32 {
        let n = n.max(1) as f32;
        let from = 0.1 + 0.6 * i as f32 / n;
        self.reveal(from, from + 0.15)
    }

    /// A table inside a before/after page area.
    fn tb_area_table(&self, area: Rect, rows: usize, cols: usize) -> Rect {
        let r = Rect::from_min_size(area.min + vec2(0.0, 8.0), vec2(area.width(), 54.0));
        self.table(r, rows, cols);
        r
    }

    /// Fills cell `(row, col)` in `c` and redraws its text line on top.
    fn tb_mark_cell(&self, r: Rect, rows: usize, cols: usize, row: usize, col: usize, c: Color32) {
        let cell = self.cell(r, rows, cols, row, col);
        self.p.rect_filled(cell.shrink(0.5), 0.0, c);
        let y = cell.center().y;
        self.line(pos2(cell.min.x + 3.0, y), pos2(cell.min.x + cell.width() * 0.6, y), if row == 0 { self.ink() } else { self.grey() }, 1.5);
    }

    /// Fills a whole row (or column) in the accent tint and redraws its text.
    fn tb_mark_row(&self, r: Rect, rows: usize, cols: usize, row: usize) {
        for col in 0..cols {
            self.tb_mark_cell(r, rows, cols, row, col, self.accent().linear_multiply(0.3));
        }
    }
    fn tb_mark_col(&self, r: Rect, rows: usize, cols: usize, col: usize) {
        for row in 0..rows {
            self.tb_mark_cell(r, rows, cols, row, col, self.accent().linear_multiply(0.3));
        }
    }

    /// Table lines at the given fractions of the width (`xs`) and height (`ys`), with a text
    /// line in each cell: for tables with uneven rows or columns.
    fn tb_table_uneven(&self, r: Rect, ys: &[f32], xs: &[f32]) {
        self.p.rect_filled(r, 1.0, Color32::WHITE);
        self.p.rect_stroke(r, 1.0, self.stroke(self.t.border_strong), egui::StrokeKind::Inside);
        for y in ys {
            let y = r.min.y + r.height() * y;
            self.line(pos2(r.min.x, y), pos2(r.max.x, y), self.t.border, 1.0);
        }
        for x in xs {
            let x = r.min.x + r.width() * x;
            self.line(pos2(x, r.min.y), pos2(x, r.max.y), self.t.border, 1.0);
        }
        let mut y_edges = vec![0.0];
        y_edges.extend_from_slice(ys);
        y_edges.push(1.0);
        let mut x_edges = vec![0.0];
        x_edges.extend_from_slice(xs);
        x_edges.push(1.0);
        for yw in y_edges.windows(2) {
            for xw in x_edges.windows(2) {
                if let ([y0, y1], [x0, x1]) = (yw, xw) {
                    let y = r.min.y + r.height() * (y0 + y1) / 2.0;
                    let x = r.min.x + r.width() * x0 + 3.0;
                    let w = r.width() * (x1 - x0) * 0.6;
                    self.line(pos2(x, y), pos2(x + w, y), self.grey(), 1.5);
                }
            }
        }
    }

    /// A table whose column edges are at the `xs` (absolute x of each edge, first to last)
    /// and whose cells hold text of fixed widths `text` (pt, per column), 3 rows; the text
    /// is clipped to its cell, so a narrow column shows it cut off.
    fn tb_table_cols(&self, y0: f32, h: f32, xs: &[f32], text: &[f32]) -> Rect {
        let (Some(&x0), Some(&x1)) = (xs.first(), xs.last()) else { return Rect::NOTHING };
        let r = Rect::from_min_max(pos2(x0, y0), pos2(x1, y0 + h));
        self.p.rect_filled(r, 1.0, Color32::WHITE);
        self.p.rect_filled(Rect::from_min_size(r.min, vec2(r.width(), h / 3.0)), 0.0, self.t.hover);
        self.p.rect_stroke(r, 1.0, self.stroke(self.t.border_strong), egui::StrokeKind::Inside);
        for i in 1..3 {
            let y = r.min.y + i as f32 * h / 3.0;
            self.line(pos2(r.min.x, y), pos2(r.max.x, y), self.t.border, 1.0);
        }
        for x in xs.iter().skip(1).take(xs.len().saturating_sub(2)) {
            self.line(pos2(*x, r.min.y), pos2(*x, r.max.y), self.t.border, 1.0);
        }
        for (j, w) in xs.windows(2).zip(text.iter()) {
            if let [a, b] = j {
                let clipped = self.p.with_clip_rect(Rect::from_min_max(pos2(*a, r.min.y), pos2(*b - 2.0, r.max.y)).intersect(self.p.clip_rect()));
                let d = self.with_painter(&clipped);
                for i in 0..3 {
                    let y = r.min.y + (i as f32 + 0.5) * h / 3.0;
                    let w = if i == 0 { w * 0.7 } else { *w };
                    d.line(pos2(a + 3.0, y), pos2(a + 3.0 + w, y), if i == 0 { d.ink() } else { d.grey() }, 1.5);
                }
            }
        }
        r
    }

    /// The shading fills the two cells.
    fn tb_shading(&self) {
        let r = self.tb_table_rect();
        self.table(r, 3, 3);
        let k = self.reveal(0.2, 0.6);
        if k > 0.0 {
            self.tb_mark_cell(r, 3, 3, 1, 1, self.accent().linear_multiply(0.45 * k));
            self.tb_mark_cell(r, 3, 3, 1, 2, self.accent().linear_multiply(0.45 * k));
        }
    }

    /// The border lines of one `kind` draw themselves over the table: all, outside, inside,
    /// top, bottom — or, for `none`, the table's own lines fade away.
    fn tb_borders(&self, kind: &str) {
        let r = self.tb_table_rect();
        let k = self.reveal(0.15, 0.6);
        if kind == "none" {
            // White cells with their text; the lines fade out.
            self.p.rect_filled(r, 1.0, Color32::WHITE);
            let fade = self.t.border.linear_multiply(1.0 - k);
            self.grid(r, 3, 3, fade);
            for row in 0..3 {
                for col in 0..3 {
                    let cell = self.cell(r, 3, 3, row, col);
                    let y = cell.center().y;
                    self.line(
                        pos2(cell.min.x + 3.0, y),
                        pos2(cell.min.x + cell.width() * 0.6, y),
                        if row == 0 { self.ink() } else { self.grey() },
                        1.5,
                    );
                }
            }
            return;
        }
        self.table(r, 3, 3);
        if k <= 0.0 {
            return;
        }
        let c = self.accent();
        let w = 2.5;
        let outside = matches!(kind, "all" | "outside");
        let inside = matches!(kind, "all" | "inside");
        if outside || kind == "top" {
            self.line(pos2(r.min.x, r.min.y + 1.0), pos2(r.min.x + r.width() * k, r.min.y + 1.0), c, w);
        }
        if outside || kind == "bottom" {
            self.line(pos2(r.min.x, r.max.y - 1.0), pos2(r.min.x + r.width() * k, r.max.y - 1.0), c, w);
        }
        if outside {
            self.line(pos2(r.min.x + 1.0, r.min.y), pos2(r.min.x + 1.0, r.min.y + r.height() * k), c, w);
            self.line(pos2(r.max.x - 1.0, r.min.y), pos2(r.max.x - 1.0, r.min.y + r.height() * k), c, w);
        }
        if inside {
            for i in 1..3 {
                let y = r.min.y + i as f32 * r.height() / 3.0;
                self.line(pos2(r.min.x, y), pos2(r.min.x + r.width() * k, y), c, 1.5);
                let x = r.min.x + i as f32 * r.width() / 3.0;
                self.line(pos2(x, r.min.y), pos2(x, r.min.y + r.height() * k), c, 1.5);
            }
        }
    }

    /// The pencil drags along the edge between two rows, leaving a thick line behind it.
    fn tb_border_painter(&self) {
        let r = self.tb_table_rect();
        self.table(r, 3, 3);
        let y = r.min.y + r.height() * 2.0 / 3.0;
        let k = self.reveal(0.1, 0.7);
        let x1 = r.min.x + r.width() * 2.0 / 3.0 * k;
        if k > 0.0 {
            self.line(pos2(r.min.x, y), pos2(x1, y), self.accent(), 3.0);
        }
        self.pencil(pos2(x1, y), pos2(x1 + 22.0, y - 26.0));
    }

    /// The New Table Style box: a preview table whose header shading and lines come in.
    fn tb_new_table_style(&self) {
        let inner = self.dialog("Table Style", 0);
        let r = Rect::from_min_size(inner.min + vec2(0.0, 2.0), vec2(inner.width(), 38.0));
        self.table(r, 3, 3);
        let k = self.reveal(0.2, 0.5);
        if k > 0.0 {
            for col in 0..3 {
                self.tb_mark_cell(r, 3, 3, 0, col, self.accent().linear_multiply(0.45 * k));
            }
        }
        let k2 = self.reveal(0.5, 0.8);
        if k2 > 0.0 {
            self.p.rect_stroke(r, 1.0, Stroke::new(2.0, self.accent().linear_multiply(k2)), egui::StrokeKind::Inside);
        }
        self.button(pos2(inner.max.x - 16.0, inner.max.y - 6.0), "OK", false);
    }

    /// The selection spreads over the cell (0), the column (1), the row (2) or the whole
    /// table (3) under the pointer.
    fn tb_select(&self, what: u8) {
        let r = self.tb_table_rect();
        self.table(r, 3, 3);
        let k = self.reveal(0.15, 0.5);
        let sel = self.t.selection.linear_multiply(k.max(0.001));
        if k > 0.0 {
            match what {
                0 => self.tb_mark_cell(r, 3, 3, 1, 1, sel),
                1 => (0..3).for_each(|row| self.tb_mark_cell(r, 3, 3, row, 1, sel)),
                2 => (0..3).for_each(|col| self.tb_mark_cell(r, 3, 3, 1, col, sel)),
                _ => {
                    for row in 0..3 {
                        for col in 0..3 {
                            self.tb_mark_cell(r, 3, 3, row, col, sel);
                        }
                    }
                }
            }
        }
        let tip = match what {
            0 => self.cell(r, 3, 3, 1, 1).center() + vec2(4.0, -2.0),
            1 => self.cell(r, 3, 3, 0, 1).center_top() + vec2(0.0, -2.0),
            2 => self.cell(r, 3, 3, 1, 0).left_center() + vec2(-3.0, 0.0),
            _ => r.left_top() + vec2(-4.0, -4.0),
        };
        self.pointer(tip);
    }

    /// The dotted gridlines fade in around the borderless cells.
    fn tb_view_gridlines(&self) {
        let r = self.tb_table_rect();
        self.p.rect_filled(r, 1.0, Color32::WHITE);
        let k = self.reveal(0.2, 0.6);
        for row in 0..3 {
            for col in 0..3 {
                let cell = self.cell(r, 3, 3, row, col);
                if k > 0.0 {
                    self.dashed_rect(cell.shrink(0.5), self.accent().linear_multiply(k));
                }
                let y = cell.center().y;
                self.line(pos2(cell.min.x + 3.0, y), pos2(cell.min.x + cell.width() * 0.6, y), self.grey(), 1.5);
            }
        }
    }

    fn tb_table_properties(&self) {
        self.dialog("Table", 3);
    }

    /// The pencil draws the box, then the column line, then the row line.
    fn tb_draw_table(&self) {
        let r = self.tb_table_rect();
        let k1 = self.reveal(0.0, 0.3);
        let k2 = self.reveal(0.3, 0.5);
        let k3 = self.reveal(0.5, 0.85);
        self.p.rect_filled(r, 1.0, Color32::WHITE);
        self.p.rect_stroke(r, 1.0, Stroke::new(1.5, self.accent().linear_multiply(k1.max(0.15))), egui::StrokeKind::Inside);
        if k2 > 0.0 {
            self.line(pos2(r.center().x, r.min.y), pos2(r.center().x, r.min.y + r.height() * k2), self.accent(), 1.5);
        }
        let y = r.center().y;
        let x_end = r.min.x + r.width() * 0.7 * k3;
        if k3 > 0.0 {
            self.line(pos2(r.min.x, y), pos2(x_end, y), self.accent(), 1.5);
        }
        let tip = if k3 > 0.0 {
            pos2(x_end, y)
        } else if k2 > 0.0 {
            pos2(r.center().x, r.min.y + r.height() * k2)
        } else {
            pos2(r.min.x + r.width() * k1, r.min.y)
        };
        self.pencil(tip, tip + vec2(22.0, -26.0));
    }

    /// The eraser slides down the border between two cells and the line goes.
    fn tb_eraser(&self) {
        let r = self.tb_table_rect();
        self.table(r, 3, 3);
        let x = r.min.x + r.width() / 3.0;
        let y0 = r.min.y + r.height() / 3.0;
        let y1 = r.min.y + r.height() * 2.0 / 3.0;
        let k = self.reveal(0.15, 0.6);
        if k > 0.0 {
            self.line(pos2(x, y0 + 1.0), pos2(x, y0 + 1.0 + (y1 - y0 - 2.0) * k), Color32::WHITE, 2.0);
        }
        // The eraser: a small slanted block, following the spot it has rubbed to.
        let e = Rect::from_center_size(pos2(x + 6.0, y0 + 6.0 + (y1 - y0 - 12.0) * k), vec2(18.0, 10.0));
        self.p.rect_filled(e, 2.0, self.accent());
        self.p.rect_filled(Rect::from_min_size(e.min, vec2(7.0, e.height())), 2.0, self.accent().linear_multiply(0.5));
    }

    /// The cell (0), the column (1), the row (2) or the whole table (3) is tinted red and
    /// crossed out.
    fn tb_delete(&self, what: u8) {
        let r = self.tb_table_rect();
        self.table(r, 3, 3);
        let tint = self.red().linear_multiply(0.25);
        let band = match what {
            0 => {
                self.tb_mark_cell(r, 3, 3, 1, 1, tint);
                self.cell(r, 3, 3, 1, 1)
            }
            1 => {
                (0..3).for_each(|row| self.tb_mark_cell(r, 3, 3, row, 1, tint));
                self.cell(r, 3, 3, 0, 1).union(self.cell(r, 3, 3, 2, 1))
            }
            2 => {
                (0..3).for_each(|col| self.tb_mark_cell(r, 3, 3, 1, col, tint));
                self.cell(r, 3, 3, 1, 0).union(self.cell(r, 3, 3, 1, 2))
            }
            _ => {
                for row in 0..3 {
                    for col in 0..3 {
                        self.tb_mark_cell(r, 3, 3, row, col, tint);
                    }
                }
                r
            }
        };
        let k = self.reveal(0.2, 0.6);
        if k > 0.0 {
            self.line(band.left_top(), band.left_top() + (band.right_bottom() - band.left_top()) * k, self.red(), 2.0);
        }
        let k2 = self.reveal(0.45, 0.8);
        if k2 > 0.0 {
            self.line(band.left_bottom(), band.left_bottom() + (band.right_top() - band.left_bottom()) * k2, self.red(), 2.0);
        }
    }

    fn tb_insert_row(&self, above: bool) {
        self.before_after(
            |d, a| {
                let r = d.tb_area_table(a, 3, 2);
                d.tb_mark_cell(r, 3, 2, 1, 0, d.t.selection);
            },
            |d, a| {
                let r = d.tb_area_table(a, 4, 2);
                let new = if above { 1 } else { 2 };
                d.tb_mark_row(r, 4, 2, new);
                let cell = d.cell(r, 4, 2, new, 0);
                let x = r.min.x - 6.0;
                if above {
                    d.arrow_up(pos2(x, cell.max.y + 4.0), pos2(x, cell.min.y - 2.0));
                } else {
                    d.arrow_down(pos2(x, cell.min.y - 4.0), pos2(x, cell.max.y + 2.0));
                }
            },
        );
    }

    fn tb_insert_column(&self, left: bool) {
        self.before_after(
            |d, a| {
                let r = d.tb_area_table(a, 3, 2);
                d.tb_mark_cell(r, 3, 2, 1, 1, d.t.selection);
            },
            |d, a| {
                let r = d.tb_area_table(a, 3, 3);
                let new = if left { 1 } else { 2 };
                d.tb_mark_col(r, 3, 3, new);
                let cell = d.cell(r, 3, 3, 0, new);
                let y = r.min.y - 5.0;
                if left {
                    d.arrow_left(pos2(cell.max.x + 4.0, y), pos2(cell.min.x - 2.0, y));
                } else {
                    d.arrow_right(pos2(cell.min.x - 4.0, y), pos2(cell.max.x + 2.0, y));
                }
            },
        );
    }

    fn tb_merge_cells(&self) {
        self.before_after(
            |d, a| {
                let r = d.tb_area_table(a, 3, 3);
                d.tb_mark_cell(r, 3, 3, 1, 0, d.t.selection);
                d.tb_mark_cell(r, 3, 3, 1, 1, d.t.selection);
            },
            |d, a| {
                let r = d.tb_area_table(a, 3, 3);
                let c0 = d.cell(r, 3, 3, 1, 0);
                let c1 = d.cell(r, 3, 3, 1, 1);
                let merged = c0.union(c1).shrink(0.5);
                d.p.rect_filled(merged, 0.0, d.accent().linear_multiply(0.3));
                let y = merged.center().y;
                d.line(pos2(merged.min.x + 3.0, y), pos2(merged.max.x - 3.0, y), d.grey(), 1.5);
            },
        );
    }

    fn tb_split_cells(&self) {
        self.before_after(
            |d, a| {
                let r = d.tb_area_table(a, 3, 2);
                d.tb_mark_cell(r, 3, 2, 1, 0, d.t.selection);
            },
            |d, a| {
                let r = d.tb_area_table(a, 3, 2);
                let cell = d.cell(r, 3, 2, 1, 0);
                d.p.rect_filled(cell.shrink(0.5), 0.0, d.accent().linear_multiply(0.3));
                let (x, y) = (cell.center().x, cell.center().y);
                d.line(pos2(x, cell.min.y), pos2(x, cell.max.y), d.accent(), 1.5);
                d.line(pos2(cell.min.x, y), pos2(cell.max.x, y), d.accent(), 1.5);
                for (cx, cy) in [(cell.min.x, cell.min.y), (x, cell.min.y), (cell.min.x, y), (x, y)] {
                    d.line(pos2(cx + 2.0, cy + cell.height() / 4.0), pos2(cx + cell.width() / 2.0 - 4.0, cy + cell.height() / 4.0), d.grey(), 1.0);
                }
            },
        );
    }

    fn tb_split_table(&self) {
        self.before_after(
            |d, a| {
                let r = Rect::from_min_size(a.min + vec2(0.0, 6.0), vec2(a.width(), 60.0));
                d.table(r, 4, 2);
                d.tb_mark_cell(r, 4, 2, 2, 0, d.t.selection);
            },
            |d, a| {
                let top = Rect::from_min_size(a.min + vec2(0.0, 2.0), vec2(a.width(), 30.0));
                d.table(top, 2, 2);
                let y = top.max.y + 8.0;
                d.line(pos2(a.min.x, y), pos2(a.min.x + a.width() * 0.5, y), d.accent(), 1.5);
                let bottom = Rect::from_min_size(pos2(a.min.x, y + 8.0), vec2(a.width(), 30.0));
                d.table(bottom, 2, 2);
            },
        );
    }

    /// The uneven table with a width arrow over each column; the arrows grow in.
    fn tb_autofit(&self) {
        let r = self.tb_table_rect();
        self.tb_table_uneven(r, &[0.34, 0.67], &[0.2, 0.65]);
        let y = r.min.y - 7.0;
        for (i, (x0, x1)) in [(0.0, 0.2), (0.2, 0.65), (0.65, 1.0)].into_iter().enumerate() {
            let k = self.tb_step(i, 3);
            if k > 0.0 {
                let (a, b) = (r.min.x + r.width() * x0 + 1.0, r.min.x + r.width() * x1 - 1.0);
                let mid = (a + b) / 2.0;
                self.measure_h(pos2(mid - (mid - a) * k, y), pos2(mid + (b - mid) * k, y));
            }
        }
    }

    /// One AutoFit mode on a page: the columns shrink to their contents (0), the table
    /// stretches to the margins (1), or the widths stay fixed and the long text wraps (2).
    fn tb_autofit_mode(&self, mode: u8) {
        let a = self.page_wide();
        let k = self.reveal(0.2, 0.7);
        let y0 = a.min.y + 12.0;
        let h = 48.0;
        let text = [12.0, 40.0, 18.0];
        match mode {
            0 => {
                // Equal columns across the page become columns just wide enough for the text.
                let full = a.width();
                let from = [0.0, full / 3.0, 2.0 * full / 3.0, full];
                let fit = [0.0, text[0] + 8.0, text[0] + text[1] + 16.0, text[0] + text[1] + text[2] + 24.0];
                let xs: Vec<f32> = from.iter().zip(fit.iter()).map(|(f, t)| a.min.x + f + (t - f) * k).collect();
                let r = self.tb_table_cols(y0, h, &xs, &text);
                if k >= 1.0 {
                    for w in xs.windows(2) {
                        if let [x0, x1] = w {
                            self.measure_h(pos2(x0 + 1.0, r.min.y - 7.0), pos2(x1 - 1.0, r.min.y - 7.0));
                        }
                    }
                }
            }
            1 => {
                // A narrow table grows to the full width between the margins.
                let narrow = a.width() * 0.55;
                let w = narrow + (a.width() - narrow) * k;
                let xs = [a.min.x, a.min.x + w * 0.25, a.min.x + w * 0.65, a.min.x + w];
                let r = self.tb_table_cols(y0, h, &xs, &text);
                self.measure_h(pos2(a.min.x + 1.0, r.min.y - 7.0), pos2(a.min.x + w - 1.0, r.min.y - 7.0));
                // The margins the table reaches for.
                for x in [a.min.x, a.max.x] {
                    self.line(pos2(x, a.min.y), pos2(x, a.max.y), self.accent().linear_multiply(0.4), 1.0);
                }
            }
            _ => {
                // Fixed, equal columns; the long text of one cell wraps onto a second line
                // instead of widening its column.
                let full = a.width();
                let xs = [a.min.x, a.min.x + full / 3.0, a.min.x + 2.0 * full / 3.0, a.max.x];
                let r = self.tb_table_cols(y0, h, &xs, &[12.0, 24.0, 18.0]);
                let cell = Rect::from_min_max(pos2(xs[1], r.min.y + h / 3.0), pos2(xs[2], r.min.y + 2.0 * h / 3.0));
                let y = cell.center().y;
                let typed = 24.0 + 20.0 * k;
                let first = typed.min(cell.width() - 6.0);
                self.line(pos2(cell.min.x + 3.0, y - 3.0), pos2(cell.min.x + 3.0 + first, y - 3.0), self.accent(), 1.5);
                let rest = (typed - first).max(0.0);
                if rest > 0.0 {
                    self.line(pos2(cell.min.x + 3.0, y + 3.0), pos2(cell.min.x + 3.0 + rest, y + 3.0), self.accent(), 1.5);
                }
                for w in xs.windows(2) {
                    if let [x0, x1] = w {
                        self.measure_h(pos2(x0 + 1.0, r.min.y - 7.0), pos2(x1 - 1.0, r.min.y - 7.0));
                    }
                }
            }
        }
    }

    fn tb_distribute_rows(&self) {
        self.before_after(
            |d, a| {
                let r = Rect::from_min_size(a.min + vec2(0.0, 8.0), vec2(a.width(), 54.0));
                d.tb_table_uneven(r, &[0.2, 0.75], &[0.5]);
            },
            |d, a| {
                let r = Rect::from_min_size(a.min + vec2(0.0, 8.0), vec2(a.width(), 54.0));
                d.tb_table_uneven(r, &[1.0 / 3.0, 2.0 / 3.0], &[0.5]);
                let x = r.max.x + 4.0;
                for i in 0..3 {
                    let y0 = r.min.y + r.height() * i as f32 / 3.0;
                    d.measure_v(pos2(x, y0 + 1.0), pos2(x, y0 + r.height() / 3.0 - 1.0));
                }
            },
        );
    }

    fn tb_distribute_columns(&self) {
        self.before_after(
            |d, a| {
                let r = Rect::from_min_size(a.min + vec2(0.0, 8.0), vec2(a.width(), 54.0));
                d.tb_table_uneven(r, &[0.5], &[0.25, 0.8]);
            },
            |d, a| {
                let r = Rect::from_min_size(a.min + vec2(0.0, 8.0), vec2(a.width(), 54.0));
                d.tb_table_uneven(r, &[0.5], &[1.0 / 3.0, 2.0 / 3.0]);
                let y = r.min.y - 5.0;
                for i in 0..3 {
                    let x0 = r.min.x + r.width() * i as f32 / 3.0;
                    d.measure_h(pos2(x0 + 1.0, y), pos2(x0 + r.width() / 3.0 - 1.0, y));
                }
            },
        );
    }

    /// Rows labelled by their first cell: C, A, B → A, B, C.
    fn tb_labelled_rows(&self, r: Rect, labels: &[&str]) {
        let rows = labels.len().max(1);
        self.p.rect_filled(r, 1.0, Color32::WHITE);
        self.p.rect_stroke(r, 1.0, self.stroke(self.t.border_strong), egui::StrokeKind::Inside);
        let rh = r.height() / rows as f32;
        let cx = r.min.x + 16.0;
        self.line(pos2(cx, r.min.y), pos2(cx, r.max.y), self.t.border, 1.0);
        for (i, l) in labels.iter().enumerate() {
            let y = r.min.y + (i as f32 + 0.5) * rh;
            if i > 0 {
                self.line(pos2(r.min.x, y - rh / 2.0), pos2(r.max.x, y - rh / 2.0), self.t.border, 1.0);
            }
            self.label(pos2(r.min.x + 8.0, y), l, 8.0, self.accent(), Align2::CENTER_CENTER);
            self.line(pos2(cx + 3.0, y), pos2(r.max.x - 4.0, y), self.grey(), 1.5);
        }
    }

    fn tb_sort_rows(&self) {
        self.before_after(
            |d, a| {
                let r = Rect::from_min_size(a.min + vec2(0.0, 8.0), vec2(a.width(), 54.0));
                d.tb_labelled_rows(r, &["C", "A", "B"]);
            },
            |d, a| {
                let r = Rect::from_min_size(a.min + vec2(0.0, 8.0), vec2(a.width(), 54.0));
                d.tb_labelled_rows(r, &["A", "B", "C"]);
                d.arrow_down(pos2(r.max.x + 5.0, r.min.y + 4.0), pos2(r.max.x + 5.0, r.max.y - 4.0));
            },
        );
    }

    /// The table runs onto a second page; its header row appears there too.
    fn tb_repeat_header(&self) {
        for (i, x) in [10.0, 94.0].into_iter().enumerate() {
            let a = self.page(x, 6.0, 72.0, 92.0);
            let r = Rect::from_min_size(a.min, vec2(a.width(), 66.0));
            self.table(r, 4, 2);
            let k = if i == 0 { 1.0 } else { self.reveal(0.3, 0.7) };
            if k > 0.0 {
                for col in 0..2 {
                    self.tb_mark_cell(r, 4, 2, 0, col, self.accent().linear_multiply(0.3 * k));
                }
            }
        }
        let arrow = self.reveal(0.05, 0.3);
        if arrow > 0.0 {
            self.arrow_right(self.at(82.0, 100.0), self.at(82.0 + 12.0 * arrow, 100.0));
        }
        self.label(self.at(88.0, 92.0), "1 → 2", 7.0, self.grey(), Align2::CENTER_CENTER);
    }

    fn tb_to_text(&self) {
        self.before_after(
            |d, a| {
                d.tb_area_table(a, 3, 2);
            },
            |d, a| {
                d.text_lines(Rect::from_min_size(a.min + vec2(0.0, 10.0), a.size()), false, &[48.0, 40.0, 50.0, 36.0, 44.0, 30.0], 9.0, d.grey());
            },
        );
    }

    /// Numbers appear down the column one by one, then their sum in the last cell.
    fn tb_formula(&self) {
        let r = self.tb_table_rect();
        self.table(r, 4, 2);
        for (i, n) in ["٤", "٧", "٢"].iter().enumerate() {
            let cell = self.cell(r, 4, 2, i + 1, 1);
            self.p.rect_filled(cell.shrink(0.5), 0.0, Color32::WHITE);
            if self.tb_step(i, 4) > 0.0 {
                self.label(cell.center(), n, 8.0, self.grey(), Align2::CENTER_CENTER);
            }
        }
        let last = self.cell(r, 4, 2, 3, 1);
        self.p.rect_filled(last.shrink(0.5), 0.0, Color32::WHITE);
        let k = self.tb_step(3, 4);
        if k > 0.0 {
            self.p.rect_filled(last.shrink(0.5), 0.0, self.accent().linear_multiply(0.3 * k));
            self.label(last.center(), "Σ ١٣", 8.0, self.accent().linear_multiply(k), Align2::CENTER_CENTER);
        }
    }
}
