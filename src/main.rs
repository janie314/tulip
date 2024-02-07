use clap::{Parser, Subcommand};

mod id;
mod misc;
mod network;
mod tasks;

/// Tulip (tulip.network)
#[derive(Debug, Parser)]
#[command(author, about, version = option_env!("TULIP_VERSION").unwrap_or("dev"))]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Turn Wireguard kernel logging on or off (requires root/sudo privilege)
    Debug {
        #[arg(value_name = "on|off")]
        onoff: Option<String>,
    },
    /// Generate a {private,public}_id.json in your cwd
    GenId {
        /// The nickname associated with your Tulip ID (e.g. miles_spiderkid)
        #[arg(short, long)]
        name: String,
        /// Output directory for the ID Files
        #[arg(short, long, default_value_t = String::from("./"))]
        output: String,
    },
    /// Add a new user to a Tulip network. For use by a Tulip network admin
    AddUser {
        /// The nickname of the Tulip user
        #[arg(short, long)]
        name: String,
        /// Path to this Tulip server's tulip_network.json
        #[arg(long)]
        network: String,
        /// Output directory for the user's Tulip network config file
        #[arg(short, long, default_value_t = String::from("./"))]
        output: String,
        /// Path to phonebook.json
        #[arg(short, long)]
        phonebook: String,
    },
    /// Generate a WireGuard config from a Tulip configuration. For use by Tulip network user
    GenWgConf {
        /// Kind of network config (qr or wg)
        #[arg(short, long, default_value_t = String::from("qr"))]
        kind: String,
        /// Path to this Tulip server's tulip_network.json
        #[arg(long)]
        network: String,
        /// Output directory for the WireGuard config file
        #[arg(short, long, default_value_t = String::from("./"))]
        output: String,
        /// Path to private_id.json
        #[arg(short, long)]
        priv_id: String,
    },
    /// Start a Tulip network
    Start {
        /// Path to tulip_network.json
        #[arg(short, long)]
        network: String,
        /// Path to phonebook.json (required if in server mode)
        #[arg(short, long)]
        phonebook: Option<String>,
        /// Path to private_id.json
        #[arg(short, long)]
        priv_id: String,
        /// Start in server mode (enable ipv4 and ipv6 forwarding kernel parameters)
        #[arg(short, long, default_value_t = false)]
        server: bool,
        /// Timeout for querying the phonebook (seconds)
        #[arg(short, long, default_value_t = 3)]
        timeout: u64,
    },
    /// Stop a Tulip network
    Stop {
        /// Path to tulip_network.json
        #[arg(short, long)]
        network: String,
    },
    /// Testing command. Herein lies DANGER
    Test,
}

fn main() {
    let args = Cli::parse();
    match args.command {
        Commands::AddUser {
            name,
            network,
            output,
            phonebook,
        } => {
            tasks::add_user(output, name, network, phonebook);
        }
        Commands::Debug { onoff } => tasks::debug(onoff),
        Commands::GenId { name, output } => tasks::gen_id(name, output),
        Commands::Start {
            network,
            priv_id,
            server,
            phonebook,
            timeout,
        } => {
            if server && phonebook.is_none() {
                eprintln!("--server mode requires a --phonebook argument");
            } else {
                tasks::start_network(network, priv_id, server, phonebook, timeout);
            }
        }
        Commands::Stop { network } => tasks::stop_network(network),
        Commands::Test => {
            let version = option_env!("CLI_GIT_COMMIT").unwrap_or("dev");
            println!("{version}");
        }
        Commands::GenWgConf {
            kind,
            output,
            priv_id,
            network,
        } => tasks::write_wg_conf_file(&kind, &output, &network, &priv_id),
    }
}
