use std::env;
use crate::plugins::publish_data::{DummyPubServer, Server, Subscriber};

mod plugins;
use crate::plugins::defs::NetType;

static DATA_ADDRESS: &str = "tcp://*:5556";
static CONTROL_ADDRESS: &str = "tcp://*:5555";
static SUBSCRIPTION_ADDRESS: &str = "tcp://localhost:5556";

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.contains(&String::from("--server")) || args.contains(&String::from("-s")) {
        println!("Server mode is not yet implemented.");
        return;
    }

    if args.contains(&String::from("--dummy")) || args.contains(&String::from("-d")) {
        println!("Running Dummy Server mode.");
        let mut server: Server = plugins::publish_data::DummyPubServer::new(DATA_ADDRESS);
        server.broadcast_loop();
        return;
    }

    if args.contains(&String::from("--archive")) || args.contains(&String::from("-a")) {
        let mut subscription: Server = plugins::publish_data::Subscriber::new(SUBSCRIPTION_ADDRESS);
        loop {
            let packet = subscription.subscribe_check();
            println!("Subscription received: \n{}",serde_json::to_string(&packet).unwrap());
            //ToDo: Write to file
            //ToDo: Send to all clients

        }
        return;
    }

    if args.contains(&String::from("--transform")) || args.contains(&String::from("-t")) {
        println!("Transformer mode is not yet implemented.");
        return;
    }

    println!("Only one of the following args is required: \n\n\
    --help\t\t -h\t Print this help message and exit\n\
    --server\t -s\t Run in Server mode\n\
    --dummy\t\t -d\t Run in Dummy Server mode\n\
    --archive\t -a\t Run in Archiver mode\n\
    --transform\t -t\t Run in Transformer mode\n");

    //let mut server = plugins::publish_data::Server::new();
    //server.broadcast_loop();
}
