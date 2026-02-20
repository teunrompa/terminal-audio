use std::io;

use terminal_daw::app::App;
use tui_logger::{TuiWidgetState, init_logger, set_default_level};

fn main() -> io::Result<()> {
    init_logger(log::LevelFilter::Trace).unwrap();
    set_default_level(log::LevelFilter::Info);
    let state = TuiWidgetState::default();

    ratatui::run(|terminal| App::new(state)?.run(terminal))
}
