use super::{NetworkError, PrivateEndpoint, PublicEndpoint};
use std::{collections::HashMap, fs, time::Duration};

pub type Phonebook = HashMap<String, PrivateEndpoint>;

pub fn curl_phonebook_list(
    list: &Vec<PublicEndpoint>,
    timeout: u64,
) -> Result<Phonebook, NetworkError> {
    for endpoint in list.iter() {
        let res = curl_phonebook(&endpoint.vpn_ip, timeout);
        if res.is_ok() {
            return res;
        }
    }
    Err(NetworkError::CurlsFailed(String::from(
        "all curl requests to /phonebook.json failed",
    )))
}

fn curl_phonebook(vpn_ip: &str, timeout: u64) -> Result<Phonebook, NetworkError> {
    let url = format!("http://{}/phonebook.json", vpn_ip);
    let phonebook: Phonebook = ureq::get(&url)
        .timeout(Duration::from_secs(timeout))
        .call()?
        .into_json()?;
    Ok(phonebook)
}

pub fn read_phonebook_file(path: String) -> Result<Phonebook, NetworkError> {
    let priv_id_json = fs::read_to_string(path)?;
    let res = serde_json::from_str(&priv_id_json)?;
    Ok(res)
}
