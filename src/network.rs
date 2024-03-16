use std::net::Ipv4Addr;

use reqwest::blocking::Client as RqClient;

pub const SERVERS: &[&str] = &[
    "icanhazip.com",
    "wtfismyip.com",
    "da.gd",
    "seeip.org",
    "ifconfig.co",
    "ipw.cn",
];

pub fn get_outside_ip(
    client: &RqClient,
    preferred_server: Option<&str>,
) -> anyhow::Result<Ipv4Addr> {
    let mut servers = SERVERS.to_vec();
    if let Some(server) = preferred_server {
        servers.insert(0, server);
    }

    let mut ip = None;
    for server_name in servers {
        let server_url = format!("https://{server_name}");
        let response = client.get(&server_url).send()?;
        let response_text = response.text()?;
        match response_text.trim().parse() {
            Ok(parsed_ip) => {
                ip = Some(parsed_ip);
                break;
            }
            Err(_) => continue,
        }
    }

    ip.ok_or_else(|| anyhow::anyhow!("Failed to get outside IP from all servers"))
}
