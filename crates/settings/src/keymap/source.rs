#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum KeybindSource {
    #[default]
    Default,
    User,
}

impl KeybindSource {
    pub fn name(&self) -> &'static str {
        match self {
            KeybindSource::Default => "Default",
            KeybindSource::User => "User",
        }
    }
}
