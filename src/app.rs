use std::{
    io::{self},
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Widget},
};

use crate::engine::{AudioEngine, AudioEngineState};

pub struct App {
    state: AppState,
    audio_engine: AudioEngine,
    current_window: AppWindow,
    last_update: Instant,
}

#[derive(PartialEq, Default)]
enum AppState {
    #[default]
    Running,
    Exiting,
}

//TODO: implement windows
#[derive(Default)]
enum AppWindow {
    #[default]
    Mixer,
    Sequencer,
}

impl App {
    pub fn new() -> io::Result<Self> {
        let audio_engine = AudioEngine::new().map_err(|e| io::Error::other(e.to_string()))?;
        Ok(App {
            state: AppState::Running,
            current_window: AppWindow::Mixer,
            audio_engine,
            last_update: Instant::now(),
        })
    }
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        {
            let mixer = self.audio_engine.get_mixer();
            if let Ok(mut mixer) = mixer.lock() {
                mixer.add_track(0.3, "Kick".into(), 16, 4, self.get_sample_rate());
            }
        }

        self.setup_test_pattern();

        if let Err(e) = self.audio_engine.start() {
            eprintln!("Failed to start audio {}", e);
        }

        while self.state == AppState::Running {
            terminal.draw(|frame| {
                self.draw(frame);
            })?;

            if event::poll(Duration::from_millis(16))?
                && let Event::Key(key) = event::read()?
            {
                self.handle_keys(key);
            }

            self.last_update = Instant::now();
        }

        self.audio_engine.stop();

        Ok(())
    }

    pub fn get_sample_rate(&self) -> f32 {
        self.audio_engine.sample_rate()
    }

    pub fn setup_test_pattern(&mut self) {
        let mixer = self.audio_engine.get_mixer();
        let mut mixer = mixer.lock().unwrap();

        mixer.add_track(0.8, "Kick".to_string(), 16, 16, self.get_sample_rate());

        if let Some(track) = mixer.selected_track() {
            let seq = track.sequencer_mut();
            seq.set_note_at(0, 60.0, 1.0); // Step 0
            seq.set_note_at(4, 144.0, 0.9); // Step 4
            seq.set_note_at(8, 60.0, 1.0); // Step 8
            seq.set_note_at(12, 144.0, 0.9); // Step 12
        }
    }

    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();

        let chunks = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(area);

        // Header with status
        let header = Block::default()
            .borders(Borders::ALL)
            .title(format!(
                " Terminal DAW | {} | {} | {:.0} BPM ",
                self.current_window_title(),
                self.playback_status(),
                self.get_bpm()
            ))
            .title_style(Style::default().fg(Color::Cyan));

        frame.render_widget(header, chunks[0]);

        // Main content area
        let content = chunks[1];
        match self.current_window {
            AppWindow::Mixer => self.render_mixer(frame, content),
            AppWindow::Sequencer => self.render_sequencer_placeholder(frame, content),
        }

        // Footer with help
        let footer = Block::default()
            .borders(Borders::ALL)
            .title(" [Space] Play/Stop | [Tab] Window | [↑↓] Volume | [Q] Quit ");
        frame.render_widget(footer, chunks[2]);
    }

    fn render_mixer(&self, frame: &mut Frame, area: ratatui::prelude::Rect) {
        let mixer = self.audio_engine.get_mixer();

        // Lock once and render directly
        if let Ok(mixer_guard) = mixer.lock() {
            mixer_guard.render(area, frame.buffer_mut());
        } else {
            // Render error state if mutex poisoned
            let block = Block::default()
                .title("Mixer (Locked)")
                .borders(Borders::ALL);
            frame.render_widget(block, area);
        }
    }

    fn render_sequencer_placeholder(&self, frame: &mut Frame, area: ratatui::prelude::Rect) {
        let block = Block::default().title("Sequencer").borders(Borders::ALL);
        frame.render_widget(block, area);
    }

    //TODO: these key inputs should only be avalable when the mixer window is open
    fn _handle_mixer_keys(&mut self, key: KeyEvent) {
        // Mixer-specific keys that App handles directly
        match key.code {
            // Add mixer-specific navigation here
            KeyCode::Char('b') => todo!(),
            KeyCode::Char('n') => todo!(),
            _ => {}
        }
    }
    //TODO: these key inputs should only be avalable when the sequencer window is open
    fn _handle_sequencer_keys(&mut self, key: KeyEvent) {
        // Sequencer editing mode
        match key.code {
            KeyCode::Char('b') => todo!(),
            KeyCode::Char('n') => todo!(),
            _ => {}
        }
    }

    fn _next_window(&mut self) {
        self.current_window = match self.current_window {
            AppWindow::Mixer => AppWindow::Sequencer,
            AppWindow::Sequencer => AppWindow::Mixer,
        };
    }

    fn _previous_window(&mut self) {
        self.current_window = match self.current_window {
            AppWindow::Mixer => AppWindow::Sequencer,
            AppWindow::Sequencer => AppWindow::Mixer,
        };
    }

    // --- Helpers ---

    fn current_window_title(&self) -> &'static str {
        match self.current_window {
            AppWindow::Mixer => "Mixer",
            AppWindow::Sequencer => "Sequencer",
        }
    }

    fn playback_status(&self) -> String {
        match self.audio_engine.state() {
            AudioEngineState::Playing => "▶ Playing".to_string(),
            AudioEngineState::Stopped => "⏹ Stopped".to_string(),
        }
    }

    fn get_bpm(&self) -> f32 {
        self.audio_engine
            .get_mixer()
            .lock()
            .map(|m| m.bpm())
            .unwrap_or(0.0)
    }

    //TODO: implement key handeling at specific context
    fn handle_keys(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.state = AppState::Exiting,
            KeyCode::Char('l') => todo!(), //TODO: implement window switch
            _ => {}
        };

        //Pass input down to lower levels
        self.audio_engine.handle_keyboard_input(key_event);
    }
}
