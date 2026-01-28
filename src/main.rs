use std::io;

use terminal_daw::app::App;

fn main() -> io::Result<()> {
    let mut app = App::new();
    ratatui::run(|terminal| app.run(terminal))
}
