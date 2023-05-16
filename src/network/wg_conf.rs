use super::{phonebook::Phonebook, Network, PrivateEndpoint, PublicEndpoint};
use crate::id::PrivId;

pub trait WgConfSection<'a> {
    type Input;
    fn wg_conf_section(&self, arg: Self::Input) -> String;
}

impl<'a> WgConfSection<'a> for PublicEndpoint {
    type Input = &'a str;
    fn wg_conf_section(&self, subnet: &str) -> String {
        format!(
            "# {}\n[Peer]\nPublicKey = {}\nAllowedIPs = {}\nEndpoint = {}:{}",
            self.name, self.public_key, subnet, self.public_hostname, self.port
        )
    }
}

impl WgConfSection<'_> for PrivateEndpoint {
    type Input = ();
    fn wg_conf_section(&self, _: ()) -> String {
        format!(
            "# {}\n[Peer]\nPublicKey = {}\nAllowedIPs = {}/32",
            self.name, self.public_key, self.vpn_ip
        )
    }
}

pub struct NetworkWgConfInput {
    pub priv_id: PrivId,
    pub mobile: bool,
    pub port: Option<u64>,
}

impl<'a> WgConfSection<'a> for Network {
    type Input = &'a NetworkWgConfInput;
    fn wg_conf_section(&self, arg: &NetworkWgConfInput) -> String {
        let mut conf = if arg.mobile {
            format!(
                "[Interface]\nPrivateKey = {}\nAddress = {}/32\n",
                &arg.priv_id.private_key, &self.user.vpn_ip
            )
        } else {
            format!("[Interface]\nPrivateKey = {}\n", &arg.priv_id.private_key)
        };
        if arg.port.is_some() {
            conf.push_str(&format!("ListenPort = {}\n", arg.port.unwrap_or_default()));
        }
        for endpoint in &self.public_endpoints[..] {
            conf.push_str(&format!("{}", endpoint.wg_conf_section(&self.subnet)));
        }
        conf
    }
}

impl WgConfSection<'_> for Phonebook {
    type Input = ();
    fn wg_conf_section(&self, _: ()) -> String {
        self.values()
            .map(|user| user.wg_conf_section(()))
            .collect::<Vec<String>>()
            .join("\n")
    }
}

pub fn get_wg_interface_name(network_name: &str) -> String {
    format!("tulip_{}", &network_name[..8])
}
