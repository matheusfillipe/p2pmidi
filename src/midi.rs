use std::error::Error;

use midir::{Ignore, MidiInput, MidiOutput};

pub fn display_devices() -> Result<(), Box<dyn Error>> {
    let mut midi_in = MidiInput::new("midir test input")?;
    midi_in.ignore(Ignore::None);
    let midi_out = MidiOutput::new("midir test output")?;

    println!("Available input ports:");
    for (i, p) in midi_in.ports().iter().enumerate() {
        println!("{}: {}", i, midi_in.port_name(p)?);
    }

    println!("\nAvailable output ports:");
    for (i, p) in midi_out.ports().iter().enumerate() {
        println!("{}: {}", i, midi_out.port_name(p)?);
    }

    Ok(())
}

pub fn get_midi_list<T: midir::MidiIO>(midi: &T) -> Vec<String> {
    midi
        .ports()
        .iter()
        .map(|p| midi.port_name(p).unwrap_or("Unknown".to_string()))
        .into_iter()
        .collect::<Vec<String>>()
}

pub fn get_midi_list_from_result<T: midir::MidiIO>(
    midi: Result<T, midir::InitError>,
) -> Result<Vec<String>, String> {
    match midi {
        Ok(m) => Ok(get_midi_list(&m)),
        Err(e) => return Err(format!("Error creating midi input: {}", e)),
    }
}

pub fn get_midi_input() -> Result<Vec<String>, String> {
    get_midi_list_from_result(MidiInput::new("midir test input"))
}

pub fn get_midi_output() -> Result<Vec<String>, String> {
    get_midi_list_from_result(MidiOutput::new("midir test output"))
}
