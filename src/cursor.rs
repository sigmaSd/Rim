use crate::stats::Stats;
use tuikit::event::Key;

pub struct Cursor {
    pub row: usize,
    pub col: usize,
    width: usize,
}
impl Cursor {
    pub fn new(width: usize) -> Self {
        Self {
            row: 0,
            col: 0,
            width,
        }
    }
    pub fn reset(&mut self) {
        *self = Self {
            row: 0,
            col: 0,
            width: self.width,
        };
    }
    pub fn tuple(&self) -> (usize, usize) {
        (self.row, self.col)
    }

    pub fn advance_row(&mut self) {
        self.row += 1;
        self.col = 0;
    }
    pub fn moveit(&mut self, arrow: &Key, stats: &Stats) {
        match arrow {
            Key::Up => self.up(),
            Key::Down => self.down(stats),
            Key::Right => self.advance(stats),
            Key::Left => self.back(stats),
            _ => unreachable!(),
        }
    }
    pub fn advance<'a, P: Into<Option<&'a Stats>>>(&mut self, stats: P) {
        let bound = if let Some(stats) = stats.into() {
            // last spot dont move
            if self.last_spot(stats) {
                return;
            }

            stats.get_row_len(self.row)
        } else {
            self.width
        };

        if self.col == bound {
            self.col = 0;
            self.row += 1;
        } else {
            self.col += 1;
        }
    }

    fn back<'a, P: Into<Option<&'a Stats>>>(&mut self, stats: P) {
        let bound = match stats.into() {
            Some(stats) => stats.previous_row_len(self.row),
            None => self.width,
        };

        // first spot dont move
        if self.first_spot() {
            return;
        }

        if self.col == 0 {
            self.col = bound;
            self.row -= 1;
        } else {
            self.col -= 1;
        }
    }

    fn up(&mut self) {
        if self.row != 0 {
            self.col = 0;
            self.row -= 1;
        }
    }

    fn down(&mut self, stats: &Stats) {
        if !self.last_row(stats) {
            self.col = 0;
            self.row += 1;
        }
    }

    fn first_spot(&self) -> bool {
        (self.row, self.col) == (0, 0)
    }
    fn last_spot(&self, stats: &Stats) -> bool {
        self.last_row(stats) && self.last_col(stats)
    }
    fn last_row(&self, stats: &Stats) -> bool {
        self.row == stats.rows_num() - 1
    }
    fn last_col(&self, stats: &Stats) -> bool {
        self.col == stats.get_row_len(self.row)
    }
}
