#[derive(Default)]
pub struct Stats {
    pub rows_len: Vec<usize>,
}

impl Stats {
    pub fn read(&mut self, buffer: &str) {
        self.rows_len = buffer.split('\n').map(|s| s.len()).collect();
    }
    pub fn previous_row_len(&self, row: usize) -> usize {
        if row == 0 {
            0
        } else {
            self.rows_len[row - 1]
        }
    }
    pub fn get_row_len(&self, row: usize) -> usize {
        self.rows_len[row]
    }
    pub fn update_row_len(&mut self, row: usize, new_len: usize) {
        self.rows_len[row] = new_len;
    }
    pub fn rows_num(&self) -> usize {
        self.rows_len.len()
    }
}
