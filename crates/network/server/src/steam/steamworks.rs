#![cfg(feature = "steamworks")]

use std::{sync::Arc, time::Duration};

use network_shared::discovery::SteamServerEvent;
use steamworks::{Client, ClientManager, SingleClient};
use tokio::{sync::Mutex, task::JoinHandle};
use tracing::{debug, warn};

use crate::runtime::NetworkRuntime;

use super::{SteamBackendHandle, SteamIntegration, SteamIntegrationError};

/// Konfiguration für die Steamworks-Integration.
#[derive(Debug, Clone)]
pub struct SteamworksIntegrationConfig {
    pub app_id: u32,
    pub callback_interval: Duration,
}

impl Default for SteamworksIntegrationConfig {
    fn default() -> Self {
        Self {
            app_id: 0,
            callback_interval: Duration::from_millis(50),
        }
    }
}

/// Steamworks-basierte Implementierung der [`SteamIntegration`].
pub struct SteamworksIntegration {
    runtime: NetworkRuntime,
    config: SteamworksIntegrationConfig,
    backend_handle: Option<SteamBackendHandle>,
    client: Option<Client<ClientManager>>,
    single: Option<Arc<Mutex<SingleClient>>>,
    callbacks_task: Option<JoinHandle<()>>,
}

impl SteamworksIntegration {
    pub fn new(runtime: NetworkRuntime, config: SteamworksIntegrationConfig) -> Self {
        Self {
            runtime,
            config,
            backend_handle: None,
            client: None,
            single: None,
            callbacks_task: None,
        }
    }

    fn spawn_callback_task(
        runtime: &NetworkRuntime,
        single: Arc<Mutex<SingleClient>>,
        handle: SteamBackendHandle,
        interval: Duration,
    ) -> JoinHandle<()> {
        runtime.spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            loop {
                ticker.tick().await;
                let result = {
                    let mut guard = single.lock().await;
                    guard.run_callbacks()
                };
                if let Err(err) = result {
                    warn!(target = "network::discovery", "steam callbacks failed: {err}");
                    let _ = handle.send(SteamServerEvent::Error {
                        message: format!("steam callbacks failed: {err}"),
                    });
                    break;
                }
            }
        })
    }
}

impl SteamIntegration for SteamworksIntegration {
    fn start(&mut self, handle: SteamBackendHandle) -> Result<(), SteamIntegrationError> {
        self.stop();

        let app_id = self.config.app_id;
        debug!(target = "network::discovery", "initialising steamworks app_id={app_id}");
        let (client, single) = Client::init_app(app_id)
            .map_err(|err| SteamIntegrationError::start(err))?;

        let single = Arc::new(Mutex::new(single));
        let callbacks = Self::spawn_callback_task(
            &self.runtime,
            single.clone(),
            handle.clone(),
            self.config.callback_interval,
        );

        self.backend_handle = Some(handle.clone());
        self.client = Some(client);
        self.single = Some(single);
        self.callbacks_task = Some(callbacks);

        let _ = handle.send(SteamServerEvent::Activated);
        Ok(())
    }

    fn stop(&mut self) {
        if let Some(task) = self.callbacks_task.take() {
            task.abort();
        }
        self.single = None;
        self.client = None;
        if let Some(handle) = &self.backend_handle {
            let _ = handle.send(SteamServerEvent::Deactivated);
        }
        self.backend_handle = None;
    }
}

impl Drop for SteamworksIntegration {
    fn drop(&mut self) {
        self.stop();
    }
}
