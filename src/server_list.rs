use parking_lot;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tracing::debug;
use uuid::Uuid;

use shared::ms_config::get_global_config;
use shared::server::Server;

pub struct ServerList {
    pub scrub_needed: AtomicBool,
    pub last_scrub_time: SystemTime,
    pub public_servers: parking_lot::RwLock<Vec<Server>>,
    pub hidden_servers: parking_lot::RwLock<Vec<Server>>,
}

impl ServerList {
    pub fn new() -> Arc<ServerList> {
        let list = Arc::new(ServerList {
            scrub_needed: false.into(),
            last_scrub_time: SystemTime::now(),
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
    pub fn add_server(&self, mut server: Server) -> Option<Server> {
        let tme = SystemTime::now().duration_since(UNIX_EPOCH);
        let current_time = match tme {
            Ok(tme) => tme.as_secs(),
            Err(_) => {
                eprint!("Failed to get current system timestamp");
                return None;
            }
        };

        let timeout_time = current_time + get_global_config().server_timeout as u64;

        let server_list = match server.hidden {
            false => &self.public_servers,
            true => &self.hidden_servers,
        };

        server.server_id = format!("{}:{}:{}", server.ip, server.port, server.key);

        server.server_expiry_time = timeout_time;

        let mut server_list = server_list.write();
        let mut servers_to_remove: Vec<usize> = Vec::with_capacity(server_list.len());

        for (i, itr) in server_list.iter_mut().enumerate() {
            if itr.server_id == server.server_id {
                server.token = itr.token;
                *itr = server.clone();

                for i in servers_to_remove.iter().rev() {
                    server_list.swap_remove(*i);
                }

                return Some(server);
            }

            if itr.server_expiry_time < current_time {
                servers_to_remove.push(i);
            }
        }

        match server.hidden {
            true => {
                let token = Uuid::new_v4();
                server.token = Some(token);
                server_list.push(server.clone());
            }
            false => {
                server_list.push(server.clone());
            }
        };

        Some(server)
    }

    pub fn get_public_servers(&self) -> &parking_lot::RwLock<Vec<Server>> {
        &self.public_servers
    }

    pub fn scrub_server_list_thread(&self) {
        loop {
            std::thread::sleep(Duration::from_secs(1));

            let duration_since_last_scrub =
                match SystemTime::now().duration_since(self.last_scrub_time) {
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

            debug!("Running server list scrub");
            {
                let mut server_list = self.public_servers.write();
                server_list.retain(|server| server.server_expiry_time > time);
            }

            {
                let mut server_list = self.hidden_servers.write();
                server_list.retain(|server| server.server_expiry_time > time);
            }

            self.scrub_needed.store(false, Ordering::Relaxed);
        }
    }

    pub fn get_hidden_server(&self, token: Uuid) -> Option<Server> {
        let servers = self.hidden_servers.read();
        for server in servers.iter() {
            let srv_token = match server.token {
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
}
