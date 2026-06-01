use std::thread;
use std::time::{Duration, SystemTime};
use zmq;
use zmq::Message;
use crate::plugins::radar_packet::*;
use rmp_serde::{to_vec, to_vec_named};

/// Contains the context and socket for a ZMQ connection
pub struct Connection {
    context: zmq::Context,
    socket: zmq::Socket,
}

/// A connection that sends dummy data on the specified ip.
pub trait Server {
    /// Creates a new Dummy Server.
    fn new_broadcast(ip:&str) -> Self;
    /// Sends simulated radar data on a loop.
    fn broadcast(&mut self,packet: &ComPacket );
}

/// A connection that subscribes to the specified ip.
pub trait Subscriber {
    /// Creates a new Subscriber.
    fn new_subscription(ip:&str) -> Self;
    /// Checks for new packets on the connection.
    fn subscribe_check(&mut self) -> ComPacket;
}

pub trait SettingsChannel {
    fn new_router(ip:&str) -> Self;
    fn new_dealer(ip:&str,whoami:&str) -> Self;
    fn send_settings(&mut self,dealerid:Vec<u8>, settings: &ComPacketSettings);
    fn check_settings(&mut self) -> Option<Vec<Vec<u8>>>;
}


impl Server for Connection {
    fn new_broadcast(ip:&str) -> Connection {
        let context = zmq::Context::new();
        let socket = context.socket(zmq::PUB).unwrap();
        socket.bind(ip).expect("Could not bind socket.");
        Connection {context, socket}
    }

    fn broadcast(&mut self, packet: &ComPacket) {
        let pak = to_vec_named(packet).expect("Could not serialize packet.");
        self.socket.send(pak,0).expect("Failed to send packet.");

        println!("Sent packet");
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
    fn subscribe_check(&mut self) -> ComPacket{
        let message = self.socket.recv_msg(0).unwrap();
        rmp_serde::from_slice(&message).expect("Could not deserialize message.")
    }
}

impl SettingsChannel for Connection {
    fn new_router(ip:&str) -> Connection {
        let context = zmq::Context::new();
        let socket = context.socket(zmq::ROUTER).unwrap();
        socket.bind(ip).expect("Could not bind socket.");
        println!("Settings channel router bound to {}",ip);
        Connection {context, socket }
    }
    fn new_dealer(ip:&str,whoami:&str) -> Connection {
        let context = zmq::Context::new();
        let socket = context.socket(zmq::DEALER).unwrap();
        socket.set_identity(whoami.as_bytes()).expect("Failed to set identity");
        socket.connect(ip).expect("Failed to connect to socket");
        println!("Settings channel dealer {} connected to {}",whoami,ip);
        Connection {context, socket}
    }
    fn send_settings(&mut self, dealer_id:Vec<u8>, settings:&ComPacketSettings) {
        self.socket.send(dealer_id, zmq::SNDMORE).expect("failed to target dealer");
        self.socket.send(&serde_json::to_string(settings).expect("Failed to serialize settings"), 0)
            .expect("Failed to send packet");
    }

    fn check_settings(&mut self) -> Option<Vec<Vec<u8>>>{
        match zmq::poll(&mut [self.socket.as_poll_item(zmq::POLLIN)], 10) {
            Ok(event) if event > 0 => {
                println!("Event: {}",event);
                Some(self.socket.recv_multipart(0).unwrap())
            }
            _ => None
        }
    }
}
