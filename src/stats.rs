#[derive(Default)]
pub struct Stats {
    rows_len: Vec<usize>,
    cummulative_rows_len: Vec<usize>,
}

impl Stats {
    pub fn read(&mut self, buffer: &str) {
        buffer.split('\n').for_each(|s| {
            self.rows_len.push(s.len());
            self.cummulative_rows_len
                .push(s.len() + self.cummulative_rows_len.last().unwrap_or(&0) + 1);
        });
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
    pub fn get_cummulative_rows_len(&self, row: usize) -> usize {
        self.cummulative_rows_len[row]
    }
    pub fn update_row_len(&mut self, row: usize) {
        self.rows_len[row] += 1;
        self.cummulative_rows_len
            .iter_mut()
            .skip(row)
            .for_each(|row_len| {
                *row_len += 1;
            });
    }
    pub fn rows_num(&self) -> usize {
        self.rows_len.len()
    }
}
