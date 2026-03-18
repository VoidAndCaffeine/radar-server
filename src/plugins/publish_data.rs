use std::thread;
use std::time::{Duration, SystemTime};
use zmq;
use crate::plugins::radar_packet::*;
use crate::plugins::radar_packet::NetType;
use crate::plugins::source_data::{FloatDataSource, DummyNormalData};

/// Contains the context and socket for a ZMQ connection
pub struct Connection {
    context: zmq::Context,
    socket: zmq::Socket,
}

/// A connection that sends dummy data on the specified ip.
pub trait DummyServer {
    /// Creates a new Dummy Server.
    fn new(ip:&str) -> Self;
    /// Sends simulated radar data on a loop.
    fn broadcast_loop(&mut self);
}

/// A connection that subscribes to the specified ip.
pub trait Subscriber {
    /// Creates a new Subscriber.
    fn new(ip:&str) -> Self;
    /// Checks for new packets on the connection.
    ///
    /// ToDo: currently is a loop, but I don't think that's actually necessary.
    fn subscribe_check(&mut self) -> ComPacketFloat;
}

impl DummyServer for Connection {
    fn new(ip:&str) -> Connection {
        let context = zmq::Context::new();
        let socket = context.socket(zmq::PUB).unwrap();
        socket.bind(ip).expect("Could not bind socket.");
        return Connection {context, socket};
    }

    fn broadcast_loop(&mut self){
        let mut packet: ComPacketFloat = ComPacketFloat {
            id: Identity{
                net_type: NetType::Server,
                version: VERSION.to_string(),
            },
            time: SystemTime::now(),
            state: State{
                range: 0,
                rotation_speed: 0.0,
                blanking: Blanking{
                    x:0,
                    y:0,
                },
                attenuation:0.0,
                tune:0.0,
            },
            data: Vec::new(),
        };
        let mut s:String;
        loop {
            packet.data = DummyNormalData::source().to_vec();
            s = serde_json::to_string(&packet).expect("Failed to serialize packet");
            self.socket.send(&s,0).expect("Failed to send packet");

            println!("Sent packet: \n{}",s);
            thread::sleep(Duration::from_millis(1000));
        }
    }
}

impl Subscriber for Connection {
    fn new(ip:&str) -> Connection {
        let context = zmq::Context::new();
        let socket = context.socket(zmq::SUB).unwrap();
        socket.connect(ip).expect("Failed to connect to socket");
        socket.set_subscribe(b"").expect("Failed to subscribe socket");
        println!("Subscribe complete");
        Connection {context, socket}
    }
    fn subscribe_check(&mut self) -> ComPacketFloat{
        loop {
            //ToDo: see if this loop is actually necessary or if I just forgot to remove it.
            let message = self.socket.recv_msg(0).unwrap();
            let s = message.as_str().unwrap();
            let packet = serde_json::from_str::<ComPacketFloat>(s).unwrap();
            return packet;
        }
    }
}