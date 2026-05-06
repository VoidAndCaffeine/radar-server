use std::cmp::PartialEq;
use std::thread;
use std::time::{Duration, SystemTime};
use zmq;
use zmq::PollItem;
use crate::plugins::radar_packet::*;
use crate::plugins::source_data::{FloatDataSource, DummyData, ComplexDataSource};

/// Contains the context and socket for a ZMQ connection
pub struct Connection {
    context: zmq::Context,
    socket: zmq::Socket,
}

/// A connection that sends dummy data on the specified ip.
pub trait DummyServer {
    /// Creates a new Dummy Server.
    fn new_broadcast(ip:&str) -> Self;
    /// Sends simulated radar data on a loop.
    fn broadcast(&mut self,packet: &mut ComPacketIntComplex );
}

/// A connection that subscribes to the specified ip.
pub trait Subscriber {
    /// Creates a new Subscriber.
    fn new_subscription(ip:&str) -> Self;
    /// Checks for new packets on the connection.
    ///
    fn subscribe_check(&mut self) -> ComPacketIntComplex;
}

pub trait SettingsChannel {
    fn new_router(ip:&str) -> Self;
    fn new_dealer(ip:&str,whoami:&str) -> Self;
    fn send_settings(&mut self, settings: &ComPacketSettings);
    fn request_date(&mut self, time:SystemTime);
    fn receive_settings(&mut self) -> Option<ComPacketSettings > ;
}

pub trait TransformerPair {
    fn bind_pair(ip:&str) -> Self;
    fn connect_pair(ip:&str) -> Self;
    fn send(&mut self, packet: &ComPacketIntComplex);
    fn receive(&mut self) -> ComPacketIntComplex;
}

impl DummyServer for Connection {
    fn new_broadcast(ip:&str) -> Connection {
        let context = zmq::Context::new();
        let socket = context.socket(zmq::PUB).unwrap();
        socket.bind(ip).expect("Could not bind socket.");
        Connection {context, socket}
    }

    fn broadcast(&mut self, packet: &mut ComPacketIntComplex) {
        self.socket.send(
            &serde_json::to_string(&packet)
                .expect("Failed to serialize packet"),
            0,
        ).expect("Failed to send packet");

        println!("Sent packet: \n{}", &serde_json::to_string(&packet)
            .expect("Failed to serialize packet"));
        thread::sleep(Duration::from_millis(1000));
    }
}

impl Subscriber for Connection {
    fn new_subscription(ip:&str) -> Connection {
        let context = zmq::Context::new();
        let socket = context.socket(zmq::SUB).unwrap();
        socket.connect(ip).expect("Failed to connect to socket");
        socket.set_subscribe(b"").expect("Failed to subscribe socket");
        println!("Subscribe complete");
        Connection {context, socket}
    }
    fn subscribe_check(&mut self) -> ComPacketIntComplex{
        let message = self.socket.recv_msg(0).unwrap();
        let s = message.as_str().unwrap();
        serde_json::from_str::<ComPacketIntComplex>(s).unwrap()
    }
}

impl SettingsChannel for Connection {
    fn new_router(ip:&str) -> Connection {
        let context = zmq::Context::new();
        let socket = context.socket(zmq::ROUTER).unwrap();
        socket.bind(ip).expect("Could not bind socket.");
        Connection {context, socket }
    }
    fn new_dealer(ip:&str,whoami:&str) -> Connection {
        let context = zmq::Context::new();
        let socket = context.socket(zmq::DEALER).unwrap();
        socket.set_identity(whoami.as_bytes()).expect("Failed to set identity");
        socket.connect(ip).expect("Failed to connect to socket");
        Connection {context, socket}
    }
    fn send_settings(&mut self, settings:&ComPacketSettings) {
        self.socket.send(&serde_json::to_string(settings).expect("Failed to serialize settings"), 0)
            .expect("Failed to send packet");
    }

    fn request_date(&mut self, time: SystemTime) {
        let packet: ComPacketSettings = ComPacketSettings {
            id: Identity{
                net_type: NetType::Client,
                version: VERSION.to_string(),
            },
            first_time: false,
            playback: Option::from(ArchivedPlayback {time:Option::from(time)}),
            setting: None,
        };
        self.send_settings(&packet);
    }

    fn receive_settings(&mut self) -> Option<ComPacketSettings>{
        if self.socket.as_poll_item(zmq::POLLIN).is_readable() {
            Option::from(serde_json::from_str::<ComPacketSettings>(
                self.socket.recv_msg(zmq::DONTWAIT).unwrap().as_str().unwrap()
            ).unwrap())
        } else {None}
    }
}

impl TransformerPair for Connection {
    fn bind_pair(ip: &str) -> Self {
        let context = zmq::Context::new();
        let socket = context.socket(zmq::PAIR).unwrap();
        socket.bind(ip).expect("Failed to bind socket.");
        Connection {context, socket}
    }

    fn connect_pair(ip: &str) -> Self {
        let context = zmq::Context::new();
        let socket = context.socket(zmq::PAIR).unwrap();
        socket.connect(ip).expect("Failed to connect to socket.");
        Connection {context, socket}
    }

    fn send(&mut self, packet: &ComPacketIntComplex) {
        self.socket.send(
            &serde_json::to_string(packet).expect("Failed to serialize packet"),
            0,
        ).expect("Failed to send packet");
    }

    fn receive(&mut self) -> ComPacketIntComplex {
        let message = self.socket.recv_msg(0).unwrap();
        let s = message.as_str().unwrap();
        serde_json::from_str::<ComPacketIntComplex>(s).unwrap()
    }
}
