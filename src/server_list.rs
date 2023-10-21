use parking_lot;
use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::{SystemTime, UNIX_EPOCH}
};
use uuid::Uuid;

use shared::server::Server;
use shared::ms_config::GLOBAL_CONFIG;

pub struct ServerList {
    pub scrub_list: AtomicBool,
    pub public_servers: parking_lot::RwLock<Vec<Server>>,
    pub hidden_servers: parking_lot::RwLock<Vec<Server>>,
}

impl Default for ServerList {
    fn default() -> Self {
        ServerList {
            scrub_list: false.into(),
            public_servers: parking_lot::RwLock::new(Vec::new()),
            hidden_servers: parking_lot::RwLock::new(Vec::new()),
        }
    }
}

impl ServerList {
    pub fn add_server(&self, mut server: Server) -> Option<Uuid> {

        let tme = SystemTime::now().duration_since(UNIX_EPOCH);
        let current_time = match tme {
            Ok(tme) => tme.as_secs(),
            Err(_) => {
                eprint!("Failed to get current system timestamp");
                return None;
            }
        };

        let timeout_time = current_time + u64::from(GLOBAL_CONFIG.server_timeout);

        let server_list = match server.hidden {
            false => &self.public_servers,
            true => &self.hidden_servers,
        };

        server.server_id = format!("{}:{}:{}",server.ip, server.port, server.key);

        server.server_expiry_time = timeout_time;

        let mut server_list = server_list.write();
        let mut servers_to_remove: Vec<usize> = Vec::<usize>::with_capacity(server_list.len());

        //Why tf is this shit so shit
        for (i, el) in server_list.iter_mut().enumerate() {
            if el.server_id.eq(&server.server_id) {
                server.token = el.token;
                *el = server;
                match el.token {
                    Some(val) => {
                        return Some(val) 
                    },
                    None => {return None}
                };
            } 
            if timeout_time < el.server_expiry_time {
                servers_to_remove.push(i);
            }
        }

        for i in servers_to_remove {
            server_list.remove(i);
        }

        match server.hidden {
            true => {
                let token = Uuid::new_v4();
                server.token = Some(token);
                server_list.push(server);
                return Some(token);
            },
            false => {
                server_list.push(server);
                return None;
            },
        };

    }

    pub fn get_public_servers(&self) -> &parking_lot::RwLock<Vec<Server>> {
        &self.public_servers
    }

    pub fn scrub_server_list(&self) {
        loop {
            if !self.scrub_list.load(Ordering::Relaxed) {
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
                server_list.retain(|server| server.server_expiry_time > time);
            }

            {
                let mut server_list = self.hidden_servers.write();
                server_list.retain(|server| server.server_expiry_time > time);
            }
        }
    }


    pub fn get_hidden_server(&self,token: Uuid) -> Option<Server> {
        let servers = self.hidden_servers.read();
        for server in servers.iter() {
            
            let srv_token = match server.token {
                Some(token) => token,
                None => {continue},
            };
            
            if srv_token == token {
                //Return by copy here because we are gonna lose our read lock
                return Some(server.clone());
            }
        }
        None
    }

}
