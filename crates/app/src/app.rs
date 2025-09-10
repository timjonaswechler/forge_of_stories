use std::path::PathBuf;

pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

pub struct AppBase {
    pub app_id: &'static str,
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub logs_dir: PathBuf,
}

impl AppBase {
    pub fn app_id(&self) -> &'static str {
        self.app_id
    }
}

pub trait Application: Sized + 'static {
    type Error: From<BoxError> + std::fmt::Display + std::fmt::Debug + 'static;

    const APP_ID: &'static str;

    fn init_platform() -> Result<(), Self::Error> {
        Ok(())
    }
}

pub fn init<A: Application>() -> Result<AppBase, A::Error> {
    A::init_platform()?;

    let app_id = A::APP_ID;

    let config_dir = paths::config_dir().join(app_id);
    let data_dir = paths::data_dir().join(app_id);
    let logs_dir = paths::logs_dir().join(app_id);

    Ok(AppBase {
        app_id,
        config_dir,
        data_dir,
        logs_dir,
    })
}
