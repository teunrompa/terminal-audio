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
use tui_logger::{TuiLoggerWidget, TuiWidgetState};

use crate::engine::{AudioEngine, AudioEngineState};

pub struct App {
    state: AppState,
    audio_engine: AudioEngine,
    current_window: AppWindow,
    last_update: Instant,
    debug_state: TuiWidgetState,
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
    Debug,
}

impl App {
    pub fn new(debug_state: TuiWidgetState) -> io::Result<Self> {
        let audio_engine = AudioEngine::new().map_err(|e| io::Error::other(e.to_string()))?;
        Ok(App {
            state: AppState::Running,
            current_window: AppWindow::Mixer,
            audio_engine,
            last_update: Instant::now(),
            debug_state,
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

    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();
        let debug_state = &self.debug_state;

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
            AppWindow::Sequencer => self.render_sequencer(frame, content),
            AppWindow::Debug => self.render_debug_window(frame, debug_state),
        }

        // Footer with help
        let footer = Block::default()
            .borders(Borders::ALL)
            .title(" [Space] Play/Stop | [Tab] Window | [↑↓] Volume | [Q] Quit ");
        frame.render_widget(footer, chunks[2]);
    }
    // --- Window rendering ---
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

    fn render_sequencer(&self, frame: &mut Frame, area: ratatui::prelude::Rect) {
        let block = Block::default().title("Sequencer").borders(Borders::ALL);
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let mixer = self.audio_engine.get_mixer();
        if let Ok(mixer_guard) = &mut mixer.lock()
            && let Some(track) = mixer_guard.selected_track()
        {
            let sequencer = track.sequencer(); // You'll need a getter method
            frame.render_widget(sequencer, inner);
        }
    }

    fn render_debug_window(&self, frame: &mut Frame, state: &TuiWidgetState) {
        let area = frame.area();

        TuiLoggerWidget::default()
            .block(Block::bordered().title("Logs"))
            .state(state)
            .render(area, frame.buffer_mut());
    }

    /// --- Window management ---
    fn next_window(&mut self) {
        self.current_window = match self.current_window {
            AppWindow::Mixer => AppWindow::Sequencer,
            AppWindow::Sequencer => AppWindow::Mixer,
            AppWindow::Debug => AppWindow::Mixer,
        };
    }

    //TODO: implement switching window tabs
    fn _previous_window(&mut self) {
        self.current_window = match self.current_window {
            AppWindow::Mixer => AppWindow::Sequencer,
            AppWindow::Sequencer => AppWindow::Mixer,
            AppWindow::Debug => AppWindow::Mixer,
        };
    }

    fn current_window_title(&self) -> &'static str {
        match self.current_window {
            AppWindow::Mixer => "Mixer",
            AppWindow::Sequencer => "Sequencer",
            AppWindow::Debug => "Debug logs",
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

    fn handle_keys(&mut self, key_event: KeyEvent) {
        if let Ok(mut mixer) = self.audio_engine.get_mixer().lock() {
            //Handle context
            match self.current_window {
                AppWindow::Mixer => mixer.handle_keyboard_input(key_event),
                AppWindow::Sequencer => {
                    if let Some(track) = mixer.selected_track() {
                        let sequencer = track.sequencer_mut();

                        sequencer.handle_keyboard_input(key_event);
                    }
                }
                AppWindow::Debug => {}
            }
        }

        match key_event.code {
            KeyCode::Char('q') => self.state = AppState::Exiting,
            KeyCode::Tab => self.next_window(),
            KeyCode::Char('d') => self.current_window = AppWindow::Debug,
            _ => {}
        };
    }
}
