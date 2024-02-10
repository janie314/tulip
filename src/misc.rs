use core::time;
use std::{
    ffi::OsStr,
    fs::{self, OpenOptions},
    io::{self, Write},
    os::unix::prelude::OpenOptionsExt,
    path::PathBuf,
    process::{Command, Stdio},
    thread::sleep,
};

/// Open a R/W file handle with 0600 permissions
pub fn create_private_file(path: &PathBuf) -> Result<fs::File, io::Error> {
    OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(true)
        .mode(0o600)
        .open(&path)
}

/// Count down from n seconds
pub fn countdown(n: i64) -> Result<(), std::io::Error> {
    if n <= 0 {
        Ok(())
    } else {
        for i in (1..(n + 1)).rev() {
            print!("{i}...");
            let _ = io::stdout().flush();
            sleep(time::Duration::from_secs(1));
        }
        print!("\n");
        Ok(())
    }
}

/// Write to a Linux kernel parameter virtual file (e.g. `/proc/sys/net/ipv6/idgen_delay`)
pub fn set_kernel_parameter(path: &str, value: &str) -> Result<(), std::io::Error> {
    fs::write(path, value)
}

/// Execute a shell command
pub fn exec<I, S>(cmd: &str, args: I, silent: bool) -> Result<(), std::io::Error>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Command::new(cmd)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(if silent {
            Stdio::null()
        } else {
            Stdio::inherit()
        })
        .stderr(if silent {
            Stdio::null()
        } else {
            Stdio::inherit()
        })
        .spawn()?
        .wait()?;
    Ok(())
}
