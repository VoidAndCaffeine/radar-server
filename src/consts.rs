/// The client will be listening on port 5555
pub static CLIENT_PORT: &str = "5555";
/// Control messages are sent on 5556
pub static CONTROL_PORT: &str = "5556";
/// The server broadcasts on 5557
pub static RADAR_PORT: &str = "5557";
/// The tcp address of the server, for the archiver to subscribe to
pub static RADAR_ADDRESS: &str = "tcp://localhost:";
/// Broadcast address
pub static WORLD_ADDRESS: &str = "tcp://*:";
/// The tcp address of the archiver, for the server to connect to for control packets
pub static ARCHIVER_ADDRESS: &str = "tcp://localhost:";
/// Data should be stored and sourced from XDG_DATA regardless of where the program is
pub static DATA_ARCHIVE_DIR: &str = ".local/share/radar-server/";
/// The chunk size for data stored in hdf5 files. Ideally this should match the amount of data in a single packet
pub static DATA_CHUNK_SIZE:usize = 1024;
/// How many seconds of data to store in each file, defaults to one day
pub static ARCHIVE_FILE_CHUNK_SIZE:i64 = 24 * 60 *60;