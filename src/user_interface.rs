use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    style::Stylize,
    widgets::{Block, Paragraph, Widget},
};

pub struct InputWindow {
    input: String,
    characer_index: usize,
    input_mode: InputMode,
    history: Vec<String>,
}

#[derive(Default, PartialEq)]
pub enum InputMode {
    #[default]
    Normal,
    Editing,
}

impl InputWindow {
    pub fn new() -> Self {
        InputWindow {
            input: String::new(),
            characer_index: 0,
            input_mode: InputMode::Normal,
            history: Vec::new(),
        }
    }

    pub fn toggle_input_mode(&mut self) {
        match self.input_mode {
            InputMode::Normal => self.input_mode = InputMode::Editing,
            InputMode::Editing => self.input_mode = InputMode::Normal,
        }
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.characer_index.saturating_sub(1);
        self.characer_index = self.clamp_cursor(cursor_moved_left);
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.characer_index.saturating_add(1);
        self.characer_index = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.characer_index)
            .unwrap_or(self.input.len())
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.characer_index != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.characer_index;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn reset_cursor(&mut self) {
        self.characer_index = 0;
    }

    fn submit_message(&mut self) {
        self.history.push(self.input.clone());
        self.input.clear();
        self.reset_cursor();
        self.toggle_input_mode();
    }

    pub fn handle_keyboard_input(&mut self, key_event: KeyEvent) {
        match self.input_mode {
            InputMode::Normal => {
                if let KeyCode::Char('e') = key_event.code {
                    self.input_mode = InputMode::Editing;
                }
            }
            InputMode::Editing => match key_event.code {
                KeyCode::Enter => self.submit_message(),
                KeyCode::Char(to_insert) => self.enter_char(to_insert),
                KeyCode::Backspace => self.delete_char(),
                KeyCode::Left => self.move_cursor_left(),
                KeyCode::Right => self.move_cursor_right(),
                KeyCode::Esc => self.input_mode = InputMode::Normal,
                _ => {}
            },
        }
    }

    pub fn is_editing(&self) -> bool {
        self.input_mode == InputMode::Editing
    }

    pub fn get_last_string_input(&self) -> &String {
        if let Some(last_input) = self.history.last() {
            last_input
        } else {
            &self.input
        }
    }
}

/// helper function to create a centered rect using up certain percentage of the available rect `r`
fn popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}

impl Widget for &InputWindow {
    fn render(self, area: Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        if self.input_mode != InputMode::Editing {
            return;
        }

        let block = Block::bordered().title("Popup");
        let area = popup_area(area, 60, 20);
        let text_block = Paragraph::new(self.input.as_str()).block(block).on_green();

        text_block.render(area, buf);
    }
}

impl Default for InputWindow {
    fn default() -> Self {
        Self::new()
    }
}
