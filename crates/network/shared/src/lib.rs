mod env;
mod messaging;

// pub(crate) enum RuntimeContext {
//     Client,
//     LocalServer,
//     DedicatedServer,
// }

// impl Default for RuntimeContext {
//     #[cfg(feature = "local_server")]
//     fn default() -> Self {
//         RuntimeContext::LocalServer
//     }
//     #[cfg(feature = "dedicated_server")]
//     fn default() -> Self {
//         RuntimeContext::DedicatedServer
//     }
//     #[cfg(feature = "client")]
//     fn default() -> Self {
//         RuntimeContext::Client
//     }
//     #[cfg(not(any(
//         feature = "client",
//         feature = "local_server",
//         feature = "dedicated_server"
//     )))]
//     fn default() -> Self {
//         RuntimeContext::Client
//     }
// }
