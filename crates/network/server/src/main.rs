mod certificate;
mod runtime;
mod server;
mod settings;
mod wizard;

fn main() {
    // wizard::run();
    let mut server = server::ServerApp::new("localhost".to_string(), 8080, 1.0);
    server.init();
}
