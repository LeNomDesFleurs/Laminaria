//PERSO
mod audio;
mod midi;
mod parameters;
mod synth;
mod ui;
mod harmonic_model;
pub use harmonic_model::HarmonicModel;
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
extern crate num;
extern crate num_derive;


type ParameterUpdate = (i32, f32);

//std and extern stuff
use std::error::Error;
extern crate anyhow;
extern crate clap;
use crate::clap::Parser;
extern crate cpal;
use crate::cpal::traits::StreamTrait;
use crate::midi::MidiMessage;
use crate::synth::HasParameters;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
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

    // initialize channels
    let (parameter_sender, parameter_receiver): (
        Sender<ParameterUpdate>,
        Receiver<ParameterUpdate>,
    ) = channel();
    let (ui_sender, ui_receiver): (Sender<ui::UiEvent>, Receiver<ui::UiEvent>) = channel();
    let (midi_sender, midi_receiver): (Sender<MidiMessage>, Receiver<MidiMessage>) = channel();

    //initialize parameter system
    let parameters = HarmonicModel::get_parameters();
    let number_of_params = parameters.nb_param;
    let parameters_mutex = Arc::new(Mutex::new(parameters));
    let parameters_clone_ui = parameters_mutex.clone();
    let parameters_clone_interaction = parameters_mutex.clone();

    let ui_sender_interaction_thread = ui_sender.clone();
    let parameter_sender_interaction_thread = parameter_sender.clone();

    // INIT AUDIO THREAD
    let stream = audio::stream_setup_for(parameter_receiver, midi_receiver)?;
    stream.play()?;

    // set default values
    for caps in HarmonicModel::get_parameters().parameters {
        parameter_sender.send((caps.id, caps.parameter.get_raw_value()))?
    }

    // INIT UI THREAD
    let _ui_thread = std::thread::Builder::new()
        .name("UI".to_string())
        .spawn(move || ui::gui(parameters_clone_ui, ui_receiver, number_of_params));
    
    // INIT INTERACTION THREAD
    ui::keyboard_input(
        parameters_clone_interaction,
        parameter_sender_interaction_thread,
        ui_sender_interaction_thread,
        midi_sender,
        midi_channel,
        number_of_params,
    );

    Ok(())
}
