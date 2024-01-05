use {
    parking_lot,
    shared::{
        ms_config::get_global_config,
        server::{HostInfo, InternalServerData, ServerInfo, ServerWithUID},
    },
    std::{
        net::SocketAddr,
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        },
        time::{Duration, SystemTime, UNIX_EPOCH},
    },
    tracing::{debug, error},
    ring::{rand::{generate, SystemRandom, Random}, digest},
    uuid::Uuid,
    once_cell::sync::OnceCell,
};

static SYSTEM_RANDOM: OnceCell<SystemRandom> = OnceCell::new();

fn get_system_random() -> &'static SystemRandom {
    SYSTEM_RANDOM.get_or_init(SystemRandom::new)
}

pub struct ServerList {
    pub scrub_needed: AtomicBool,
    pub last_scrub_time: std::sync::Mutex<SystemTime>,
    pub public_servers: parking_lot::RwLock<Vec<ServerInfo>>,
    pub hidden_servers: parking_lot::RwLock<Vec<ServerInfo>>,
}

impl ServerList {
    pub fn new() -> Arc<ServerList> {
        let list = Arc::new(ServerList {
            scrub_needed: false.into(),
            last_scrub_time: SystemTime::now().into(),
            public_servers: parking_lot::RwLock::new(Vec::new()),
            hidden_servers: parking_lot::RwLock::new(Vec::new()),
        });
        let srv_list_handle = list.clone();
        std::thread::spawn(move || {
            srv_list_handle.scrub_server_list_thread();
        });
        list
    }
}

impl ServerList {
    pub async fn add_server(
        &self,
        mut server_request: ServerWithUID,
        adr: SocketAddr,
    ) -> Option<HostInfo> {
        let current_time = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_secs(),
            Err(_) => {
                error!("Failed to get current system timestamp");
                return None;
            }
        };

        let timeout_time = current_time + get_global_config().server_timeout as u64;

        //Grab the correct list for the server visibility
        let server_list = match server_request.server.hidden {
            false => &self.public_servers,
            true => &self.hidden_servers,
        };

        //Set the servers ip to the one that made the initial post request
        server_request.server.ip = adr.ip().to_string();

        let mut server_list = server_list.write();
        //The most servers we can ever remove is the number we have so do this to save unneeded allocation
        let mut servers_to_remove: Vec<usize> = Vec::with_capacity(server_list.len());

        debug!("Looking for server with UID {}", server_request.uid);

        //Iterate over all the servers we currently have
        for (i, itr) in server_list.iter_mut().enumerate() {
            debug!("Trying server with uid of {}", itr.internal.uid);
            //If the server is the one we are looking for
            if itr.internal.uid == *server_request.uid {
                debug!("Found server with UID {}", itr.internal.uid);

                itr.server = server_request.server;
                itr.internal.server_expiry_time = timeout_time;

                let host_data = HostInfo {
                    ip: itr.server.ip.clone(),
                    port: itr.server.port,
                    uid: server_request.uid.clone(),
                    token: itr.internal.token,
                };

                //Go through all the servers we added in reverse,
                //we want to remove the highest index first to avoid the shifting messing up the other index's
                for server in servers_to_remove.iter().rev() {
                    server_list.swap_remove(*server);
                }

                return Some(host_data);
            }

            //If the current server isn't the one we are looking for
            if itr.internal.server_expiry_time < current_time {
                //If ths server expired add it to the list of servers that have expired
                servers_to_remove.push(i);
            }
        }

        //Go through all the servers we added in reverse,
        //we want to remove the highest index first to avoid the shifting messing up the other index's
        for i in servers_to_remove.iter().rev() {
            server_list.swap_remove(*i);
        }

        debug!(
            "Did not find server with UID '{}' Pushing new server",
            server_request.uid
        );

        let rand_bytes: Random<[u8;4]> = match generate(get_system_random()) {
            Ok(bytes) => {bytes},
            Err(_) => {return None}
        };

        let mut ctx = digest::Context::new(&digest::SHA1_FOR_LEGACY_USE_ONLY);
        ctx.update(server_request.server.ip.as_bytes());
        ctx.update(&server_request.server.port.to_le_bytes());
        ctx.update(server_request.server.key.as_bytes());
        ctx.update(&rand_bytes.expose());
        let digest = ctx.finish();

        let mut internal_store = InternalServerData {
            uid: String::from_utf8(digest.as_ref().to_ascii_uppercase()).unwrap(),
            server_expiry_time: timeout_time,
            region: String::new(),
            token: None,
            time_stamp: server_request.time_stamp,
        };

        debug!(
            "Next post expected by {}",
            internal_store.server_expiry_time
        );

        if server_request.server.hidden {
            internal_store.token = Some(Uuid::new_v4());
        }

        let server_ip = server_request.server.ip.clone();
        let server_port = server_request.server.port;

        server_list.push(ServerInfo {
            server: server_request.server,
            players: Vec::new(),
            internal: internal_store.clone(),
            kick_list: Vec::new(),
        });

        //Return the info the game expects
        Some(HostInfo {
            ip: server_ip,
            port: server_port,
            uid: internal_store.uid,
            token: internal_store.token,
        })
    }

    pub fn get_public_servers(&self) -> &parking_lot::RwLock<Vec<ServerInfo>> {
        &self.public_servers
    }

    pub fn get_hidden_servers(&self) -> &parking_lot::RwLock<Vec<ServerInfo>> {
        &self.hidden_servers
    }

    pub fn scrub_server_list_thread(&self) {
        loop {
            std::thread::sleep(Duration::from_secs(5));

            let duration_since_last_scrub =
                match SystemTime::now().duration_since(*self.last_scrub_time.lock().unwrap()) {
                    Ok(duration) => duration,
                    Err(_) => {
                        continue;
                    }
                };

            if !self.scrub_needed.load(Ordering::Relaxed)
                && duration_since_last_scrub < Duration::from_secs(1800)
            {
                continue;
            }

            let tme = SystemTime::now().duration_since(UNIX_EPOCH);
            let time = match tme {
                Ok(tme) => tme.as_secs(),
                Err(_) => {
                    eprint!("Failed to get current system timestamp");
                    return;
                }
            };

            {
                let mut server_list = self.public_servers.write();
                server_list.retain(|server| server.internal.server_expiry_time > time);
            }

            {
                let mut server_list = self.hidden_servers.write();
                server_list.retain(|server| server.internal.server_expiry_time > time);
            }

            self.scrub_needed.store(false, Ordering::Relaxed);
            *self.last_scrub_time.lock().unwrap() = SystemTime::now();
        }
    }

    pub fn get_hidden_server(&self, token: Uuid) -> Option<ServerInfo> {
        let servers = self.hidden_servers.read();
        for server in servers.iter() {
            let srv_token = match server.internal.token {
                Some(token) => token,
                None => continue,
            };

            if srv_token == token {
                //Return by copy here because we are gonna lose our read lock
                return Some(server.clone());
            }
        }
        None
    }

    pub fn update_kick_list(&self, uid: String, player_uids: Vec<u64>) -> bool {
        for list in [&self.hidden_servers, &self.public_servers] {
            let mut servers = list.write();
            if let Some(server) = servers.iter_mut().find(|server| server.internal.uid == uid) {
                server.kick_list = player_uids;
                return true;
            }
        }
        false
    }

    pub fn find_server_from_uid(&self, uid: String) -> Option<ServerInfo> {
        for list in [&self.hidden_servers, &self.public_servers] {
            let servers = list.read();
            if let Some(server) = servers.iter().find(|&server| server.internal.uid == uid) {
                return Some(server.clone());
            }
        }
        None
    }

    pub fn does_server_exist(&self, uid: &String) -> bool {
        for list in [&self.hidden_servers, &self.public_servers] {
            let servers = list.read();
            if servers.iter().any(|server| server.internal.uid == *uid)
            {
                return true
            }
        }
        false
    }

}
