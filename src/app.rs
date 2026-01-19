use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    widgets::{Block, Widget},
};

use crate::engine::AudioEngine;

#[derive(Default)]
pub struct App {
    app_state: AppState,
}

#[derive(PartialEq, Default)]
enum AppState {
    #[default]
    Running,
    Stopping,
}

enum AppWindows {
    Mixer,
    DeviceEditor,
}

impl App {
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        AudioEngine::new()
            .run()
            .expect("Audio engine failed to start");
        while self.app_state == AppState::Running {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_keys(key_event);
            }

            _ => {}
        };

        Ok(())
    }

    //TODO: implement key handeling at specific context
    fn handle_keys(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.app_state = AppState::Stopping,
            KeyCode::Char('l') => todo!(), //TODO:: implement next window
            KeyCode::Char('h') => todo!(), //TODO:: implement prevuis window
            _ => {}
        }
    }
}

impl Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let block = Block::bordered().title("Terminal Daw app");
        block.render(area, buf);
    }
}
