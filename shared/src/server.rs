use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Default, Debug, Deserialize, Serialize, Clone)]
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

    #[serde(skip_serializing)]
    pub version: String,

    pub hidden: bool,

    #[serde(skip)]
    pub server_id: String,

    #[serde(skip)]
    pub server_expiry_time: u64,
    
    #[serde(skip_serializing)]
    #[serde(rename = "timeStamp")]
    pub time_stamp: u64,

    #[serde(skip_serializing)]
    pub token: Option<Uuid>,

    #[serde(skip_deserializing)]
    pub region: String,
}
