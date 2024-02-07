use self::{
    phonebook::{curl_phonebook_list, Phonebook},
    wg_conf::{get_wg_interface_name, NetworkWgConfInput, WgConfSection},
};
use crate::{
    id::PrivId,
    misc::{countdown, create_private_file, exec},
};
use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path::Path};
pub mod phonebook;
pub mod wg_conf;

/*
 * Error enum
 */
#[derive(Debug)]
pub enum NetworkError {
    // BadSubnet(String),
    CurlsFailed(String),
    FileIO(std::io::Error),
    MissingPort(String),
    Serde(serde_json::Error),
    Ureq(ureq::Error),
}

impl From<std::io::Error> for NetworkError {
    fn from(e: std::io::Error) -> Self {
        NetworkError::FileIO(e)
    }
}

impl From<ureq::Error> for NetworkError {
    fn from(e: ureq::Error) -> Self {
        NetworkError::Ureq(e)
    }
}

impl From<serde_json::Error> for NetworkError {
    fn from(e: serde_json::Error) -> Self {
        NetworkError::Serde(e)
    }
}

/*
 * The different types of endpoints available in our JSON config files.
 * - PublicEndpoint: a Tulip network endpoint with a Wireguard interface accessible
 *    from the public Internet.
 * - PrivateEndpoint: a Tulip network endpoint only available via the network's internal phonebook.json.
 * - UserEndpoint: YOU, the user joining a Tulip network via WireGuard, using a private key.
 */

#[derive(Serialize, Deserialize, Debug)]
pub struct PublicEndpoint {
    pub name: String,
    pub vpn_ip: String,
    pub public_hostname: String,
    pub public_key: String,
    pub port: i64,
}

#[derive(Deserialize, Debug)]
pub struct PrivateEndpoint {
    pub name: String,
    pub vpn_ip: String,
    pub public_key: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserEndpoint {
    pub name: String,
    pub vpn_ip: String,
    pub port: Option<u64>,
}

/*
 * The main .*_tulip_network.json format
 */
#[derive(Serialize, Deserialize, Debug)]
pub struct Network {
    pub name: String,
    pub subnet: String,
    pub user: UserEndpoint,
    pub public_endpoints: Vec<PublicEndpoint>,
}

pub fn read_network_file(path: &str) -> Result<Network, NetworkError> {
    let network_json = fs::read_to_string(path)?;
    let res = serde_json::from_str(&network_json)?;
    Ok(res)
}

/*
 * start(network, priv_id, server)
 * -------------------------------
 * Start the Tulip `Network`
 * Create /tmp/example_tulip_network.conf and give it to wg-quick.
 * If starting in server mode, set appropriate kernel parameters.
 * Then pause for a moment and curl the phonebook.json from the first
 * available PublicEndpoints, in order.
 */
pub fn start(
    network: Network,
    priv_id: PrivId,
    server: bool,
    phonebook: Option<Phonebook>,
    timeout: u64,
) -> Result<(), NetworkError> {
    if server && (network.user.port.is_none() || phonebook.is_none()) {
        Err(NetworkError::MissingPort(String::from(
            "in --server mode, you need a port",
        )))
    } else {
        add_wg_interface(&network, priv_id, phonebook, timeout)?;
        Ok(())
    }
}

/*
 * stop(network, server)
 * -------------------------------
 * Stop the Tulip `Network`
 * If stopping in server mode, set appropriate kernel parameters.
 */
pub fn stop(network: Network) -> Result<(), NetworkError> {
    let network_name = format!("tulip_{}", &network.name[..8]);
    exec("sudo", ["ip", "link", "delete", "dev", &network_name])?;
    Ok(())
}

fn add_wg_interface(
    network: &Network,
    priv_id: PrivId,
    phonebook: Option<Phonebook>,
    timeout: u64,
) -> Result<(), NetworkError> {
    let network_name = get_wg_interface_name(&network.name);
    /*
     * Create wg interface and set some of its basic properties
     */
    exec(
        "sudo",
        ["ip", "link", "add", &network_name, "type", "wireguard"],
    )?;
    exec(
        "sudo",
        [
            "ip",
            "-4",
            "address",
            "add",
            &format!("{}/32", network.user.vpn_ip),
            "dev",
            &network_name,
        ],
    )?;
    exec(
        "sudo",
        ["ip", "link", "set", "mtu", "1420", "dev", &network_name],
    )?;
    exec("sudo", ["ip", "link", "set", "up", "dev", &network_name])?;
    exec(
        "sudo",
        [
            "ip",
            "-4",
            "route",
            "add",
            &network.subnet,
            "dev",
            &network_name,
        ],
    )?;
    let path = Path::new("/tmp").join(format!("{}.conf", &network_name));
    let mut wg_conf = create_private_file(&path)?;
    /*
     * Add public endpoints to the WireGuard config
     */
    writeln!(
        wg_conf,
        "{}",
        network.wg_conf_section(&NetworkWgConfInput {
            priv_id,
            mobile: false,
            port: network.user.port
        })
    )?;
    exec("sudo", ["wg", "setconf", &network_name, &path])?;
    /*
     * Add phonebook users to the WireGuard config
     * If in server mode, the `phonebook` arg here will be Some
     * Otherwise, it will be none and curled from a WireGuard endpoint
     */
    let phonebook = match phonebook {
        Some(p) => Ok(p),
        None => {
            countdown(3)?;
            curl_phonebook_list(&network.public_endpoints, timeout)
        }
    }?;
    let path = format!("/tmp/{}_phonebook.conf", &network_name);
    let mut phonebook_wg_conf = create_private_file(&path)?;
    writeln!(phonebook_wg_conf, "{}", phonebook.wg_conf_section(()))?;
    exec("sudo", ["wg", "addconf", &network_name, &path])?;
    Ok(())
}
