use color_eyre::eyre::Result;

#[derive(Default)]
pub struct State {
    pub active_operation_index: usize,
    pub input_mode: InputMode,
}

#[derive(Default, PartialEq)]
pub enum InputMode {
    #[default]
    Normal,
    Insert,
    Command,
}

impl State {
    pub fn new() -> Result<Self> {
        Ok(Self {
            active_operation_index: 0,
            input_mode: InputMode::Normal,
        })
    }
}
