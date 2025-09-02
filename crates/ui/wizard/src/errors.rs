use std::env;
use std::sync::OnceLock;

use color_eyre::Result;
use tracing::error;

static INIT: OnceLock<()> = OnceLock::new();

pub fn init() -> Result<()> {
    // idempotent: wenn schon initialisiert, tue nichts
    if INIT.get().is_some() {
        return Ok(());
    }

    // try_into_hooks vermeidet Panic bei bereits gesetztem Theme
    let hooks = color_eyre::config::HookBuilder::default()
        .panic_section(format!(
            "This is a bug. Consider reporting it at {}",
            env!("CARGO_PKG_REPOSITORY")
        ))
        .capture_span_trace_by_default(false)
        .display_location_section(false)
        .display_env_section(false)
        .try_into_hooks()?; // <-- statt into_hooks()

    let (panic_hook, eyre_hook) = hooks;
    eyre_hook.install()?;

    std::panic::set_hook(Box::new(move |panic_info| {
        if let Ok(mut t) = crate::tui::Tui::new() {
            if let Err(r) = t.exit() {
                error!("Unable to exit Terminal: {:?}", r);
            }
        }

        #[cfg(not(debug_assertions))]
        {
            use human_panic::{handle_dump, metadata, print_msg};
            let metadata = metadata!();
            let file_path = handle_dump(&metadata, panic_info);
            print_msg(file_path, &metadata)
                .expect("human-panic: printing error message to console failed");
            eprintln!("{}", panic_hook.panic_report(panic_info));
        }
        let msg = format!("{}", panic_hook.panic_report(panic_info));
        error!("Error: {}", strip_ansi_escapes::strip_str(msg));

        #[cfg(debug_assertions)]
        {
            better_panic::Settings::auto()
                .most_recent_first(false)
                .lineno_suffix(true)
                .verbosity(better_panic::Verbosity::Full)
                .create_panic_handler()(panic_info);
        }

        std::process::exit(libc::EXIT_FAILURE);
    }));

    // Markiere als initialisiert
    let _ = INIT.set(());

    Ok(())
}
