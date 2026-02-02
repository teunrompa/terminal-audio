use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    widgets::{Block, Widget},
};

use crate::engine::{AudioEngine, AudioEngineState};

pub struct App {
    app_state: AppState,
    audio_engine: AudioEngine,
}

#[derive(PartialEq, Default)]
enum AppState {
    #[default]
    Running,
    Stopping,
}

//TODO: implement windows
enum AppWindows {
    Mixer,
    DeviceEditor,
    Sequencer,
}

impl App {
    pub fn new() -> Self {
        let audio_engine = AudioEngine::new().unwrap();
        App {
            app_state: AppState::Running,
            audio_engine,
        }
    }
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
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
            KeyCode::Char('p') => self.audio_engine.process().unwrap(),
            _ => {}
        };

        //Pass input down to lower levels
        self.audio_engine.handle_keyboard_input(key_event);
    }
}

impl Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let block = Block::bordered().title("Terminal Daw app");

        let inner = block.inner(area);

        let mixer = self.audio_engine.get_mixer();
        let state = self.audio_engine.get_engine_state();

        let block = match *state {
            AudioEngineState::Playing => block.title_top("Playing"),
            AudioEngineState::Stopping => block.title_top("Stopping"),
        };

        mixer.render(inner, buf);

        block.render(area, buf);
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
