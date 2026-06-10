//! # Radar Server
//!
//! The one binary to rule them all for all things related to Professor John Mower's radar system.
//!
//! ## Goals:
//!
//! ### When running as a Server:
//! - Source radar data from the radar hardware in such a way that adding support for new radar systems is easy.
//! - Source simulated radar data from a random function and / or a sample file.
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
//! ## Current ToDos
//! - Improve Archiver Performance, the data is copied a few times and I don't like that.
//! - Fix radar control, decide if the client should ever be allowed to connect to the server, or if all communication should pass through the Archiver
//! - Acquire hardware/hardware simulation
//! - Source Real Server data. (Blocked by Acquire hardware/hardware simulation)
//! - Implement Radar Blanking. (Blocked by Acquire hardware/hardware simulation)
//! - Implement radar control. security.
//! - Finalize radar control (PBlocked by Implement radar control security)
//! - Implement true Server mode. (Blocked by Sourcing Real Server Data and Implement radar control)
use std::{env, fs, thread};
use std::collections::HashSet;
use std::time::{Duration};
use hdf5_metno::File;

/// The plugins module contains all logic and datastructure submodules
mod plugins;
mod consts;

use crate::plugins::radar_packet::*;
use crate::plugins::publish_data::*;
use crate::plugins::source_data::*;
use crate::consts::*;


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
        let identity = Identity {
            net_type: NetType::Server,
            version: VERSION.to_string(),
        };
        let mut demo = DemoData::new();
        let mut server: Connection = Server::new_broadcast([WORLD_ADDRESS, RADAR_PORT].concat().as_str());
        let mut settings_channel: Connection = SettingsChannel::new_dealer([ARCHIVER_ADDRESS,CONTROL_PORT].concat().as_str());
        let mut settings_check;
        settings_channel.send_settings(
            "".as_bytes(),
            SettingsPacket {
                identity: identity.clone(),
                controls: SettingType::SettingInfo(SettingInfo::get_server_settings()),
            });
        let mut i = 0;
        while i < 200 {
            settings_check = settings_channel.check_settings();
            if settings_check.is_some() {
                let message = settings_check.unwrap();
                match serde_json::from_slice::<SettingsPacket>(message[1].as_slice()) {
                    Ok(p) => match p.controls {
                        SettingType::SettingData(s) => {
                            demo.update_state(s);
                        }
                        _ => {}
                    }
                    _ => {}
                }
            }
            server.broadcast(demo.source_complex_data());
            thread::sleep(Duration::from_millis(demo.delay as u64));
            i-=1;
        }
    }

    if args.contains(&String::from("--archive")) || args.contains(&String::from("-a")) {
        let identity = Identity {
            net_type: NetType::Archiver,
            version: VERSION.to_string(),
        };
        let mut subscription: Connection = Subscriber::new_subscription([RADAR_ADDRESS, RADAR_PORT].concat().as_str());
        let mut client: Connection = Server::new_broadcast([WORLD_ADDRESS, CLIENT_PORT].concat().as_str());
        let mut settings_channel: Connection = SettingsChannel::new_router([WORLD_ADDRESS,CONTROL_PORT].concat().as_str());
        let mut clients = HashSet::new();
        let mut servers = HashSet::new();
        let mut settings_check;
        let mut setting_info = SettingInfo::get_archiver_settings();
        let mut data_path = dirs::home_dir().expect("Failed to get home directory");
        data_path.push(DATA_ARCHIVE_DIR);
        let mut file = File::open([DATA_ARCHIVE_DIR, "radar-data.h5"].concat()).unwrap_or_else(|_| {
            fs::create_dir_all(data_path.to_str().unwrap()).expect("Failed to create data dir"); // creates the data dir or does nothing
            File::create([data_path.to_str().unwrap(),"radar-data.h5"].concat()).expect("Couldn't create radar-data.h5 file")
        });

        let mut idx = 0;
        loop {
            settings_check = settings_channel.check_settings();
            if settings_check.is_some() {
                let message = settings_check.unwrap();
                match serde_json::from_slice::<SettingsPacket>(message[1].as_slice()) {
                    Ok(p) => match p.clone().controls{
                        SettingType::SettingInfo(s) => if p.identity.net_type == NetType::Server {
                            setting_info.extend_from_slice(s.as_slice());

                            println!("Adding server {:?} to known servers.", message[0]);
                            servers.insert(message[0].clone());
                        }
                        #[allow(unused)]
                        SettingType::SettingData(s) => {
                            for s_id in servers.clone(){
                                settings_channel.send_settings(s_id.as_slice(),p.clone());
                                //ToDo: update archiver settings
                            }
                        }
                    }
                    _ => {
                        if message[1].len() == 0 {
                            settings_channel.send_settings(
                                message[0].as_slice(),
                                SettingsPacket{
                                    identity: identity.clone(),
                                    controls: SettingType::SettingInfo(setting_info.clone()),
                                });
                            println!("Adding client {:?} to known clients.", message[0]);
                            clients.insert(message[0].clone());
                        }
                    }
                }
                for i in 1..message.len(){
                    println!("Message: {}", std::str::from_utf8(message[i].as_slice()).unwrap());
                }

                for c_id in clients.clone() {
                    settings_channel.send_settings(
                        c_id.as_slice(),
                        SettingsPacket{
                            identity: identity.clone(),
                            controls: SettingType::SettingInfo(setting_info.clone()),
                        });
                }
            }
            let receive_packet = subscription.subscribe_check();
            if receive_packet.is_some() {
                let p = receive_packet.unwrap();
                p.to_hdf5(&mut file).expect("Failed to write packet");
                ComPacket::from_hdf5(idx,&file).expect("Failed to read packet");
                client.broadcast(p);
                idx +=1;
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
