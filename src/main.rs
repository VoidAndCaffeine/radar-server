mod plugins;

fn main() {
    println!("Hello, world!");
    let mut server = plugins::publish_data::Server::new();
    server.run();
}
