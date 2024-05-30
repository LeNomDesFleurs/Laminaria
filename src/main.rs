use std::io::{self, Read};
use std::error::Error;
use std::io::{stdin, stdout, Write};

mod audio;
mod parameters;
mod ui;
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
/* This example expose parameter to pass generator of sample.
Good starting point for integration of cpal into your application.
*/

// mod midi;
// pub use midi::Midi;
extern crate anyhow;
extern crate clap;
extern crate cpal;
use std::env;
use std::sync::mpsc::sync_channel;

use std::sync::mpsc::{Receiver, SyncSender};
use termion::{event::Key, input::TermRead};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SizedSample,
};
use cpal::{FromSample, Sample};
use midir::{Ignore, MidiInput, MidiInputConnection};
use std::io::{self, Read};
use crossterm::terminal::enable_raw_mode;
use crossterm::terminal::disable_raw_mode;
use crossterm::style::Stylize;
pub use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute, queue, style,
    terminal::{self, ClearType},
    Command,
};



fn midi(midi_sender: SyncSender<[u8; 3]>) -> Result<MidiInputConnection<()>, Box<dyn Error>> {
    let mut input = String::new();

    let mut midi_in = MidiInput::new("midir reading input")?;
    midi_in.ignore(Ignore::None);

    // Get an input port (read from console if multiple are available)
    let in_ports = midi_in.ports();
    let in_port = match in_ports.len() {
        0 => return Err("no input port found".into()),
        1 => {
            println!(
                "Choosing the only available input port: {}",
                midi_in.port_name(&in_ports[0]).unwrap()
            );
            &in_ports[0]
        }
        _ => {
            println!("\nAvailable input ports:");
            for (i, p) in in_ports.iter().enumerate() {
                println!("{}: {}", i, midi_in.port_name(p).unwrap());
            }
            print!("Please select input port: ");
            stdout().flush()?;
            let mut input = String::new();
            stdin().read_line(&mut input)?;
            in_ports
                .get(input.trim().parse::<usize>()?)
                .ok_or("invalid input port selected")?
        }
    };

    println!("\nOpening connection");
    let in_port_name = midi_in.port_name(in_port)?;

    // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
    let _conn_in = midi_in.connect(
        in_port,
        "midir-read-input",
        move |stamp, message, _| {
            println!("{}: {:?} (len = {})", stamp, message, message.len());
            midi_sender.send([message[0], message[1], message[2]]);
        },
        (),
    )?;

    Ok(_conn_in)
}

fn main() -> anyhow::Result<()> {
    let (interface_sender, interface_receiver) = sync_channel(1);
    let (midi_sender, midi_receiver):(SyncSender<[u8; 3]>, Receiver<[u8; 3]>) = sync_channel(100);


    let stream = stream_setup_for(interface_receiver, midi_receiver)?;
    stream.play()?;
    
    // need to get the midi as a variable to keep it in scope
    let _midi_connection = match midi(midi_sender){
        Ok(midi_connection)=> midi_connection,
        Err(error) => panic!("Problem opening the file: {:?}", error)
    };

    // Handle keyboard input to switch frequency on key press
    let mut string_input = "test";
    let mut buffer = String::new();
    let stdin = io::stdin(); // We get `Stdin` here.
    loop {
        buffer.clear();
        match stdin.read_line(&mut buffer) {
            Ok(n) => {
                println!("{n} bytes read");
                println!("{buffer}");
            }
            Err(error) => println!("error: {error}"),
        }
        buffer.trim();
        string_input = &buffer as &str;

        // string_input = io::stdin() as String;
        let textcarac = crate::textparsing::parse_text(string_input);
        println!(
            "{} {} {} {}",
            textcarac.number_of_consonant,
            textcarac.number_of_space,
            textcarac.number_of_vowel,
            textcarac.number_of_special_character
        );
        interface_sender.send(textcarac)?
    }
}
