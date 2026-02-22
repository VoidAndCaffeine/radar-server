use std::thread;
use std::time::{Duration, SystemTime};
use zmq;
use crate::plugins::defs::*;
use crate::plugins::defs::NetType;
use crate::plugins::source_dummy_data::NormalData;


pub struct Server {
    context: zmq::Context,
    socket: zmq::Socket,
}

pub trait DummyPubServer {
    fn new(ip:&str) -> Self;
    fn broadcast_loop(&mut self);
}

pub trait Subscriber {
    fn new(ip:&str) -> Self;
    fn subscribe_check(&mut self) -> ComPacketFloat;
}

impl DummyPubServer for Server {
    fn new(ip:&str) -> Server{
        let context = zmq::Context::new();
        let socket = context.socket(zmq::PUB).unwrap();
        socket.bind(ip).expect("Could not bind socket.");
        return Server{context, socket};
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
            packet.data = NormalData::source().to_vec();
            s = serde_json::to_string(&packet).expect("Failed to serialize packet");
            self.socket.send(&s,0).expect("Failed to send packet");

            println!("Sent packet: \n{}",s);
            thread::sleep(Duration::from_millis(1000));
        }
    }
}

impl Subscriber for Server {
    fn new(ip:&str) -> Server{
        let context = zmq::Context::new();
        let socket = context.socket(zmq::SUB).unwrap();
        socket.connect(ip).expect("Failed to connect to socket");
        socket.set_subscribe(b"").expect("Failed to subscribe socket");
        println!("Subscribe complete");
        Server{context, socket}
    }
    fn subscribe_check(&mut self) -> ComPacketFloat{
        loop {
            let message = self.socket.recv_msg(0).unwrap();
            let s = message.as_str().unwrap();
            let packet = serde_json::from_str::<ComPacketFloat>(s).unwrap();
            return packet;
        }
    }
}