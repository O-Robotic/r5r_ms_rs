use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Default, Debug, Deserialize, Serialize, Clone)]
pub struct Server {
    pub name: String,
    pub description: Option<String>,
    pub map: String,
    pub playlist: String,
    
    #[serde(rename = "maxPlayers")]
    pub max_players: String,

    #[serde(rename = "playerCount")]
    pub player_count: String,

    pub ip: String,
    pub port: String,
    pub key: String,

    #[serde(skip_serializing)]
    pub checksum: String,

    #[serde(skip_serializing)]
    pub version: String,

    #[serde(skip_serializing)]
    pub hidden: bool,

    #[serde(skip)]
    pub server_id: String,

    #[serde(skip)]
    pub server_expiry_time: u64,

    #[serde(skip_serializing)]
    pub token: Option<Uuid>,
}
