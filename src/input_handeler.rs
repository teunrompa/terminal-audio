//Handels inputs for specific window context
pub struct InputHandler {
    window: ContextWindow,
}

enum ContextWindow {
    Sequencer,
    Mixer,
    Debug,
}
