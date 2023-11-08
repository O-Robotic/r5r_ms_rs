use std::{net::IpAddr, str::FromStr};

pub fn format_ip_to_ipv6(ip: Option<String>) -> Option<String> {
    let ip_adr = IpAddr::from_str(ip?.as_str());
    match ip_adr {
        Ok(ip) => Some(
            match ip {
                IpAddr::V4(ip) => ip.to_ipv6_mapped(),
                IpAddr::V6(ip) => ip,
            }
            .to_string(),
        ),
        Err(_) => None,
    }
}
