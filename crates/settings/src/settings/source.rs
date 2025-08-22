#[derive(Clone, Copy, Debug)]
pub struct SettingsSources<'a, T> {
    /// The default Zed settings.
    pub default: &'a T,
    /// Global settings (loaded before user settings).
    pub global: Option<&'a T>,
    /// Settings provided by extensions.
    pub extensions: Option<&'a T>,
    /// The user settings.
    pub user: Option<&'a T>,
    /// The user settings for the current release channel.
    pub release_channel: Option<&'a T>,
    /// The user settings for the current operating system.
    pub operating_system: Option<&'a T>,
    /// The settings associated with an enabled settings profile
    pub profile: Option<&'a T>,
    /// The server's settings.
    pub server: Option<&'a T>,
    /// The project settings, ordered from least specific to most specific.
    pub project: &'a [&'a T],
}
