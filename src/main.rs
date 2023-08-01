pub mod constants;
pub mod gui;
pub mod midi;
pub mod p2p;
pub mod settings;

fn main() {
    let (args, mut settings) = settings::get_program_config();
    settings.apply_default_values();

    if args.as_relay {
        println!("Running as relay");
        match p2p::relay::start_relay_loop(settings.relay_port.unwrap(), 42, constants::USE_IPV6) {
            Ok(_) => (),
            Err(e) => println!("Error running relay: {}", e),
        }
        return;
    }

    if args.cli && args.gui {
        panic!("Cannot use both --gui and --cli");
    }

    if args.gui {
        println!("Running GUI");
        match gui::run_app(settings) {
            Ok(_) => (),
            Err(e) => println!("Error running GUI: {}", e),
        }
    } else {
        println!("Running CLI");
        let _ = p2p::client::start_client(
            p2p::client::Mode::Dial,
            44,
            settings.relay_address.unwrap().as_str(),
            settings.relay_port.unwrap(),
            42,
            constants::USE_IPV6,
        );
    }
}
