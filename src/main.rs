//PERSO
mod audio;
mod midi;
mod parameters;
mod synth;
mod ui;
use parameters::ParameterID;
use parameters::Parameters;
pub use synth::Synth;
mod oscillator;
pub use oscillator::HarmonicOscillator;
pub use oscillator::Lfo;
mod filter;
pub use filter::Biquad;
mod buffer;
pub use buffer::RingBuffer;
mod chorus;
mod outils;
pub use chorus::Chorus;
mod textparsing;
pub use textparsing::TextCharacteristic;
mod envelope;
mod midibuffer;
mod reverb;

type ParameterUpdate = (ParameterID, f32);

//std and extern stuff
use std::error::Error;
extern crate anyhow;
extern crate clap;
use crate::clap::Parser;
extern crate cpal;
use crate::cpal::traits::StreamTrait;
use crate::midi::MidiMessage;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::env;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long, default_value_t = 0)]
    channel: u8,
}

pub use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute, queue, style,
    terminal::{self, ClearType},
    Command,
};

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let midi_channel: u8;

    if args.channel > 15 {
        midi_channel = 15
    } else {
        midi_channel = args.channel;
    }

    let (parameter_sender, parameter_receiver): (
        Sender<ParameterUpdate>,
        Receiver<ParameterUpdate>,
    ) = channel();
    let (ui_sender, ui_receiver): (Sender<ui::UiEvent>, Receiver<ui::UiEvent>) = channel();
    let (midi_sender, midi_receiver): (Sender<MidiMessage>, Receiver<MidiMessage>) = channel();

    let parameters = Parameters::new();
    let parameters_mutex = Arc::new(Mutex::new(parameters));
    let parameters_clone_ui = parameters_mutex.clone();
    let parameters_clone_interaction = parameters_mutex.clone();

    let ui_sender_interaction_thread = ui_sender.clone();

    let parameter_sender_interaction_thread = parameter_sender.clone();

    let stream = audio::stream_setup_for(parameter_receiver, midi_receiver)?;
    stream.play()?;

    // set default value
    for caps in Parameters::new().parameters {
        parameter_sender.send((caps.id, caps.parameter.get_raw_value()))?
    }

    let _ui_thread = std::thread::Builder::new()
        .name("UI".to_string())
        .spawn(move || ui::gui(parameters_clone_ui, ui_receiver));

    ui::keyboard_input(
        parameters_clone_interaction,
        parameter_sender_interaction_thread,
        ui_sender_interaction_thread,
        midi_sender,
        midi_channel,
    );

    Ok(())
}
