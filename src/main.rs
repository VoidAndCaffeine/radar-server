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
use std::{env, thread};
use std::collections::HashSet;
use std::time::{Duration, SystemTime};
use hdf5_metno::File;
use serde_json::Value;
use crate::plugins::publish_data::*;

/// The plugins module contains all logic and datastructure submodules
mod plugins;
use crate::plugins::radar_packet::*;
use crate::plugins::radar_packet::NetType;
use crate::plugins::source_data::*;

static CLIENT_PORT: &str = "5555";
static CONTROL_PORT: &str = "5556";
static RADAR_PORT: &str = "5557";
static RADAR_ADDRESS: &str = "tcp://localhost:";
static WORLD_ADDRESS: &str = "tcp://*:";
static ARCHIVER_ADDRESS: &str = "tcp://localhost:";

/// Main handles argument parsing and calling the necessary submodules.
fn main() {
    //!
    //! The main function takes the following arguments:\
    //!     -h, --help      Print this help message and exit\
    //!     -s, --server    Run in Server mode\
    //!     -d, --dummy     Run in Dummy Server mode\
    //!     -a, --archive   Run in Archiver mode\
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
    let args: Vec<String> = env::args().collect();

    if args.contains(&String::from("--server")) || args.contains(&String::from("-s")) {
        println!("Server mode is not yet implemented.");
        todo!();
    }

    if args.contains(&String::from("--dummy")) || args.contains(&String::from("-d")) {
        println!("Running Dummy Server mode.");
        let mut demo = DemoData::new();
        let mut server: Connection = Server::new_broadcast([WORLD_ADDRESS, RADAR_PORT].concat().as_str());
        let mut settings_channel: Connection = SettingsChannel::new_dealer([ARCHIVER_ADDRESS,CONTROL_PORT].concat().as_str());
        let mut settings_check;
        settings_channel.send_settings(
            "".as_bytes(),
            &Settings::SettingsInfo(SettingsInfo {
                identity: Identity {
                    net_type: NetType::Server,
                    version: VERSION.to_string(),
                },
                server_controls: ServerSetting::get_server_settings(),
                controls: None,
            }));
        loop {
            settings_check = settings_channel.check_settings();
            if settings_check.is_some() {
                let message = settings_check.unwrap();
                for i in 0..message.len() {
                    match serde_json::from_slice::<SettingData>(message[i].as_slice()) {
                        Ok(set) => {
                        }
                        Err(_) => {}
                    }
                }
            }
            let p = demo.source_complex_data();
            server.broadcast(&p);
            thread::sleep(Duration::from_millis(demo.delay));
        }
    }

    if args.contains(&String::from("--archive")) || args.contains(&String::from("-a")) {
        let mut subscription: Connection = Subscriber::new_subscription([RADAR_ADDRESS, RADAR_PORT].concat().as_str());
        let mut client: Connection = Server::new_broadcast([WORLD_ADDRESS, CLIENT_PORT].concat().as_str());
        let mut settings_channel: Connection = SettingsChannel::new_router([WORLD_ADDRESS,CONTROL_PORT].concat().as_str());
        let mut clients = HashSet::new();
        let mut servers = HashSet::new();
        let mut settings_check;
        let mut settings = SettingsInfo {
            identity: Identity {
                net_type: NetType::Archiver,
                version: VERSION.to_string(),
            },
            server_controls: None,
            controls: ArchiverSetting::get_archiver_settings(),
        };

        loop {
            settings_check = settings_channel.check_settings();
            if settings_check.is_some() {
                let message = settings_check.unwrap();
                match serde_json::from_slice::<Settings>(&message[1].as_slice()) {
                    Ok(Settings::SettingsInfo(s)) => {
                        if s.identity.net_type == NetType::Server {
                            println!("Adding server {:?} to known servers", message[0]);
                            servers.insert(message[0].clone());
                            settings.server_controls = s.server_controls.clone();
                        }
                    }
                    Ok(Settings::SettingsData(s)) => {
                        if s.server_controls.is_some(){}
                        if s.controls.is_some(){}
                    }
                    Err(_) => if message[1].len() == 0 && !clients.contains(&message[0]) {
                        println!("Adding client {:?} to known clients", message[0]);
                        clients.insert(message[0].clone());
                    }
                }
                for dealer_id in clients.clone() {
                    settings_channel.send_settings(
                        dealer_id.as_slice(),
                        &Settings::SettingsInfo(settings.clone()),
                    );
                }
            }
            let receive_packet = subscription.subscribe_check();
            if receive_packet.is_some() {
            }
        }
    }

    println!("Only one of the following args is required: \n\n\
    -h, --help\t Print this help message and exit\n\
    -s, --server\t Run in Server mode\n\
    -d, --dummy\t Run in Dummy Server mode\n\
    -a, --archive\t Run in Archiver mode\n\
    ");
}
