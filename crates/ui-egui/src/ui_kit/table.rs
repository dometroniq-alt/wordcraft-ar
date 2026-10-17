//! Table Design and Table Layout tabs thumbnails: one small table per scene, with the row,
//! column or cell the button touches in the accent colour (or selected, or crossed out in red).

use egui::{Align2, Color32, Rect, Stroke, pos2, vec2};

use super::Entry;
use super::draw::Draw;

pub const ENTRIES: &[Entry] = &[
    // Table Design.
    Entry::new("table.shading", "Fills the selected cells with a background colour.", |d| d.tb_shading()),
    Entry::new("table.borders", "Chooses which edges of the selected cells get lines, and how thick they are.", |d| d.tb_borders()),
    Entry::new("table.borderPainter", "Paints a line onto any cell edge you click or drag along.", |d| d.tb_border_painter()),
    // Table Layout.
    Entry::new("table.selectCell", "Selects the whole cell the caret is in, or its row, column or table.", |d| d.tb_select_cell()),
    Entry::new("table.viewGridlines", "Shows faint dotted lines around cells that have no borders, on screen only.", |d| d.tb_view_gridlines()),
    Entry::new("table.properties", "Opens a box to set the table's width, alignment, row heights and cell widths.", |d| d.tb_table_properties()),
    Entry::new("table.draw", "Lets you draw a table with the mouse: a box for the table, then lines for the cells.", |d| d.tb_draw_table()),
    Entry::new("table.eraser", "Rubs out a cell border so the two cells on either side become one.", |d| d.tb_eraser()),
    Entry::new("table.deleteCells", "Removes the cells, row, column or whole table the caret is in.", |d| d.tb_delete_cells()),
    Entry::new("table.insertRowAbove", "Adds a new empty row above the row the caret is in.", |d| d.tb_insert_row(true)),
    Entry::new("table.insertRowBelow", "Adds a new empty row below the row the caret is in.", |d| d.tb_insert_row(false)),
    Entry::new("table.insertColumnLeft", "Adds a new empty column to the left of the column the caret is in.", |d| d.tb_insert_column(true)),
    Entry::new("table.insertColumnRight", "Adds a new empty column to the right of the column the caret is in.", |d| d.tb_insert_column(false)),
    Entry::new("table.merge", "Joins the selected cells into one larger cell.", |d| d.tb_merge_cells()),
    Entry::new("table.split", "Divides the cell at the caret into several smaller cells.", |d| d.tb_split_cells()),
    Entry::new("table.splitTable", "Cuts the table in two at the caret's row, with a paragraph between the halves.", |d| d.tb_split_table()),
    Entry::new("table.autofit", "Sizes the columns to fit their contents, the window, or fixed widths.", |d| d.tb_autofit()),
    Entry::new("table.distributeRows", "Makes all the selected rows the same height.", |d| d.tb_distribute_rows()),
    Entry::new("table.distributeColumns", "Makes all the selected columns the same width.", |d| d.tb_distribute_columns()),
    Entry::new("table.sort", "Puts the rows in order by one column, A to Z or smallest to largest.", |d| d.tb_sort_rows()),
    Entry::new("table.repeatHeader", "Repeats the header row at the top of every page the table runs onto.", |d| d.tb_repeat_header()),
    Entry::new("table.toText", "Turns the table back into ordinary paragraphs, one line per row.", |d| d.tb_to_text()),
    Entry::new("table.formula", "Puts a calculation in the cell, such as the sum of the numbers above it.", |d| d.tb_formula()),
];

/// The thumbnail's one centred table.
const TABLE: (f32, f32, f32, f32) = (38.0, 20.0, 100.0, 64.0);

impl Draw<'_> {
    fn tb_table_rect(&self) -> Rect {
        Rect::from_min_size(self.at(TABLE.0, TABLE.1), vec2(TABLE.2, TABLE.3))
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

    fn tb_shading(&self) {
        let r = self.tb_table_rect();
        self.table(r, 3, 3);
        self.tb_mark_cell(r, 3, 3, 1, 1, self.accent().linear_multiply(0.45));
        self.tb_mark_cell(r, 3, 3, 1, 2, self.accent().linear_multiply(0.45));
    }

    fn tb_borders(&self) {
        let r = self.tb_table_rect();
        self.table(r, 3, 3);
        let c = self.accent();
        self.p.rect_stroke(r, 1.0, Stroke::new(2.5, c), egui::StrokeKind::Inside);
        for i in 1..3 {
            let y = r.min.y + i as f32 * r.height() / 3.0;
            self.line(pos2(r.min.x, y), pos2(r.max.x, y), c, 1.5);
            let x = r.min.x + i as f32 * r.width() / 3.0;
            self.line(pos2(x, r.min.y), pos2(x, r.max.y), c, 1.5);
        }
    }

    fn tb_border_painter(&self) {
        let r = self.tb_table_rect();
        self.table(r, 3, 3);
        // The edge being painted: between rows 1 and 2, across the first two columns.
        let y = r.min.y + r.height() * 2.0 / 3.0;
        let x1 = r.min.x + r.width() * 2.0 / 3.0;
        self.line(pos2(r.min.x, y), pos2(x1, y), self.accent(), 3.0);
        self.pencil(pos2(x1, y), pos2(x1 + 22.0, y - 26.0));
    }

    fn tb_select_cell(&self) {
        let r = self.tb_table_rect();
        self.table(r, 3, 3);
        self.tb_mark_cell(r, 3, 3, 1, 1, self.t.selection);
        let cell = self.cell(r, 3, 3, 1, 1);
        self.pointer(cell.center() + vec2(4.0, -2.0));
    }

    fn tb_view_gridlines(&self) {
        let r = self.tb_table_rect();
        self.p.rect_filled(r, 1.0, Color32::WHITE);
        for row in 0..3 {
            for col in 0..3 {
                let cell = self.cell(r, 3, 3, row, col);
                self.dashed_rect(cell.shrink(0.5), self.accent());
                let y = cell.center().y;
                self.line(pos2(cell.min.x + 3.0, y), pos2(cell.min.x + cell.width() * 0.6, y), self.grey(), 1.5);
            }
        }
    }

    fn tb_table_properties(&self) {
        self.dialog("Table", 3);
    }

    fn tb_draw_table(&self) {
        let r = self.tb_table_rect();
        // The outer box is drawn; one column line is finished, the row line is being drawn.
        self.p.rect_filled(r, 1.0, Color32::WHITE);
        self.p.rect_stroke(r, 1.0, Stroke::new(1.5, self.accent()), egui::StrokeKind::Inside);
        self.line(pos2(r.center().x, r.min.y), pos2(r.center().x, r.max.y), self.accent(), 1.5);
        let y = r.center().y;
        let x_end = r.min.x + r.width() * 0.7;
        self.line(pos2(r.min.x, y), pos2(x_end, y), self.accent(), 1.5);
        self.pencil(pos2(x_end, y), pos2(x_end + 22.0, y - 26.0));
    }

    fn tb_eraser(&self) {
        let r = self.tb_table_rect();
        self.table(r, 3, 3);
        // The border between the two cells of the middle row, left pair, is gone.
        let x = r.min.x + r.width() / 3.0;
        let y0 = r.min.y + r.height() / 3.0;
        let y1 = r.min.y + r.height() * 2.0 / 3.0;
        self.line(pos2(x, y0 + 1.0), pos2(x, y1 - 1.0), Color32::WHITE, 2.0);
        // The eraser: a small slanted block over the spot.
        let e = Rect::from_center_size(pos2(x + 6.0, y0 + 6.0), vec2(18.0, 10.0));
        self.p.rect_filled(e, 2.0, self.accent());
        self.p.rect_filled(Rect::from_min_size(e.min, vec2(7.0, e.height())), 2.0, self.accent().linear_multiply(0.5));
    }

    fn tb_delete_cells(&self) {
        let r = self.tb_table_rect();
        self.table(r, 3, 3);
        for col in 0..3 {
            self.tb_mark_cell(r, 3, 3, 1, col, self.red().linear_multiply(0.25));
        }
        let row = self.cell(r, 3, 3, 1, 0);
        let band = Rect::from_min_max(row.min, pos2(r.max.x, row.max.y));
        self.line(band.left_top(), band.right_bottom(), self.red(), 2.0);
        self.line(band.left_bottom(), band.right_top(), self.red(), 2.0);
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

    fn tb_autofit(&self) {
        let r = self.tb_table_rect();
        self.tb_table_uneven(r, &[0.34, 0.67], &[0.2, 0.65]);
        let y = r.min.y - 7.0;
        for (x0, x1) in [(0.0, 0.2), (0.2, 0.65), (0.65, 1.0)] {
            self.measure_h(pos2(r.min.x + r.width() * x0 + 1.0, y), pos2(r.min.x + r.width() * x1 - 1.0, y));
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

    fn tb_repeat_header(&self) {
        for x in [10.0, 94.0] {
            let a = self.page(x, 6.0, 72.0, 92.0);
            let r = Rect::from_min_size(a.min, vec2(a.width(), 66.0));
            self.table(r, 4, 2);
            self.tb_mark_row(r, 4, 2, 0);
        }
        self.label(self.at(88.0, 100.0), "1 → 2", 7.0, self.grey(), Align2::CENTER_CENTER);
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

    fn tb_formula(&self) {
        let r = self.tb_table_rect();
        self.table(r, 4, 2);
        // Numbers down the right column, their sum in the last cell.
        for (i, n) in ["٤", "٧", "٢"].iter().enumerate() {
            let cell = self.cell(r, 4, 2, i + 1, 1);
            self.p.rect_filled(cell.shrink(0.5), 0.0, Color32::WHITE);
            self.label(cell.center(), n, 8.0, self.grey(), Align2::CENTER_CENTER);
        }
        let last = self.cell(r, 4, 2, 3, 1);
        self.p.rect_filled(last.shrink(0.5), 0.0, Color32::WHITE);
        self.p.rect_filled(last.shrink(0.5), 0.0, self.accent().linear_multiply(0.3));
        self.label(last.center(), "Σ ١٣", 8.0, self.accent(), Align2::CENTER_CENTER);
    }
}
