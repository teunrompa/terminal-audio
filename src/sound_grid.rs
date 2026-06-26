use crossterm::event::KeyEvent;
use ratatui::{buffer::Buffer, layout::Rect};

pub trait Panel {
    fn render(&self, area: Rect, buf: &mut Buffer);
    fn handle_input(&mut self, key: KeyEvent);
    fn title(&self) -> &str;
}

pub enum GridMode {
    Navigate,
    Edit,
}

pub struct Grid {
    cells: Vec<Vec<Option<Box<dyn Panel>>>>,
    cursor_pos: (usize, usize),
    mode: GridMode,
}

impl Grid {
    pub fn new(colls: usize, rows: usize) -> Self {
        let cells = (0..rows)
            .map(|_| (0..colls).map(|_| None).collect())
            .collect();
        Grid {
            cells,
            cursor_pos: (0, 0),
            mode: GridMode::Navigate,
        }
    }

    pub fn cols(&self) -> usize {
        self.cells.first().map_or(0, |r| r.len())
    }
    pub fn rows(&self) -> usize {
        self.cells.len()
    }
}
