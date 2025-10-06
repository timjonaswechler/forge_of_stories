mod assets;
mod fate;
mod themes;

use crate::assets::*;
use crate::fate::sidebar_fate::FateSidebar;
use gpui::Application;

fn main() {
    let app = Application::new().with_assets(Assets);

    app.run(move |cx| {
        fate::init(cx);
        cx.activate(true);

        fate::create_new_window(
            "Gallery of GPUI Component",
            move |window, cx| FateSidebar::view(window, cx),
            cx,
        );
    });
}
