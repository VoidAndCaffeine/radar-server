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
use std::time::{Duration, SystemTime};
use hdf5_metno::File;
use crate::plugins::publish_data::*;

/// The plugins module contains all logic and datastructure submodules
mod plugins;
use crate::plugins::radar_packet::*;
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
        let mut server: Connection = Server::new_broadcast([WORLD_ADDRESS, RADAR_PORT].concat().as_str());
        let mut settings_channel: Connection = SettingsChannel::new_dealer([ARCHIVER_ADDRESS,CONTROL_PORT].concat().as_str(), "DummyServer");
        let mut settings: Option<ComPacketSettings>;
        let mut dummy = DummyData;

        loop {
            settings =settings_channel.receive_settings();
            if settings.is_some(){
                let u_setting = settings.unwrap();
                if u_setting.first_time {
                    settings_channel.send_settings(&ComPacketSettings {
                        identity: Identity {
                            net_type: NetType::Server,
                            version: VERSION.to_string(),
                        },
                        first_time: true,
                        playback: None,
                        setting: Option::from(dummy.get_state().get_setting()),
                    });
                    continue;
                }
                println!("No settings implemented");
                continue;
            }
            server.broadcast(&dummy.source_complex_data());
            thread::sleep(Duration::from_millis(1000));
        }
    }

    if args.contains(&String::from("--archive")) || args.contains(&String::from("-a")) {
        let mut subscription: Connection = Subscriber::new_subscription([RADAR_ADDRESS, RADAR_PORT].concat().as_str());
        let mut settings_channel: Connection = SettingsChannel::new_router([WORLD_ADDRESS,CONTROL_PORT].concat().as_str());
        let mut client: Connection = Server::new_broadcast([WORLD_ADDRESS, CLIENT_PORT].concat().as_str());
        let mut t_playback = Option::from(SystemTime::now());
        let mut archived = ArchivedData{
            packet_time: SystemTime::now(),
            time_next: None,
            rate: 0f64,
            file: File::open_rw("radar_archive.h5").or_else(|_| File::create("radar_archive.h5"))
                .expect("Unable to open radar_archive.h5 file")
        };
        loop {
            let settings = settings_channel.receive_settings();
            if settings.is_some() {
                let u_settings  = settings.unwrap();
                if u_settings.playback.is_some(){
                    let playback = u_settings.playback.unwrap();
                    archived.time_next = match playback.time {
                        Some(t) => Option::from(t),
                        None => None,
                    };
                    t_playback = playback.time;
                }
            }
            let receive_packet = subscription.subscribe_check();
            //receive_packet.to_hdf5(&archived.file).expect("Unable to write to file");
            if archived.time_next.is_none() {
                println!("Forwarding Live Packet.");
                client.broadcast(&receive_packet);
                continue;
            } else {
                archived.time_next = t_playback;
                //let to_send = archived.source_complex_data();
                //client.broadcast(&to_send);
                continue;
            }
        }
    }
    if args.contains(&String::from("--demo")) || args.contains(&String::from("-e")){
        let mut client: Connection = Server::new_broadcast([WORLD_ADDRESS, CLIENT_PORT].concat().as_str());
        let mut settings_channel: Connection = SettingsChannel::new_router([WORLD_ADDRESS,CONTROL_PORT].concat().as_str());
        let mut demo = DemoData::new();
        loop {
            let p = demo.source_complex_data();
            let s = serde_json::to_string(&p).unwrap();
            println!("{s}");
            client.broadcast(&p);
            thread::sleep(Duration::from_millis(1000));
        }
    }

    println!("Only one of the following args is required: \n\n\
    -h, --help\t Print this help message and exit\n\
    -s, --server\t Run in Server mode\n\
    -d, --dummy\t Run in Dummy Server mode\n\
    -a, --archive\t Run in Archiver mode\n\
    ");
}
