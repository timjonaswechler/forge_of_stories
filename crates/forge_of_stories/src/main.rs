mod fos_app;

use crate::fos_app::FOSApp;
use app::AppBuilder;

fn main() {
    let mut app = AppBuilder::<FOSApp>::new(env!("CARGO_PKG_VERSION"))
        .expect("Failed to initialize application")
        .build_with_bevy(|mut app, _ctx| {
            app.add_plugins(bevy::prelude::DefaultPlugins);
            app
        });
    
    app.run();
}
