
mod audio;
mod parameters;
mod ui;
mod midi;
mod synth;
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
/* This example expose parameter to pass generator of sample.
Good starting point for integration of cpal into your application.
*/
type ParameterUpdate = (ParameterID, f32);
type MidiEvent = [u8; 3];
use std::error::Error;
extern crate anyhow;
extern crate clap;
extern crate cpal;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, Sender};
use crate::cpal::traits::StreamTrait;
use crossterm::terminal::disable_raw_mode;

pub use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute, queue, style,
    terminal::{self, ClearType},
    Command,
};


fn main() -> Result<(), Box<dyn Error>> {
    // disable_raw_mode().unwrap();

    // use std::panic;
    
    // panic::set_hook(Box::new(|_| {
    //     disable_raw_mode().unwrap();
    // }));
    let (parameter_sender, parameter_receiver):(Sender<ParameterUpdate>, Receiver<ParameterUpdate>) = channel();
    let (ui_sender, ui_receiver):(Sender<ui::UiEvent>, Receiver<ui::UiEvent>) = channel();
    let (midi_sender, midi_receiver):(Sender<MidiEvent>, Receiver<MidiEvent>) = channel();

    let parameters = Parameters::new(); 
    let parameters_mutex = Arc::new(Mutex::new(parameters));
    let parameters_clone_ui = parameters_mutex.clone();
    let parameters_clone_midi = parameters_mutex.clone();
    let parameters_clone_interaction = parameters_mutex.clone();

    let ui_sender_interaction_thread = ui_sender.clone();
    let ui_sender_midi_thread = ui_sender.clone();

    let parameter_sender_midi_thread = parameter_sender.clone();
    let parameter_sender_interaction_thread = parameter_sender.clone();
    

    let stream = audio::stream_setup_for(parameter_receiver, midi_receiver)?;
    stream.play()?;
    
    

    // set default value
    for caps in Parameters::new().parameters{
        parameter_sender.send((caps.id, caps.parameter.get_raw_value()))?
    }

    let _ui_thread= std::thread::Builder::new().name("UI".to_string()).spawn(move||{ui::ui(parameters_clone_ui, ui_receiver)});

    ui::interaction(parameters_clone_interaction, parameter_sender_interaction_thread, ui_sender_interaction_thread, midi_sender);


    Ok(())
}
