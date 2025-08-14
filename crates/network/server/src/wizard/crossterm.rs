use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};

use ratatui::{
    Terminal,
    backend::{Backend, CrosstermBackend},
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
};

use crate::wizard::{app::App, ui};

pub fn run(tick_rate: Duration, enhanced_graphics: bool) -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::new(
        concat!("Forge of Stories - Server v", env!("CARGO_PKG_VERSION")),
        enhanced_graphics,
    );
    let (app_result, final_app) = run_app(&mut terminal, app, tick_rate);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = app_result {
        println!("{err:?}");
    }

    // Check if wizard was completed
    if !final_app.wizard_completed {
        terminal.clear()?;
        panic!(
            "Wizard wurde nicht ordnungsgemäß abgeschlossen! Server kann nicht gestartet werden."
        );
    }

    Ok(())
}

fn run_app<'a, B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App<'a>,
    tick_rate: Duration,
) -> (io::Result<()>, App<'a>) {
    let mut last_tick = Instant::now();
    loop {
        let draw_result = terminal.draw(|frame| ui::draw(frame, &mut app));
        if let Err(e) = draw_result {
            return (Err(e), app);
        }

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if let Ok(true) = event::poll(timeout) {
            if let Ok(Event::Key(key)) = event::read() {
                if key.kind == KeyEventKind::Press {
                    app.on_key(key.code);
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
        if app.should_quit {
            return (Ok(()), app);
        }
    }
}
