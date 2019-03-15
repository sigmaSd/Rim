use std::cell::RefCell;
use std::cmp;
use std::env::args;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::rc::Rc;

use tuikit::attr::*;
use tuikit::event::{Event, Key};
use tuikit::term::{Term, TermHeight};

mod stats;
use stats::Stats;

type RR<T> = Rc<RefCell<T>>;

fn main() {
    let file_path = args().skip(1).collect();
    open_file(file_path);
}

fn open_file(file_path: PathBuf) {
    let mut file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(file_path)
        .expect("Error while reading file");

    let data = {
        let mut d = String::new();
        let _ = file.read_to_string(&mut d);
        d
    };
    open_data(data);
}

fn open_data(data: String) {
    let mut terminal = Terminal::new(data);
    terminal.print_all();

    terminal.run();
}

struct Terminal {
    term: RR<Term>,
    buffer: String,
    width: usize,
    height: usize,
    cursor: Cursor,
    stats: Stats,
}
impl Terminal {
    fn new(buffer: String) -> Self {
        let term = Rc::new(RefCell::new(
            Term::with_height(TermHeight::Percent(100)).unwrap(),
        ));

        let (width, height) = term.borrow().term_size().unwrap();
        let cursor = Cursor::new(width);
        let mut stats = Stats::default();
        stats.read(&buffer);

        Self {
            term,
            buffer,
            width,
            height,
            cursor,
            stats,
        }
    }
    fn run(&mut self) {
        self.initilize();
        let term = self.term.clone();

        while let Ok(ev) = term.borrow().poll_event() {
            match ev {
                Event::Key(Key::ESC) | Event::Key(Key::Char('q')) => break,
                Event::Key(ref arrow)
                    if arrow == &Key::Up
                        || arrow == &Key::Down
                        || arrow == &Key::Left
                        || arrow == &Key::Right =>
                {
                    self.move_cursor(arrow)
                }
                Event::Key(Key::Char(character)) => self.insert_character(character),
                _ => (),
            }
        }
    }
    fn initilize(&mut self) {
        self.cursor.reset();
        let _ = self
            .term
            .borrow()
            .set_cursor(self.cursor.row, self.cursor.col);

        self.present();
    }
    fn get_current_row_len(&self) -> usize {
        self.stats.get_row_len(self.cursor.row)
    }
    fn update_current_row_len(&mut self) {
        self.stats
            .update_row_len(self.cursor.row, self.get_current_row_len() + 1);
    }
    fn insert_character(&mut self, character: char) {
        self.buffer.insert(self.cursor_to_pos(), character);
        self.update_current_row_len();
        self.move_cursor(&Key::Right);
        //self.print_char(character);
        //self.present();
        self.print_all();
    }

    fn cursor_to_pos(&self) -> usize {
        self.buffer
            .split('\n')
            .take(self.cursor.row)
            .fold(0, |acc, x| acc + x.len() + 1)
            + self.cursor.col
    }

    fn move_cursor(&mut self, arrow: &Key) {
        self.cursor.moveit(arrow, &self.stats);
        self.print_cursor();
    }

    fn print_cursor(&self) {
        let _ = self
            .term
            .borrow()
            .set_cursor(self.cursor.row, self.cursor.col);
        self.present();
    }

    fn print_all(&mut self) {
        let _ = self.term.borrow().clear();

        let mut cursor = Cursor::new(self.width);

        let mut tmp_buffer: String = self.buffer.drain(..).collect();
        for character in tmp_buffer.chars() {
            if character == '\n' {
                cursor.advance_row();
                continue;
            }

            self.print_char_at(character, &cursor);
            cursor.advance(None);
        }

        self.buffer = tmp_buffer.drain(..).collect();
        self.present();
    }
    fn print_char_at(&self, character: char, cursor: &Cursor) {
        let (row, col) = cursor.tuple();
        //XXX too much allocation
        let _ = self.term.borrow().print(row, col, &character.to_string());
    }
    fn print_char(&self, character: char) {
        let (row, col) = self.cursor.tuple();
        //XXX too much allocation
        let _ = self.term.borrow().print(row, col, &character.to_string());
    }
    fn present(&self) {
        let _ = self.term.borrow().present();
    }
}

struct Cursor {
    row: usize,
    col: usize,
    width: usize,
}
impl Cursor {
    fn new(width: usize) -> Self {
        Self {
            row: 0,
            col: 0,
            width,
        }
    }
    fn reset(&mut self) {
        *self = Self {
            row: 0,
            col: 0,
            width: self.width,
        };
    }
    fn tuple(&self) -> (usize, usize) {
        (self.row, self.col)
    }

    fn advance_row(&mut self) {
        self.row += 1;
        self.col = 0;
    }
    fn moveit(&mut self, arrow: &Key, stats: &Stats) {
        match arrow {
            Key::Up => self.up(),
            Key::Down => self.down(),
            Key::Right => self.advance(stats.get_row_len(self.row)),
            Key::Left => self.back(stats.previous_row_len(self.row)),
            _ => unreachable!(),
        }
    }
    fn advance<P: Into<Option<usize>>>(&mut self, current_row_len: P) {
        let bound = match current_row_len.into() {
            Some(row_len) => row_len,
            None => self.width,
        };
        if self.col == bound {
            self.col = 0;
            self.row += 1;
        } else {
            self.col += 1;
        }
    }

    fn back<P: Into<Option<usize>>>(&mut self, previous_row_len: P) {
        let bound = match previous_row_len.into() {
            Some(row_len) => row_len,
            None => self.width,
        };
        if self.col == 0 {
            self.col = bound;
            self.row -= 1;
        } else {
            self.col -= 1;
        }
    }

    fn up(&mut self) {
        self.col = 0;
        if self.row != 0 {
            self.row -= 1;
        }
    }

    fn down(&mut self) {
        self.col = 0;
        self.row += 1;
    }
}

// trait StringUtils {
//     fn insert_push(&mut self, idx: usize, character: char);
// }
// impl StringUtils for String {
//     fn insert_push(&mut self, idx: usize, character: char) {
//         let mut tmp_string: String = self.drain(..idx).collect();
//         tmp_string.push(character);
//         tmp_string = self.drain(..).collect();
//         self.clear();
//self.push_str(&tmp_string);
//     }
// }
