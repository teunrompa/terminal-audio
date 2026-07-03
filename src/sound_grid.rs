use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::Widget,
};

pub trait GridItem {
    fn render(&self, area: Rect, buf: &mut Buffer);
    fn handle_input(&mut self, key: KeyEvent);
}

#[derive(PartialEq)]
pub enum GridMode {
    Navigate,
    Edit,
}

pub enum SelectedItem {
    DefaultCell,
    Instrument,
    Fx,
}

pub struct Grid {
    cells: Vec<Vec<Option<Box<dyn GridItem>>>>,
    cursor_pos: (usize, usize),
    mode: GridMode,
    selected_item: SelectedItem,
}

pub struct Cell(u16, u16);

impl Grid {
    pub fn new(cols: usize, rows: usize) -> Self {
        let cells = (0..rows)
            .map(|_| (0..cols).map(|_| None).collect())
            .collect();
        Grid {
            cells,
            cursor_pos: (0, 0),
            mode: GridMode::Navigate,
            selected_item: SelectedItem::DefaultCell,
        }
    }

    /// Single entry point — routes the key event based on current mode.
    /// Call this from your app's input loop instead of handle_navigate/handle_edit directly.
    pub fn handle_key(&mut self, key: KeyEvent) {
        match self.mode {
            GridMode::Navigate => self.handle_navigate(key),
            GridMode::Edit => self.handle_edit(key),
        }
    }

    pub fn set_selected_item(&mut self, item: SelectedItem) {
        self.selected_item = item;
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

    fn handle_edit(&mut self, key: KeyEvent) {
        if key.code == KeyCode::Esc {
            self.mode = GridMode::Navigate;
            return;
        }

        let (col, row) = self.cursor_pos;
        let cell_is_empty = self.cells[row][col].is_none();

        if key.code == KeyCode::Enter && cell_is_empty {
            let item = self.get_item_to_insert();
            if let Err(e) = self.insert_cell(self.cursor_pos, item) {
                // cursor_pos is always clamped in-bounds, so this shouldn't
                // normally happen — but worth knowing about if it ever does.
                eprintln!("insert_cell failed: {e}");
            }
            self.mode = GridMode::Navigate;
            return;
        }

        if let Some(panel) = &mut self.cells[row][col] {
            panel.handle_input(key);
        }
    }

    fn get_item_to_insert(&self) -> Box<dyn GridItem> {
        match self.selected_item {
            SelectedItem::DefaultCell => {
                Box::new(Cell(self.cursor_pos.0 as u16, self.cursor_pos.1 as u16))
            }
            SelectedItem::Instrument => todo!(),
            SelectedItem::Fx => todo!(),
        }
    }

    /// Inserts a GridItem at the given position.
    pub fn insert_cell(
        &mut self,
        cursor_pos: (usize, usize),
        panel: Box<dyn GridItem>,
    ) -> Result<(), String> {
        let (col, row) = cursor_pos;
        let row_vec = self
            .cells
            .get_mut(row)
            .ok_or_else(|| format!("row out of bounds {row}"))?;
        let cell = row_vec
            .get_mut(col)
            .ok_or_else(|| format!("col out of bounds {col}"))?;

        *cell = Some(panel);
        Ok(())
    }

    pub fn cols(&self) -> usize {
        self.cells.first().map_or(0, |r| r.len())
    }

    pub fn rows(&self) -> usize {
        self.cells.len()
    }
}

impl Widget for &Grid {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let rows = self.rows();
        let cols = self.cols();
        if rows == 0 || cols == 0 {
            return;
        }

        for row_idx in 0..rows {
            for col_idx in 0..cols {
                let x = area.x + col_idx as u16;
                let y = area.y + row_idx as u16;

                if x >= area.x + area.width || y >= area.y + area.height {
                    continue;
                }

                let cell_area = Rect {
                    x,
                    y,
                    width: 1,
                    height: 1,
                };
                let is_cursor = self.cursor_pos == (col_idx, row_idx);

                match &self.cells[row_idx][col_idx] {
                    Some(item) => {
                        item.render(cell_area, buf);
                        if is_cursor {
                            // Occupied + cursor: tint background, keep the
                            // item's own character/style untouched.
                            let bg = match self.mode {
                                GridMode::Navigate => Color::DarkGray,
                                GridMode::Edit => Color::Blue,
                            };
                            buf.set_style(cell_area, Style::default().bg(bg));
                        }
                    }
                    None => {
                        let (ch, style) = if is_cursor {
                            match self.mode {
                                GridMode::Navigate => (
                                    '_',
                                    Style::default()
                                        .fg(Color::Yellow)
                                        .add_modifier(Modifier::BOLD),
                                ),
                                GridMode::Edit => (
                                    self.selected_item_preview_char(),
                                    Style::default()
                                        .fg(Color::Black)
                                        .bg(Color::Green)
                                        .add_modifier(Modifier::BOLD),
                                ),
                            }
                        } else {
                            ('.', Style::default())
                        };
                        buf.set_string(x, y, ch.to_string(), style);
                    }
                }
            }
        }
    }
}

impl Grid {
    fn selected_item_preview_char(&self) -> char {
        match self.selected_item {
            SelectedItem::DefaultCell => '#',
            SelectedItem::Instrument => 'I',
            SelectedItem::Fx => 'F',
        }
    }
}

impl GridItem for Cell {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        buf.set_string(area.x, area.y, "#", Style::default());
    }

    fn handle_input(&mut self, _key: KeyEvent) {
        todo!()
    }
}

