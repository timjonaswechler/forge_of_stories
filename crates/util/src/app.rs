#[derive(Debug)]
pub enum ContextType {
    Client,
    DedicatedServer,
    LocalServer,
    None,
}

#[derive(Debug)]
pub struct AppContext {
    pub context: Vec<ContextType>,
}

impl AppContext {
    #[cfg(feature = "server")]
    pub fn new() -> Self {
        Self {
            context: vec![ContextType::DedicatedServer],
        }
    }

    #[cfg(feature = "client")]
    pub fn new_client() -> Self {
        Self {
            context: vec![ContextType::Client, ContextType::LocalServer],
        }
    }
    #[cfg(all(not(feature = "server"), not(feature = "client")))]
    pub fn new() -> Self {
        Self {
            context: vec![ContextType::None],
        }
    }
}
