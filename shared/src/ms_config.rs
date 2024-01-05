use {
    once_cell::sync::OnceCell,
    serde::{Deserialize, Serialize},
    std::{
        fs::File,
        io::{ErrorKind, Read, Write},
    },
};

const CFG_FILE_PATH: &str = "ms.cfg";

pub static GLOBAL_CONFIG: once_cell::sync::OnceCell<Config> = OnceCell::new();

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub validate_server_conn: bool,
    pub server_timeout: u16,
    pub server_conn_validation_listen_timeout: u16,
    pub server_conn_validation_retry_count: u8,
    pub min_server_name_length: u16,
    pub max_server_name_length: u16,
    pub max_server_description_length: u16,
    pub allowed_chars: String,
    pub allowed_checksums: Vec<u32>,
    pub allowed_sdk_versions: Vec<String>,
    pub ban_fail_condition: bool,
    pub postgres_connection_uri: String,
}

//Creates a blank cfg with default options
impl Default for Config {
    fn default() -> Config {
        Config {
            validate_server_conn: false,
            server_timeout: 30,
            server_conn_validation_listen_timeout: 300,
            server_conn_validation_retry_count: 3,
            min_server_name_length: 3,
            max_server_name_length: 32,
            max_server_description_length: 256,
            allowed_chars: String::from(
                "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_ ",
            ),
            allowed_checksums: Vec::new(),
            allowed_sdk_versions: Vec::new(),
            ban_fail_condition: true,
            postgres_connection_uri: Default::default(),
        }
    }
}

impl Config {
    pub fn from_file() -> Config {
        let file = File::open(CFG_FILE_PATH);

        let mut file = file.unwrap_or_else(|error| {
            if error.kind() == ErrorKind::NotFound {
                // Create the file, write a default set of var's and die
                let mut file: File =
                    File::create(CFG_FILE_PATH).expect("Failed to create cfg file");
                let cfg: Config = Config::from_file();

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

pub fn get_global_config() -> &'static Config {
    //Not sure why this would ever fail, unless it used wrong so im fine with unwrap here tbh
    return GLOBAL_CONFIG.get().unwrap();
}
