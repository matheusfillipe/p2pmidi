use serde::{Serialize, Deserialize};
use std::env;
use std::io::Cursor;
use std::{fs::File, io::BufReader, path::Path};

use super::midi;

use super::constants;
use clap::Parser;
use clap_serde_derive::ClapSerde;
use skim::prelude::{SkimItemReader, SkimOptionsBuilder};
use skim::Skim;

/// Connect to other nodes creating virtual MIDI output devices for each of them and streaming MIDI
/// from one input device of your choice to all of them.
///
/// Run without arguments will launch the GUI
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Config file
    #[clap(short, long = "config", default_value = constants::DEFAULT_CONFIG_PATH)]
    pub config_path: std::path::PathBuf,

    /// Open in GUI mode.
    #[clap(short = 'g', long = "gui")]
    pub gui: bool,

    /// Prompt for midi input device interactively.
    #[clap(short = 'D', long = "prompt")]
    pub prompt_for_midi_device: bool,

    /// Rest of arguments
    #[clap(flatten)]
    pub settings: <Settings as ClapSerde>::Opt,
}

#[derive(clap::ValueEnum, Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum ThemeType {
    Light,
    Dark,
}


#[derive(ClapSerde, Serialize)]
pub struct Settings {
    /// Give yourself a name. Defaults to your username.
    #[clap(short = 'n', long = "name")]
    pub name: Option<String>,

    /// IP address of node to connect to. Can be suplied multiple times
    #[clap(short = 'i', long = "address")]
    pub ip_addresses: Vec<String>,

    /// Port to connect to. All nodes must use the same port.
    #[clap(short='p', long="port", default_value = constants::DEFAULT_PORT)]
    pub port: u16,

    /// MIDI input device to use.
    #[clap(short = 'd', long = "device")]
    pub midi_device: Option<String>,

    /// Circuit relay address. Use a non default address to connect.
    #[clap(short='r', long="relay", default_value = constants::RELAY_ADDRESS)]
    pub relay_address: String,

    /// Circuit relay port. Use a non default port to connect.
    #[clap(short='P', long="relay_port", default_value = constants::RELAY_PORT)]
    pub relay_port: u16,

    /// GUI theme.
    #[clap(long="theme", value_enum)]
    pub theme: Option<ThemeType>,
}

impl Settings {
    /// Save settings to config file as serde serialized YAML
    pub(crate) fn save(&self) -> Result<String, Box<dyn std::error::Error>> {
        let contents = match serde_yaml::to_string(self) {
            Ok(s) => s,
            Err(err) => return Err(err.into()),
        };

        // Get config file
        let path = shellexpand::tilde(constants::DEFAULT_CONFIG_PATH).into_owned();
        let config_path = Path::new(&path);
        if let Ok(_) = File::open(config_path.to_path_buf()) {
            std::fs::write(config_path.to_path_buf(), contents)?;
        } else {
            // Create directory and empty config file
            let parent = config_path.parent().unwrap();
            std::fs::create_dir_all(parent)?;
            std::fs::write(config_path.to_path_buf(), contents)?;
        }
        Ok(config_path.display().to_string())
    }
}

pub fn parse_config_file(args: &mut Args) -> Settings {
    // Get config file
    let path = shellexpand::tilde(&args.config_path.display().to_string()).into_owned();
    args.config_path = Path::new(&path).to_path_buf();
    if let Ok(f) = File::open(&args.config_path) {
        // Parse config with serde
        match serde_yaml::from_reader::<_, <Settings as ClapSerde>::Opt>(BufReader::new(f)) {
            // merge config already parsed from clap
            Ok(config) => Settings::from(config).merge(&mut args.settings),
            Err(err) => panic!("Error in configuration file:\n{}", err),
        }
    } else {
        // Create directory and empty config file
        let parent = args.config_path.parent().unwrap();
        match std::fs::create_dir_all(parent) {
            Ok(_) => (),
            Err(err) => println!("Error creating config directory:\n{}", err),
        }
        match std::fs::write(&args.config_path, "") {
            Ok(_) => (),
            Err(err) => println!("Error creating config file:\n{}", err),
        }

        // If there is not config file return only config parsed from clap
        Settings::from(&mut args.settings)
    }
}

pub fn get_program_config() -> (Args, Settings) {
    let mut args = Args::parse();
    let mut settings = parse_config_file(&mut args);

    // Prompt for chosing midi device
    if args.prompt_for_midi_device {
        let inputs = match midi::get_midi_input() {
            Ok(i) => i,
            Err(e) => panic!("Error creating midi input: {}", e),
        };
        let items = inputs.join("\n");
        let options = SkimOptionsBuilder::default()
            .height(Some("50%"))
            .multi(false)
            .build()
            .unwrap();

        let item_reader = SkimItemReader::default();
        let items = item_reader.of_bufread(Cursor::new(items));
        let selected_items = Skim::run_with(&options, Some(items))
            .map(|out| out.selected_items)
            .unwrap_or_else(|| Vec::new());

        for item in selected_items {
            println!("Selected item: {}", item.output());
            settings.midi_device = Some(item.output().to_string());
        }
    }

    println!("Listening on {}", settings.port);
    for ip in &settings.ip_addresses {
        println!("Connecting to {}", ip);
    }

    let arglen = env::args().collect::<Vec<String>>().len();
    if !atty::is(atty::Stream::Stdin) || arglen == 1 {
        args.gui = true;
    }
    return (args, settings);
}
