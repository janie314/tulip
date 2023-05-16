use qrcode_generator::QrCodeEcc;

use crate::{
    id,
    misc::{create_private_file, exec_silent, set_kernel_parameter},
    network::{
        self, phonebook,
        wg_conf::{NetworkWgConfInput, WgConfSection},
    },
};
use std::{io::Write, path::Path};

/*
 * Public task functions
 * Functions in this file are allowed to panic
 */
pub fn debug(onoff: Option<String>) {
    let cmd = onoff.unwrap_or_default().to_lowercase();
    if cmd == "on" {
        set_kernel_parameter(
            "/sys/kernel/debug/dynamic_debug/control",
            "module wireguard +p",
        )
        .expect("enable forwarding problem");
    } else if cmd == "off" {
        set_kernel_parameter(
            "/sys/kernel/debug/dynamic_debug/control",
            "module wireguard -p",
        )
        .expect("enable kernel param problem 2");
    } else {
        eprintln!("argument to debug must be \"on\" or \"off\"")
    }
}

pub fn gen_id(name: String) {
    id::gen_id_files(name).expect("gen_id problem");
}

pub fn start_network(
    network_path: String,
    priv_id_path: String,
    server: bool,
    phonebook_path: Option<String>,
    timeout: u64,
) {
    let network = network::read_network_file(&network_path).expect("reading network file problem");
    let priv_id = id::read_id_file(&priv_id_path).expect("priv-id problem");
    let phonebook = if server {
        let path = network::phonebook::read_phonebook_file(phonebook_path.unwrap_or_default())
            .expect("reading phonebook file problem");
        Some(path)
    } else {
        None
    };
    network::start(network, priv_id, server, phonebook, timeout).expect("start network problem");
}

pub fn stop_network(network_path: String) {
    let network = network::read_network_file(&network_path).expect("reading network file problem");
    network::stop(network).expect("stop network problem");
}

pub fn write_network_json_file(
    out_dir: String,
    name: String,
    network_path: String,
    phonebook_path: String,
) {
    let phonebook =
        phonebook::read_phonebook_file(phonebook_path).expect("reading public_id.json issue");
    match phonebook.get(&name) {
        Some(user) => {
            let mut net_conf =
                network::read_network_file(&network_path).expect("reading network file problem");
            net_conf.user.name = name.clone();
            net_conf.user.vpn_ip = user.vpn_ip.clone();
            let net_conf_json = serde_json::to_string_pretty(&net_conf).expect("json issue");
            let out_path_aux = Path::new(&out_dir).join(format!("{}_tulip_network.json", &name));
            let out_path = out_path_aux.to_str().expect("path concat issue");
            let out_file = create_private_file(&out_path).expect("couldn't create output file");
            writeln!(&out_file, "{}", &net_conf_json).expect("couldn't write the json file");
        }
        None => {
            panic!("{} is not a user", &name);
        }
    }
}

pub fn write_wg_conf_file(kind: &str, out_dir: &str, network_path: &str, priv_id_path: &str) {
    let priv_id = id::read_id_file(priv_id_path).expect("reading public_id.json issue");
    let network = network::read_network_file(&network_path).expect("network reading problem");
    let phonebook = phonebook::curl_phonebook_list(&network.public_endpoints, 3)
        .expect("couldn't curl phonebook");
    let name = priv_id.name.clone();
    match phonebook.get(&name) {
        Some(user) => {
            let mut net_conf =
                network::read_network_file(&network_path).expect("reading network file problem");
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
                let out_path_aux = Path::new("/tmp").join(format!("{}_tulip_network.svg", &name));
                let out_path = out_path_aux.to_str().expect("path concat issue");
                let out_file = create_private_file(&out_path).expect("couldn't make out file");
                write!(&out_file, "{}", &qr).expect("couldn't write the network file");
                println!("opening {} with your default SVG viewer", &out_path);
                exec_silent("xdg-open", [&out_path]).expect("couldn't open the svg");
            } else {
                let out_path_aux =
                    Path::new(&out_dir).join(format!("{}_tulip_network.conf", &name));
                let out_path = out_path_aux.to_str().expect("path concat issue");
                let out_file = create_private_file(&out_path).expect("couldn't make out file");
                write!(&out_file, "{}", &wg_conf).expect("couldn't write the network file");
                println!("wrote to {}", &out_path);
            }
        }
        None => {
            panic!("{} is not a user", &name);
        }
    }
}
