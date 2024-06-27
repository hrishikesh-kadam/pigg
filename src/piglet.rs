use crate::hw::{BCMPinNumber, PinLevel};
use anyhow::Context;
use clap::{Arg, ArgMatches, Command};
use env_logger::Builder;
use futures_lite::StreamExt;
use hw::config::HardwareConfig;
use hw::Hardware;
use iroh_net::{key::SecretKey, relay::RelayMode, Endpoint};
use log::{info, trace, LevelFilter};
use std::env;
use std::str::FromStr;

mod hw;

const PIGLET_ALPN: &[u8] = b"pigg/piglet/0";

/// Piglet will expose the same functionality from the GPIO Hardware Backend used by the GUI
/// in Piggy, but without any GUI or related dependencies, loading a config from file and
/// over the network.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let matches = get_matches();

    setup_logging(&matches);

    let mut hw = hw::get();
    info!("\n{}", hw.description().unwrap().details);
    trace!("Pin Descriptions:");
    for pin_description in hw.description().unwrap().pins.iter() {
        trace!("{pin_description}")
    }

    // Load any config file specified on the command line, or else the default
    let config = match matches.get_one::<String>("config-file") {
        Some(config_filename) => {
            let config = HardwareConfig::load(config_filename).unwrap();
            info!("Config loaded from file: {config_filename}");
            trace!("{config}");
            config
        }
        None => {
            info!("Default Config loaded");
            HardwareConfig::default()
        }
    };

    hw.apply_config(&config, input_level_changed)?;
    trace!("Configuration applied to hardware");

    listen().await
}

/// Callback function that is called when an input changes level
fn input_level_changed(bcm_pin_number: BCMPinNumber, level: PinLevel) {
    info!("Pin #{bcm_pin_number} changed level to '{level}'");
}

/// Setup logging with the requested verbosity level
fn setup_logging(matches: &ArgMatches) {
    let default = String::from("error");
    let verbosity_option = matches.get_one::<String>("verbosity");
    let verbosity = verbosity_option.unwrap_or(&default);
    let level = LevelFilter::from_str(verbosity).unwrap_or(LevelFilter::Error);
    let mut builder = Builder::from_default_env();
    builder.filter_level(level).init();
}

/// Parse the command line arguments using clap
fn get_matches() -> ArgMatches {
    let app = Command::new(env!("CARGO_BIN_NAME")).version(env!("CARGO_PKG_VERSION"));

    let app = app.about(
        "'piglet' - for making Raspberry Pi GPIO hardware accessible remotely using 'piggui'",
    );

    let app = app.arg(
        Arg::new("verbosity")
            .short('v')
            .long("verbosity")
            .num_args(1)
            .number_of_values(1)
            .value_name("VERBOSITY_LEVEL")
            .help(
                "Set verbosity level for output (trace, debug, info, warn, error (default), off)",
            ),
    );

    let app = app.arg(
        Arg::new("config-file")
            .num_args(0..)
            .help("Path of a '.pigg' config file to load"),
    );

    app.get_matches()
}

/// Listen for an incoming iroh-net connection
async fn listen() -> anyhow::Result<()> {
    let secret_key = SecretKey::generate();
    println!("secret key: {secret_key}");

    // Build a `Endpoint`, which uses PublicKeys as node identifiers, uses QUIC for directly connecting to other nodes, and uses the relay servers to holepunch direct connections between nodes when there are NATs or firewalls preventing direct connections. If no direct connection can be made, packets are relayed over the relay servers.
    let endpoint = Endpoint::builder()
        // The secret key is used to authenticate with other nodes. The PublicKey portion of this secret key is how we identify nodes, often referred to as the `node_id` in our codebase.
        .secret_key(secret_key)
        // set the ALPN protocols this endpoint will accept on incoming connections
        .alpns(vec![PIGLET_ALPN.to_vec()])
        // `RelayMode::Default` means that we will use the default relay servers to holepunch and relay.
        // Use `RelayMode::Custom` to pass in a `RelayMap` with custom relay urls.
        // Use `RelayMode::Disable` to disable holepunching and relaying over HTTPS
        // If you want to experiment with relaying using your own relay server, you must pass in the same custom relay url to both the `listen` code AND the `connect` code
        .relay_mode(RelayMode::Default)
        // you can choose a port to bind to, but passing in `0` will bind the socket to a random available port
        .bind(0)
        .await?;

    let me = endpoint.node_id();
    println!("node id: {me}");
    println!("node listening addresses:");

    let local_addrs = endpoint
        .direct_addresses()
        .next()
        .await
        .context("no endpoints")?
        .into_iter()
        .map(|endpoint| {
            let addr = endpoint.addr.to_string();
            println!("\t{addr}");
            addr
        })
        .collect::<Vec<_>>()
        .join(" ");

    let relay_url = endpoint.home_relay()
    .expect("should be connected to a relay server, try calling `endpoint.local_endpoints()` or `endpoint.connect()` first, to ensure the endpoint has actually attempted a connection before checking for the connected relay server");

    println!("node relay server url: {relay_url}");
    println!("\nin a separate terminal run:");

    println!(
        "\tcargo run --example connect-unreliable -- --node-id {me} --addrs \"{local_addrs:?}\" --relay-url {relay_url}\n"
    );

    // accept incoming connections, returns a normal QUIC connection
    while let Some(mut conn) = endpoint.accept().await {
        let alpn = conn.alpn().await?;
        let conn = conn.await?;
        let node_id = iroh_net::endpoint::get_remote_node_id(&conn)?;
        // TODO below will need String::from_utf8_lossy(&alpn), when next release is out
        info!(
            "new (unreliable) connection from {node_id} with ALPN {} (coming from {})",
            String::from_utf8_lossy(&alpn),
            conn.remote_address()
        );

        // spawn a task to handle reading and writing from/to the connection
        tokio::spawn(async move {
            // use the `quinn` API to read a datagram off the connection, and send a datagram in return
            while let Ok(message) = conn.read_datagram().await {
                let message = String::from_utf8(message.into())?;
                println!("received: {message}");

                let message = format!("hi! you connected to pigglet@{me}.");
                conn.send_datagram(message.as_bytes().to_vec().into())?;
            }

            Ok::<_, anyhow::Error>(())
        });
    }
    // stop with SIGINT (ctrl-c)

    Ok(())
}
