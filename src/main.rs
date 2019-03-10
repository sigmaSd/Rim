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
}
impl Terminal {
    fn new(buffer: String) -> Self {
        let term = Rc::new(RefCell::new(
            Term::with_height(TermHeight::Percent(100)).unwrap(),
        ));

        let (width, height) = term.borrow().term_size().unwrap();
        let cursor = Cursor::new(width);

        Self {
            term,
            buffer,
            width,
            height,
            cursor,
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
    fn insert_character(&mut self, character: char) {
        self.buffer.insert(self.cursor_to_pos(), character);
        // self.print_char(character);
        // self.present();
        self.print_all();
    }

    fn _reset_cursor(&mut self) {
        self.cursor.reset();
    }

    fn advance_cursor(&mut self) {
        self.cursor.advance();
        self.print_cursor();
    }
    fn cursor_to_pos(&self) -> usize {
        self.buffer
            .split('\n')
            .take(self.cursor.row)
            .fold(0, |acc, x| acc + x.len() + 1)
            + self.cursor.col
    }

    fn move_cursor(&mut self, arrow: &Key) {
        let mut current_cursor = (self.cursor.row as i32, self.cursor.col as i32);

        let direction = match arrow {
            Key::Up => (-1, 0),
            Key::Down => (1, 0),
            Key::Right => (0, 1),
            Key::Left => (0, -1),
            _ => unreachable!(),
        };

        current_cursor.0 += direction.0;
        current_cursor.1 += direction.1;

        current_cursor.0 = cmp::max(current_cursor.0, 0);
        current_cursor.1 = cmp::max(current_cursor.1, 0);

        self.cursor.row = current_cursor.0 as usize;
        self.cursor.col = current_cursor.1 as usize;

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
            cursor.advance();
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

    fn advance(&mut self) {
        if self.col == self.width {
            self.col = 0;
            self.row += 1;
        } else {
            self.col += 1;
        }
    }

    fn advance_row(&mut self) {
        self.row += 1;
        self.col = 0;
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
