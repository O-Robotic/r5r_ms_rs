use serde::Deserialize;
use serde::Serialize;
use std::fs::File;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;
use lazy_static::lazy_static;

const CFG_FILE_PATH: &str = "ms.cfg";

lazy_static! {
    pub static ref GLOBAL_CONFIG: Config = Config::new();
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub check_server_name: bool,
    pub validate_server_conn: bool,
    pub server_timeout: u16,
    pub server_conn_validation_listen_timeout: u16,
    pub server_conn_validation_retry_count: u8,
    pub min_server_name_length: u16,
    pub max_server_name_length: u16,
    pub allowed_chars: String,
    pub allowed_checksums: Vec<u32>,
    pub server_conn_uid: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    fn default_vals() -> Config {
        Config {
            check_server_name: true,
            validate_server_conn: false,
            server_timeout: 30,
            server_conn_validation_listen_timeout: 300,
            server_conn_validation_retry_count: 3,
            min_server_name_length: 3,
            max_server_name_length: 32,
            allowed_chars: String::from(
                "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_ ",
            ),
            allowed_checksums: Vec::new(),
            server_conn_uid: 0,
        }
    }

    pub fn new() -> Config {
        let file = File::open(CFG_FILE_PATH);

        let mut file = file.unwrap_or_else(|error| {
            if error.kind() == ErrorKind::NotFound {
                // Create the file, write a default set of vars and die
                let mut file: File =
                    File::create(CFG_FILE_PATH).expect("Failed to create cfg file");
                let cfg: Config = Config::default_vals();

                let str: String = serde_json::to_string_pretty(&cfg)
                    .expect("Failed to serialize config to string");

                file.write_all(str.as_bytes())
                    .expect("Failed to write json");

                println!("Cfg file written please check the configuration");
                std::process::exit(0);
            } else {
                panic!("Could not read the cfg file: {:?}", error);
            }
        });

        let mut string: String = Default::default();
        file.read_to_string(&mut string)
            .expect("Failed to read cfg json");

        serde_json::from_str(&string).expect("Unable to load config file")
    }
}
