use std::thread;
use std::time::{Duration, SystemTime};
use zmq;
use crate::plugins::{defs, defs::DataSource, source_dummy_data};
use crate::plugins::defs::NUM_SAMPLES;
use crate::plugins::source_dummy_data::NormalData;

static DATA_ADDRESS: &str = "tcp://*:5555";
static CONTROL_ADDRESS: &str = "tcp://*:5556";

pub struct Server {
    context: zmq::Context,
    data_socket: zmq::Socket,
}
impl Server {
    pub(crate) fn new() -> Server {
        let context = zmq::Context::new();
        let data_socket = context.socket(zmq::PUB).unwrap();
        assert!(data_socket.bind(DATA_ADDRESS).is_ok());
        Server {
            context,
            data_socket,
        }
    }
    pub(crate) fn run(&mut self){
        loop {
            let random_numbers = NormalData::source().to_vec();
            let serialized: Vec<u8> = serde_pickle::to_vec(&random_numbers, Default::default())
                .expect("Serialization failed");
            self.data_socket.send(serialized.as_slice(), 0).expect("Sending failed");
            println!("Sent data");
            thread::sleep(Duration::from_millis(1000));
        }
    }
}
