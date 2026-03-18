//! # Radar Server
//!
//! The one binary to rule them all for all things related to Professor John Mower's radar system.
//!
//! ## Goals:
//!
//! ### When running as a Server:
//! - Source radar data from the radar hardware in such a way that adding support for new radar systems is easy.
//! - Source simulated radar data from a random function.
//! - Send radar data to clients through a ZMQ Publisher Subscriber connection.
//! - Listen as a ZMQ router, and adjust radar control commands from authorized clients.
//!
//! ### When running as an Archiver:
//! - Subscribe to a Server's published data.
//! - Archive that data in an HDF5 file.
//! - Send live data to the appropriate clients as a ZMQ router.
//! - Listen as a ZMQ router for requests for archived data.
//! - Respond as a ZMQ router with archived data.
//! - Listen as a ZMQ router and forward server commands as a dealer.
//!
//! ### When running as a Transformer:
//! - Receive live or archived data from the Archiver as a ZMQ dealer.
//! - Listen as a ZMQ router and forward server commands as a dealer.
//! - Listen as a ZMQ router client requests.
//! - Request the appropriate live or historical data from the archiver.
//! - Transform the data if requested.
//! - Send the potentially transformed, live or historical data to the client as a router.
//!
//! ## Current ToDos
//! - Check if the Subscriber trait implementation actually needs to be a loop. (It probably doesn't)
//! - Missing ZMQ Router Implementation.
//! - Send live packets as Archiver. (Blocked by Missing ZMQ Router Implementation)
//! - Archiver is not Multithreaded.
//! - Send Archived packets as Archiver. (Blocked by Missing ZMQ Router Implementation and Archiver is not Multithreaded)
//! - Figure out what the transformer actually does.
//! - Implement Transform mode. (Blocked by Figure out what the transformer actually does)
//! - Add `NUMSAMPLES` to the network spec
//! - Move `NUMSAMPLES` in source_data into one of the radar_packet structs. (Blocked by Add `NUMSAMPLES` to the network spec)
//! - Acquire hardware/hardware simulation
//! - Source Real Server data. (Blocked by Acquire hardware/hardware simulation)
//! - Finalize Radar Blanking Values. (Blocked by Acquire hardware/hardware simulation)
//! - Implement Radar Control. (Blocked by Finalize Radar Blanking Values)
//! - Finalize radar control security. (Blocked by Implement Radar Control)
//! - Add cryptography data to radar_packet (Blocked by Finalize radar control security)
//! - Implement radar control (Blocked by Add cryptography data to radar packet)
//! - Implement Server mode. (Blocked by Sourcing Real Server Data and Implement radar control)
use std::env;
use hdf5_metno::File;
use crate::plugins::publish_data::{DummyServer, Connection, Subscriber};

/// The plugins module contains all logic and datastructure submodules
mod plugins;
use crate::plugins::radar_packet::{Hdf5Object};

///  The address to which all radar data is sent, tcp://*:5556
static DATA_ADDRESS: &str = "tcp://*:5556";
///  The UNUSED address to which all control data is sent and received, tcp://*:5555
static CONTROL_ADDRESS: &str = "tcp://*:5555";
///  The address to which all archiver data is received, tcp://localhost:5556
static SUBSCRIPTION_ADDRESS: &str = "tcp://localhost:5556";
///  The UNUSED address on which the archiver and transformer communicate, tcp://localhost:5557
static TRANSFORMER_ADDRESS: &str = "tcp://localhost:5557";

/// Main handles argument parsing and calling the necessary submodules.
fn main() {
    //!
    //! The main function takes the following arguments:\
    //!     -h, --help      Print this help message and exit\
    //!     -s, --server    Run in Server mode\
    //!     -d, --dummy     Run in Dummy Server mode\
    //!     -a, --archive   Run in Archiver mode\
    //!     -t, --transform Run in Transformer mode\
    //! currently only Dummy mode and archiver mode do anything.
    //!
    //! ## Server Mode
    //! Sources real radar data and sends it to the DATA_ADDRESS.
    //!
    //! ToDo: needs to be implemented
    //!
    //! ## Dummy Mode
    //! Sources simulated radar data and sends it to the DATA_ADDRESS.
    //!
    //! ## Archive Mode
    //! Sources radar packets from the SUBSCRIPTION_ADDRESS, archives each packet and forwards the live data the DATA_ADDRESS.
    //! Additionally will send archived data to individual clients on request.
    //!
    //! ToDo: Sending archived data as well as forwarding live data is not yet implemented
    //!
    //! ToDo: Additionally, should run on multiple threads, 1 to receive from a server instance, 1 to forward live data, 1 to handle communicating with a transformer instance, and 1 for each client wanting archived data.
    //!
    //! ## Transform Mode
    //! Sources data from an archiver instance of this program, transforms it based on the transformation requested by the client.
    //!
    //! ToDo: needs to be implemented
    let args: Vec<String> = env::args().collect();

    if args.contains(&String::from("--server")) || args.contains(&String::from("-s")) {
        println!("Server mode is not yet implemented.");
        todo!();
    }

    if args.contains(&String::from("--dummy")) || args.contains(&String::from("-d")) {
        println!("Running Dummy Server mode.");
        let mut server: Connection = DummyServer::new(DATA_ADDRESS);
        server.broadcast_loop();
        return;
    }

    if args.contains(&String::from("--archive")) || args.contains(&String::from("-a")) {
        let mut subscription: Connection = Subscriber::new(SUBSCRIPTION_ADDRESS);
        let file = File::open_rw("radar_archive.h5").or_else(|_| File::create("radar_archive.h5"))
            .expect("Unable to open radar_archive.h5 file");
        loop {
            let packet = subscription.subscribe_check();
            println!("Subscription received: \n{}",serde_json::to_string(&packet).unwrap());
            packet.to_hdf5(&file).expect("Unable to write to file");
            //ToDo: Send to all clients
        }
    }

    if args.contains(&String::from("--transform")) || args.contains(&String::from("-t")) {
        println!("Transformer mode is not yet implemented.");
        todo!();
    }

    println!("Only one of the following args is required: \n\n\
    -h, --help\t\t Print this help message and exit\n\
    -s, --server\t Run in Server mode\n\
    -d, --dummy\t\t Run in Dummy Server mode\n\
    -a, --archive\t Run in Archiver mode\n\
    -t, --transform\t Run in Transformer mode\n");

    //let mut server = plugins::publish_data::Server::new();
    //server.broadcast_loop();
}
