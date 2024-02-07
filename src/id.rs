use serde::{Deserialize, Serialize};
use std::{
    fs::{self},
    io::Write,
    path::Path,
    process::{Command, Stdio},
};
use crate::misc::create_private_file;

#[derive(Debug)]
pub enum IdError {
    FileIO(std::io::Error),
    FromUtf8Error(std::string::FromUtf8Error),
    KeyFileExists(String),
    PipeError(String),
    Serde(serde_json::Error),
}

impl From<std::io::Error> for IdError {
    fn from(e: std::io::Error) -> Self {
        IdError::FileIO(e)
    }
}

impl From<serde_json::Error> for IdError {
    fn from(e: serde_json::Error) -> Self {
        IdError::Serde(e)
    }
}

impl From<std::string::FromUtf8Error> for IdError {
    fn from(e: std::string::FromUtf8Error) -> Self {
        IdError::FromUtf8Error(e)
    }
}

/// Returns a WireGuard private key, as a vector of u8 chars.
fn genkey() -> Result<Vec<u8>, IdError> {
    let mut priv_key = Command::new("wg").arg("genkey").output()?.stdout;
    priv_key.pop();
    Ok(priv_key)
}

/// Given a WireGuard private key, returns its corresponding public key.
fn priv_key_to_pub_key(input: &Vec<u8>) -> Result<String, IdError> {
    let mut pub_key_cmd = Command::new("wg")
        .arg("pubkey")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    pub_key_cmd
        .stdin
        .take()
        .ok_or(IdError::PipeError(String::from("could not read public key from wg")))?
        .write(&input)?;
    let mut res = String::from_utf8(pub_key_cmd.wait_with_output()?.stdout)?;
    res.pop();
    Ok(res)
}

/// Represents a Tulip private id.
#[derive(Deserialize, Serialize, Debug)]
pub struct PrivId {
    pub name: String,
    pub private_key: String,
}

/// Represents a Tulip public id.
#[derive(Serialize, Debug)]
pub struct PubId {
    pub name: String,
    pub public_key: String,
}

pub fn gen_id_files(name: String) -> Result<(), IdError> {
    let priv_key = genkey()?;
    let pub_key = priv_key_to_pub_key(&priv_key)?;
    let priv_key = String::from_utf8(priv_key)?;
    let pub_id_struct = PubId {
        name: name.clone(),
        public_key: pub_key,
    };
    let priv_id_struct = PrivId {
        name: name.clone(),
        private_key: priv_key,
    };
    let pub_id_json = serde_json::to_string_pretty(&pub_id_struct)?;
    let priv_id_json = serde_json::to_string_pretty(&priv_id_struct)?;
    let pub_id_filepath = format!("{}_public_id.json", &name);
    let priv_id_filepath = format!("{}_private_id.json", &name);
    if Path::new(&pub_id_filepath).exists() || Path::new(&priv_id_filepath).exists() {
        Err(IdError::KeyFileExists(format!(
            "quitting; will not overwrite {} and/or {}",
            &pub_id_filepath, &priv_id_filepath
        )))
    } else {
    let mut pub_id_file = create_private_file(&pub_id_filepath)?;
    let mut priv_id_file = create_private_file(&priv_id_filepath)?;
    println!("writing {}", &pub_id_filepath);
    writeln!(pub_id_file, "{}", pub_id_json)?;
    println!("writing {}", &priv_id_filepath);
    writeln!(priv_id_file, "{}", priv_id_json)?;
    Ok(())
    }
}

pub fn read_id_file(path: &str) -> Result<PrivId, IdError> {
    let priv_id_json = fs::read_to_string(path)?;
    let res = serde_json::from_str(&priv_id_json)?;
    Ok(res)
}
