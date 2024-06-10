use midir::{Ignore, MidiInput, MidiInputConnection};
use std::sync::mpsc::Sender;
use std::io::{stdin, stdout, Write};
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::outils;

use crate::parameters::Parameter;
use crate::ui::UiEvent;

pub fn connect_midi(midi_sender: Sender<[u8; 3]>, parameter_clone: Arc<Mutex<HashMap<String, Parameter>>>, parameter_sender: Sender<Parameter>, gui_sender:Sender<UiEvent>) -> Result<MidiInputConnection<()>, Box<dyn Error>> {
    let mut midicc_hash: HashMap<u8, String> = HashMap::new();
    for (name, parameter) in parameter_clone.lock().unwrap().iter(){
        midicc_hash.insert(outils::get_orca_integer(parameter.midicc).unwrap_or(0), name.to_string());
    }

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

    // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
    let _conn_in = midi_in.connect(
        in_port,
        "midir-read-input",
        move |_stamp, message, _| {
            // println!("{}: {:?} (len = {})", stamp, message, message.len());
            //check if CC
            if message[0]==176{
                //try to get parameter name from name table
                match midicc_hash.get(&message[1]) {
                    Some(parameter_name) =>{
                    //convert midi 127 to orca 36
                    let value = ((message[2] as f32/127.) *36.).floor() as i32;
                    let mut parameter_binding = parameter_clone.lock().unwrap();
                    let parameter = parameter_binding.get_mut(parameter_name).unwrap();
                        parameter.value=value;
                        parameter_sender.send(parameter.clone()).unwrap();
                        gui_sender.send(None).unwrap();}
                    //if cc is not bounded, do nothing
                    None=>{}
                }
            }
            //else send note
            // unwrap cause I can't return the error in this closure (so maybe I shouldn't use it here altogether 😬)
            midi_sender.send([message[0], message[1], message[2]]).unwrap();
        },
        (),
    )?;

    Ok(_conn_in)
}
