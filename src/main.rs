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

mod cursor;
use cursor::Cursor;

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
    window: Window,
}
impl Terminal {
    fn new(buffer: String) -> Self {
        let term = Rc::new(RefCell::new(
            Term::with_height(TermHeight::Percent(100)).unwrap(),
        ));

        let (width, height) = term.borrow().term_size().unwrap();
        let window = Window::new(height);
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
            window,
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
        self.update_current_row_len();
        self.move_cursor(&Key::Right);
    }

    fn move_cursor(&mut self, arrow: &Key) {
        self.cursor.moveit(arrow, &self.stats);
        self.move_window();
        self.set_cursor();
        self.print_all();
    }

    fn move_window(&mut self) {
        if self.cursor.row == self.window.upper_bound {
            self.window.move_down();
        } else if self.cursor.row + 1 == self.window.lower_bound && self.window.lower_bound != 0 {
            self.window.move_up();
        }
    }

    fn set_cursor(&self) {
        let _ = self
            .term
            .borrow()
            .set_cursor(self.cursor.row - self.window.lower_bound, self.cursor.col);
    }

    fn print_all(&mut self) {
        let _ = self.term.borrow().clear();

        let mut cursor = Cursor::new(self.width);
        self.buffer
            .split('\n')
            .skip(self.window.lower_bound)
            .take(self.window.upper_bound)
            .for_each(|line| {
                line.chars().for_each(|c| {
                    self.print_char_at(c, &cursor);
                    cursor.advance(None);
                });
                cursor.advance_row();
            });

        self.present();
    }
    fn print_char_at(&self, character: char, cursor: &Cursor) {
        let (row, col) = cursor.tuple();
        //XXX too much allocation
        let _ = self.term.borrow().print(row, col, &character.to_string());
    }
    fn _print_char(&self, character: char) {
        let (row, col) = self.cursor.tuple();
        //XXX too much allocation
        let _ = self.term.borrow().print(row, col, &character.to_string());
    }
    fn present(&self) {
        let _ = self.term.borrow().present();
    }

    // helper fns
    fn get_current_row_len(&self) -> usize {
        self.stats.get_row_len(self.cursor.row)
    }
    fn update_current_row_len(&mut self) {
        self.stats
            .update_row_len(self.cursor.row, self.get_current_row_len() + 1);
    }
    fn cursor_to_pos(&self) -> usize {
        self.buffer
            .split('\n')
            .take(self.cursor.row)
            .fold(0, |acc, x| acc + x.len() + 1)
            + self.cursor.col
    }

    // debug
    fn _debug<T: ToString>(&self, x: T) {
        let _ = self.term.borrow().clear();
        let _ = self.term.borrow().print(0, 0, &x.to_string());
        self.present();
    }
}

struct Window {
    lower_bound: usize,
    upper_bound: usize,
}
impl Window {
    fn new(upper_bound: usize) -> Self {
        Self {
            lower_bound: 0,
            upper_bound,
        }
    }
    fn move_down(&mut self) {
        self.upper_bound += 1;
        self.lower_bound += 1;
    }
    fn move_up(&mut self) {
        if self.upper_bound != 0 {
            self.upper_bound -= 1;
        }
        if self.lower_bound != 0 {
            self.lower_bound -= 1;
        }
    }
}
