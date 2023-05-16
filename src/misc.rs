use core::time;
use std::{
    ffi::OsStr,
    fs::{self, OpenOptions},
    io::{self, Write},
    os::unix::prelude::OpenOptionsExt,
    process::{Command, Stdio},
    thread::sleep,
};

pub fn create_private_file(path: &str) -> Result<fs::File, io::Error> {
    OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(true)
        .mode(0o600)
        .open(&path)
}

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

pub fn set_kernel_parameter(path: &str, value: &str) -> Result<(), std::io::Error> {
    fs::write(path, value)
}

pub fn exec<I, S>(cmd: &str, args: I) -> Result<(), std::io::Error>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Command::new(cmd)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?
        .wait()?;
    Ok(())
}

pub fn exec_silent<I, S>(cmd: &str, args: I) -> Result<(), std::io::Error>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Command::new(cmd)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?
        .wait()?;
    Ok(())
}
