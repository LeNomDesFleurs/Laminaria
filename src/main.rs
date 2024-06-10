
mod audio;
mod parameters;
mod ui;
mod midi;
mod synth;
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
/* This example expose parameter to pass generator of sample.
Good starting point for integration of cpal into your application.
*/

use std::error::Error;
extern crate anyhow;
extern crate clap;
extern crate cpal;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Receiver, Sender};
use crate::cpal::traits::StreamTrait;

pub use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute, queue, style,
    terminal::{self, ClearType},
    Command,
};


fn main() -> Result<(), Box<dyn Error>> {
    let (parameter_sender, parameter_receiver):(Sender<parameters::Parameter>, Receiver<parameters::Parameter>) = channel();
    let (ui_sender, ui_receiver):(Sender<ui::UiEvent>, Receiver<ui::UiEvent>) = channel();
    let (midi_sender, midi_receiver):(Sender<[u8; 3]>, Receiver<[u8; 3]>) = channel();

    let parameters = parameters::get_parameters(); 
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
    
    // need to get the midi as a variable to keep it in scope
    let _midi_connection = match midi::connect_midi(midi_sender, parameters_clone_midi, parameter_sender_midi_thread, ui_sender_midi_thread){
        Ok(midi_connection)=> midi_connection,
        Err(error) => panic!("can't connect to midi: {:?}", error)
    };

    let _ui_thread= std::thread::spawn(move||{ui::ui(parameters_clone_ui, ui_receiver)});

    ui::interaction(parameters_clone_interaction, parameter_sender_interaction_thread, ui_sender_interaction_thread);


    Ok(())
}
