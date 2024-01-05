use {
    serde::{Deserialize, Serialize},
    uuid::Uuid,
};

//This contains all the data we store about a server
//This can then be broken down into smaller structs for different uses / requests
#[derive(Serialize, Clone)]
pub struct ServerInfo {
    pub server: Server,
    pub internal: InternalServerData,
    pub players: Vec<Player>,
    pub kick_list: Vec<u64>,
}

//This contains the details about an individual player that is on a server
#[derive(Serialize, Clone)]
pub struct Player {
    pub name: String,
    pub ip: Option<String>,
    pub uid: Option<u64>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ServerWithUID {
    pub uid: String,
    #[serde(flatten)]
    pub server: Server,
    #[serde(rename = "timeStamp")]
    pub time_stamp: u64,
}

//This is needed as the game needs to know what its own IP is
#[derive(Serialize)]
pub struct HostInfo {
    pub ip: String,
    pub port: u16,
    pub uid: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<Uuid>,
}

//This contains details about the server we handle internally
#[derive(Deserialize, Serialize, Clone, Default)]
pub struct InternalServerData {
    pub uid: String,
    pub server_expiry_time: u64,
    pub region: String,
    pub token: Option<Uuid>,
    #[serde(rename = "timeStamp")]
    pub time_stamp: u64,
}

//Data about the server that the game client needs
#[derive(Deserialize, Serialize, Clone)]
pub struct Server {
    pub name: String,
    pub description: Option<String>,
    pub map: String,
    pub playlist: String,
    #[serde(rename = "maxPlayers")]
    pub max_players: u16,
    #[serde(rename = "playerCount")]
    pub player_count: u16,
    pub ip: String,
    pub port: u16,
    pub key: String,
    pub checksum: u32,
    pub hidden: bool,
    pub version: String,
}
