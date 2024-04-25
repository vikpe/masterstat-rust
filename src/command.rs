use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow as e, Result};
use tokio::sync::Mutex;
use zerocopy::FromBytes;

use crate::server_address::{RawServerAddress, ServerAddress, RAW_ADDRESS_SIZE};
use crate::udp;

const SERVERS_COMMAND: [u8; 3] = [0x63, 0x0a, 0x00];
const SERVERS_RESPONSE_HEADER: [u8; 6] = [0xff, 0xff, 0xff, 0xff, 0x64, 0x0a];

/// Get server addresses from a single master server
///
/// # Example
///
/// ```
/// use std::time::Duration;
///
/// let master = "master.quakeworld.nu:27000";
/// let timeout = Some(Duration::from_secs(2));
/// let server_addresses = masterstat::server_addresses(&master, timeout)?;
/// ```
pub fn server_addresses(
    master_address: &str,
    timeout: Option<Duration>,
) -> Result<Vec<ServerAddress>> {
    let response = udp::send_and_receive(master_address, &SERVERS_COMMAND, timeout)?;
    let server_addresses = parse_servers_response(&response)?;
    Ok(sorted_and_unique(&server_addresses))
}

/// Get server addresses from many master servers (async, in parallel)
///
/// # Example
///
/// ```
/// use std::time::Duration;
///
/// let masters = ["master.quakeworld.nu:27000", "master.quakeservers.net:27000"];
/// let timeout = Some(Duration::from_secs(2));
/// let server_addresses = masterstat::server_addresses_from_many(&masters, timeout).await?;
/// ```
pub async fn server_addresses_from_many(
    master_addresses: &[impl AsRef<str>],
    timeout: Option<Duration>,
) -> Vec<ServerAddress> {
    let mut task_handles = vec![];
    let result_mux = Arc::<Mutex<Vec<ServerAddress>>>::default();

    for master_address in master_addresses.iter().map(|a| a.as_ref().to_string()) {
        let result_mux = result_mux.clone();

        let task = tokio::spawn(async move {
            if let Ok(servers) = server_addresses(&master_address, timeout) {
                let mut result = result_mux.lock().await;
                result.extend(servers);
            }
        });
        task_handles.push(task);
    }

    futures::future::join_all(task_handles).await;

    let server_addresses = result_mux.lock().await.clone();
    sorted_and_unique(&server_addresses)
}

fn parse_servers_response(response: &[u8]) -> Result<Vec<ServerAddress>> {
    if !response.starts_with(&SERVERS_RESPONSE_HEADER) {
        return Err(e!("Invalid response"));
    }

    let body = &response[SERVERS_RESPONSE_HEADER.len()..];
    let server_addresses = body
        .chunks(RAW_ADDRESS_SIZE)
        .filter(|b| b.len() == RAW_ADDRESS_SIZE)
        .filter_map(RawServerAddress::read_from)
        .map(ServerAddress::from)
        .collect::<Vec<ServerAddress>>();

    Ok(server_addresses)
}

pub fn sorted_and_unique(server_addresses: &[ServerAddress]) -> Vec<ServerAddress> {
    let mut servers = server_addresses.to_vec();
    servers.sort();
    servers.dedup();
    servers
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_parse_servers_response() -> Result<()> {
        // invalid response header
        {
            let response = [0xff, 0xff];
            let result = parse_servers_response(&response);
            assert_eq!(result.unwrap_err().to_string(), "Invalid response");
        }

        // valid response
        {
            let response = [
                0xff, 0xff, 0xff, 0xff, 0x64, 0x0a, 192, 168, 1, 1, 0x75, 0x30, 192, 168, 1, 2,
                0x75, 0x30,
            ];
            let result = parse_servers_response(&response)?;
            assert_eq!(result.len(), 2);
            assert_eq!(result[0].ip, "192.168.1.1");
            assert_eq!(result[0].port, 30000);
            assert_eq!(result[1].ip, "192.168.1.2");
            assert_eq!(result[1].port, 30000);
        }

        Ok(())
    }

    #[test]
    fn test_sorted_and_unique() {
        let server1_1 = ServerAddress {
            ip: "192.168.1.1".to_string(),
            port: 1,
        };
        let server1_2 = ServerAddress {
            ip: "192.168.1.1".to_string(),
            port: 2,
        };
        let server3 = ServerAddress {
            ip: "192.168.1.3".to_string(),
            port: 1,
        };
        let server4 = ServerAddress {
            ip: "192.168.1.4".to_string(),
            port: 1,
        };
        let servers = vec![
            server4.clone(),
            server4.clone(),
            server4.clone(),
            server1_1.clone(),
            server1_2.clone(),
            server3.clone(),
        ];
        assert_eq!(
            sorted_and_unique(&servers),
            vec![server1_1, server1_2, server3, server4]
        );
    }
}
