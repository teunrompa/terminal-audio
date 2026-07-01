use std::cell;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect, Spacing},
    style::{Color, Modifier, Style},
    symbols::merge::MergeStrategy,
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::generators::Processor;

pub trait GridItem {
    fn render(&self, area: Rect, buf: &mut Buffer);
    fn handle_input(&mut self, key: KeyEvent);
}

pub enum GridMode {
    Navigate,
    Edit,
}

pub struct Grid {
    cells: Vec<Vec<Option<Box<dyn GridItem>>>>,
    cursor_pos: (usize, usize),
    mode: GridMode,
}

impl Grid {
    pub fn new(colls: usize, rows: usize) -> Self {
        //Init empty cells
        let cells = (0..rows)
            .map(|_| (0..colls).map(|_| None).collect())
            .collect();
        Grid {
            cells,
            cursor_pos: (0, 0),
            mode: GridMode::Navigate,
        }
    }

    pub fn handle_navigate(&mut self, key: KeyEvent) {
        let (col, row) = self.cursor_pos;
        match key.code {
            KeyCode::Up => self.cursor_pos = (col, row.saturating_sub(1)),
            KeyCode::Down => self.cursor_pos = (col, (row + 1).min(self.rows() - 1)),
            KeyCode::Left => self.cursor_pos = (col.saturating_sub(1), row),
            KeyCode::Right => self.cursor_pos = ((col + 1).min(self.cols() - 1), row),
            KeyCode::Enter => self.mode = GridMode::Edit,
            _ => {}
        }
    }

    //Inserts a gridItem at cursor pos
    pub fn insert_panel(
        &mut self,
        cursor_pos: (usize, usize),
        panel: Box<dyn GridItem>,
    ) -> Result<(), String> {
        let (row, col) = cursor_pos;

        let row_vec = self
            .cells
            .get_mut(row)
            .ok_or_else(|| format!("row out of bounds {}", row))?;
        let cell = row_vec
            .get_mut(col)
            .ok_or_else(|| format!("col out of bounds {}", col))?;

        *cell = Some(panel);

        Ok(())
    }

    pub fn handle_edit(&mut self, key: KeyEvent) {
        if key.code == KeyCode::Esc {
            self.mode = GridMode::Navigate;
            return;
        }

        let (col, row) = self.cursor_pos;

        if let Some(panel) = &mut self.cells[row][col] {
            panel.handle_input(key);
        }
    }

    pub fn cols(&self) -> usize {
        self.cells.first().map_or(0, |r| r.len())
    }
    pub fn rows(&self) -> usize {
        self.cells.len()
    }
}

impl Widget for &Grid {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
    }
}
