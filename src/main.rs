pub mod constants;
pub mod gui;
pub mod midi;
pub mod settings;


fn main() {
    let (args, settings) = settings::get_program_config();

    if args.gui {
        println!("Running GUI");
        match gui::run_app(settings) {
            Ok(_) => (),
            Err(e) => println!("Error running GUI: {}", e),
        }
    } else {
        println!("Running CLI");
    }
}
