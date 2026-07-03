use std::{
    io::{self},
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    widgets::{Block, Borders, Widget},
};
use tui_logger::{TuiLoggerWidget, TuiWidgetState};

use crate::{engine::AudioEngine, sound_grid::Grid};

pub struct App {
    state: AppState,
    audio_engine: AudioEngine,
    grid: Grid,
    current_window: AppWindow,
    last_update: Instant,
    debug_state: TuiWidgetState,
    tick_rate: Duration,
    last_tick: Instant,
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
    Grid,
    Sequencer,
    Debug,
}

impl App {
    pub fn new(debug_state: TuiWidgetState) -> io::Result<Self> {
        let audio_engine = AudioEngine::new().map_err(|e| io::Error::other(e.to_string()))?;
        Ok(App {
            state: AppState::Running,
            current_window: AppWindow::Grid,
            audio_engine,
            last_update: Instant::now(),
            debug_state,
            grid: Grid::new(250, 100),
            tick_rate: Duration::from_millis(33),
            last_tick: Instant::now(),
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
            let timeout = self.tick_rate.saturating_sub(self.last_tick.elapsed());

            if self.last_tick.elapsed() >= self.tick_rate {
                terminal.draw(|frame| {
                    self.draw(frame);
                })?;
            }

            if event::poll(Duration::from_millis(timeout.as_secs()))?
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

        // Main content area
        let content = chunks[1];

        match self.current_window {
            AppWindow::Grid => self.render_grid(frame, area),
            AppWindow::Sequencer => self.render_sequencer(frame, content),
            AppWindow::Debug => self.render_debug_window(frame, debug_state),
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

    fn render_grid(&self, frame: &mut Frame, area: ratatui::prelude::Rect) {
        frame.render_widget(&self.grid, area);
    }

    fn render_debug_window(&self, frame: &mut Frame, state: &TuiWidgetState) {
        let area = frame.area();

        TuiLoggerWidget::default()
            .block(Block::bordered().title("Logs"))
            .state(state)
            .render(area, frame.buffer_mut());
    }

    /// --- Window management ---
    /// TODO: implement context window
    fn next_window(&mut self) {
        self.current_window = match self.current_window {
            AppWindow::Grid => AppWindow::Sequencer,
            AppWindow::Sequencer => AppWindow::Grid,
            AppWindow::Debug => AppWindow::Grid,
        };
    }

    //TODO: implement switching window tabs
    fn _previous_window(&mut self) {
        self.current_window = match self.current_window {
            AppWindow::Grid => AppWindow::Sequencer,
            AppWindow::Sequencer => AppWindow::Grid,
            AppWindow::Debug => AppWindow::Grid,
        };
    }

    fn handle_keys(&mut self, key_event: KeyEvent) {
        if let Ok(mut mixer) = self.audio_engine.get_mixer().lock() {
            //Handle context
            match self.current_window {
                AppWindow::Grid => {
                    self.grid.handle_key(key_event);
                }
                AppWindow::Sequencer => {
                    if let Some(track) = mixer.selected_track() {
                        let sequencer = track.sequencer_mut();

                        sequencer.handle_keyboard_input(key_event);
                    }
                }
                AppWindow::Debug => {}
            }
        }

        //TODO: fix implementation so that it does not go to debug when typing in the note
        match key_event.code {
            KeyCode::Char('q') => self.state = AppState::Exiting,
            KeyCode::Tab => self.next_window(),
            KeyCode::Char('d') => self.current_window = AppWindow::Debug,
            _ => {}
        };
    }
}
