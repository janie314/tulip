/*
 * This file consists of public task functions
 * Functions in this file are allowed to panic
 */

use crate::{
    id,
    misc::{create_private_file, exec, set_kernel_parameter},
    network::{
        self, phonebook,
        wg_conf::{NetworkWgConfInput, WgConfSection},
    },
};
use qrcode_generator::QrCodeEcc;
use std::{io::Write, path::Path};

/// Adds a user to a Tulip network.
/// Updates the server's `phonebook.json` file and outputs a network config file for the user.
///
/// Arguments:
/// - `out_dir` - Output directory to contain `${name}_tulip_network.json`
/// - `name` - Name of the Tulip network
/// - `name` - Name of the Tulip network
pub fn add_user(out_dir: String, name: String, network_path: String, phonebook_path: String) {
    let phonebook =
        phonebook::read_phonebook_file(phonebook_path).expect("could not read public id");
    match phonebook.get(&name) {
        Some(user) => {
            let mut net_conf =
                network::read_network_file(&network_path).expect("could not read network file");
            net_conf.user.name = name.clone();
            net_conf.user.vpn_ip = user.vpn_ip.clone();
            let net_conf_json =
                serde_json::to_string_pretty(&net_conf).expect("could not format json");
            let out_path = Path::new(&out_dir).join(format!("{}_tulip_network.json", &name));
            let out_file = create_private_file(&out_path).expect("could not create private id");
            writeln!(&out_file, "{}", &net_conf_json).expect("could not write network file");
        }
        None => {
            panic!("{} is not a user", &name);
        }
    }
}

/// Enables or disables kernel WireGuard debugging
///
/// Arguments:
/// - `onoff` - Should be `Some("on")` or `Some("off")``, case-insensitive
pub fn debug(onoff: Option<String>) {
    let cmd = onoff.unwrap_or_default().to_lowercase();
    if cmd == "on" {
        set_kernel_parameter(
            "/sys/kernel/debug/dynamic_debug/control",
            "module wireguard +p",
        )
        .expect("could not turn on wg debug mode");
    } else if cmd == "off" {
        set_kernel_parameter(
            "/sys/kernel/debug/dynamic_debug/control",
            "module wireguard -p",
        )
        .expect("could not turn off wg debug mode");
    } else {
        eprintln!("argument to debug must be \"on\" or \"off\"")
    }
}

/// Generates Tulip ID files
///
/// Arguments:
/// - `name` - Name of the user, which will yield the ID name {public,private}
/// - `out_dir` - Directory to write `${name}_{public,private}_id.json`
pub fn gen_id(name: String, out_dir: String) {
    id::gen_id_files(name, out_dir).expect("gen_id problem");
}

/// Starts a Tulip network
///
/// Arguments:
/// - `network_path` - Path to `tulip_network.json` config file
/// - `priv_id_path` - Path to `private_id.json` file
/// - `server` - Whether to run in server mode (enable kernel IP forwarding)
/// - `phonebook_path` - Path to `phonebook.json` file. Only required for server mode
/// - `timeout` - Timeout (secs) for querying `/phonebook.json` via HTTP
pub fn start_network(
    network_path: String,
    priv_id_path: String,
    server: bool,
    phonebook_path: Option<String>,
    timeout: u64,
) {
    let network = network::read_network_file(&network_path).expect("could not read network file");
    let priv_id = id::read_id_file(&priv_id_path).expect("could not read private id");
    let phonebook = if server {
        let path = network::phonebook::read_phonebook_file(phonebook_path.unwrap_or_default())
            .expect("could not read phonebook");
        Some(path)
    } else {
        None
    };
    network::start(network, priv_id, server, phonebook, timeout).expect("could not start network");
}

/// Stops a Tulip network
///
/// Arguments:
/// - `network_path` - Path to `tulip_network.json` config file
pub fn stop_network(network_path: String) {
    let network = network::read_network_file(&network_path).expect("could not read network file");
    network::stop(network).expect("could not stop network");
}

/// Generate a WireGuard config file from a Tulip configuration
pub fn write_wg_conf_file(kind: &str, out_dir: &str, network_path: &str, priv_id_path: &str) {
    let priv_id = id::read_id_file(priv_id_path).expect("could not reading public id");
    let network = network::read_network_file(&network_path).expect("could not read network file");
    let phonebook = phonebook::curl_phonebook_list(&network.public_endpoints, 3)
        .expect("could not curl phonebook");
    let name = priv_id.name.clone();
    match phonebook.get(&name) {
        Some(user) => {
            let mut net_conf =
                network::read_network_file(&network_path).expect("could not read network file");
            net_conf.user.name = name.clone();
            net_conf.user.vpn_ip = user.vpn_ip.clone();
            let mut wg_conf = net_conf.wg_conf_section(&NetworkWgConfInput {
                priv_id,
                mobile: true,
                port: network.user.port,
            });
            wg_conf.push_str(&phonebook.wg_conf_section(()));
            if kind == "qr" {
                let qr: String = qrcode_generator::to_svg_to_string(
                    &wg_conf,
                    QrCodeEcc::Low,
                    1024,
                    None::<&str>,
                )
                .expect("qr code issues");
                let out_path = Path::new("/tmp").join(format!("{}_tulip_network.svg", &name));
                let out_file = create_private_file(&out_path).expect("could not create private id");
                write!(&out_file, "{}", &qr).expect("could not write network file");
                println!(
                    "opening {} with your default SVG viewer",
                    &out_path.to_str().unwrap_or_default()
                );
                exec("xdg-open", [&out_path], true).expect("could not open svg");
            } else {
                let out_path = Path::new(&out_dir).join(format!("{}_tulip_network.conf", &name));
                let out_file = create_private_file(&out_path).expect("could not create private id");
                write!(&out_file, "{}", &wg_conf).expect("could not write network file");
                println!("wrote to {}", &out_path.to_str().unwrap_or_default());
            }
        }
        None => {
            panic!("{} is not a user", &name);
        }
    }
}
