use crate::certificate::create_certificate;
enum State {
    Starting,
    Idle,
    Running,
    Stopped,
}
#[derive(Debug)]
struct ServerSettings {
    host: String,
    port: u16,
}

pub struct ServerApp {
    config: ServerSettings,
    tick_rate: f64,
    state: State,
}

impl ServerApp {
    pub fn new(host: String, port: u16, tick_rate: f64) -> Self {
        ServerApp {
            config: ServerSettings { host, port },
            tick_rate,
            state: State::Starting,
        }
    }

    pub fn init(&mut self) {
        self.state = State::Idle;

        println!("Server initialized with config: {:?}", self.config);

        println!("Creating Certificate");
        let (ca, issuer) = create_certificate();
        println!("CA Certificate: {:?}", ca);
        println!("Issuer Certificate: {:?}", issuer);
    }

    pub fn start(&mut self) {
        self.state = State::Running;
        println!(
            "Server started at {}:{}",
            self.config.host, self.config.port
        );
    }

    pub fn stop(&mut self) {
        self.state = State::Stopped;
        println!("Server stopped.");
    }

    pub fn tick(&self) {
        if let State::Running = self.state {
            println!("Ticking server at rate: {}", self.tick_rate);
        } else {
            println!("Server is not running.");
        }
    }
}
