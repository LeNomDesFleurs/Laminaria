use crate::{outils, ParameterUpdate};
use crossterm::{
    cursor, event, event::Event, event::KeyCode, event::KeyEvent, event::KeyModifiers,
    style::Stylize, terminal,
};
use midir::{Ignore, MidiInput, MidiInputConnection};
use std::collections::HashMap;
use std::error::Error;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use crate::ui::option_menu;

const NOTE_OFF_MASK: u8 = 0b1000_0000;
const NOTE_ON_MASK: u8 = 0b1001_0000;
const POLYPHONIC_KEY_PRESSURE_MASK: u8 = 0b1010_0000;
const CONTROL_CHANGE_MASK: u8 = 0b1011_0000;
const PROGRAM_CHANGE_MASK: u8 = 0b1100_0000;
const CHANNEL_PRESSURE_MASK: u8 = 0b1101_0000;
const PITCH_BEND_MASK: u8 = 0b1110_0000;
const SYSTEM_MASK: u8 = 0b1111_0000;

const SYSEX_START_CODE: u8 = 0b1111_0000;
const MTC_QUARTER_FRAME_CODE: u8 = 0b1111_0001;
const SONG_POSITION_POINTER_CODE: u8 = 0b1111_0010;
const SONG_SELECT_CODE: u8 = 0b1111_0011;
const TUNE_REQUEST_CODE: u8 = 0b1111_0110;
const SYSEX_END_CODE: u8 = 0b1111_0111;

const TIMING_CLOCK_CODE: u8 = 0b1111_1000;
const START_CODE: u8 = 0b1111_1010;
const CONTINUE_CODE: u8 = 0b1111_1011;
const STOP_CODE: u8 = 0b1111_1100;
const ACTIVE_SENSING_CODE: u8 = 0b1111_1110;
const SYSTEM_RESET_CODE: u8 = 0b1111_1111;

pub enum MidiMessage {
    ///note number
    NoteOff(u8),
    ///note number
    NoteOn(u8),
    ControlChange(u8, u8),
    None,
}

//stole from https://github.com/chris-zen/kiro-synth/blob/master/kiro-midi-core/src/decoder.rs
///take message one and two, return the channel and the midiMessage
/// * `status` - first midi message, contain the event type and the channel index
/// * `note` - second midi message, contain the note number / cc index
/// * `velocity` - third midi message, contain the velocity / cc value
fn raw_midi_to_message(status: u8, note: u8, velocity: u8) -> (u8, MidiMessage) {
    let channel = status & 0x0f;
    type MM = MidiMessage;
    match status & 0xf0 {
        NOTE_OFF_MASK => (channel, MM::NoteOff(note)),
        NOTE_ON_MASK => (channel, MM::NoteOn(note)),
        CONTROL_CHANGE_MASK => (channel, MM::ControlChange(note, velocity)),
        _ => (channel, MM::None),
    }
}

use crate::parameters::{Parameters};
use crate::ui::UiEvent;

pub fn connect_midi(
    midi_sender: Sender<MidiMessage>,
    parameter_clone: Arc<Mutex<Parameters>>,
    parameter_sender: Sender<ParameterUpdate>,
    gui_sender: Sender<UiEvent>,
    channel_index: Arc<Mutex<u8>>,
) -> Result<(MidiInputConnection<()>, String), Box<dyn Error>> {
    let mut midicc_hash: HashMap<u8, i32> = HashMap::new();
    for capsule in parameter_clone.lock().unwrap().parameters.iter() {
        let id = capsule.id;
        let cc = capsule.parameter.midicc;
        midicc_hash.insert(outils::get_orca_integer(cc).unwrap_or(0), id);
    }
    let mut selection = 0;
    let mut midi_in = MidiInput::new("midir reading input")?;
    midi_in.ignore(Ignore::None);
    println!("{}", terminal::Clear(terminal::ClearType::All));
    println!("{}", cursor::MoveTo(0, 0));
    // Get an input port (read from console if multiple are available)
    let in_ports = midi_in.ports();
    let number_of_option = in_ports.len();
    let in_port = match number_of_option {
        0 => return Err("no input port found".into()),
        1 => {
            //choose the only connection avaible
            &in_ports[0]
        }
        _ => {
            let mut options:Vec<String>= vec![];
             for (i, p) in in_ports.iter().enumerate() {
                    options.push(midi_in.port_name(p).unwrap());
                }
            selection = option_menu(options);
            in_ports
                .get(selection)
                .ok_or("invalid input port selected")?
        }
    };

    println!("\nOpening connection");

    let port_name = midi_in.port_name(&in_ports[selection as usize]).unwrap();

    // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
    let _conn_in = (
        midi_in.connect(
            in_port,
            "midir-read-input",
            move |_stamp, message, _| {
                let (channel, midi_message): (u8, MidiMessage);
                if message.len()<3{
                    (channel, midi_message) =
                        raw_midi_to_message(message[0], 0, 0);
                }
                else {
                    (channel, midi_message) =
                    raw_midi_to_message(message[0], message[1], message[2]);
                }
                //check if CC
                // println!("{}: {:?}", _stamp, message);
                if channel == *channel_index.lock().unwrap() {
                    match midi_message {
                        MidiMessage::ControlChange(cc, midi_value) => {
                            //try to get parameter name from name table
                            match midicc_hash.get(&cc) {
                                Some(id) => {
                                    //convert midi 127 to orca 36
                                    let orca_value =
                                        ((midi_value as f32 / 127.) * 36.).floor() as i32;
                                    let parameter_binding =
                                        &mut parameter_clone.lock().unwrap()[*id];
                                    parameter_binding.value = orca_value;
                                    let raw_value = parameter_binding.get_raw_value();
                                    parameter_sender.send((*id, raw_value)).unwrap();
                                    gui_sender.send(UiEvent::Refresh).unwrap();
                                }
                                //if cc is not bounded, do nothing
                                None => {}
                            }
                        }
                        MidiMessage::NoteOff(note) => {
                            midi_sender.send(MidiMessage::NoteOff(note)).unwrap()
                        }
                        MidiMessage::NoteOn(note) => {
                            midi_sender.send(MidiMessage::NoteOn(note)).unwrap()
                        }
                        MidiMessage::None => {}
                    }
                }
            },
            (),
        )?,
        port_name,
    );

    Ok(_conn_in)
}
