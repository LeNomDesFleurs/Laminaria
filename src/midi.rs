use midir::{Ignore, MidiInput, MidiInputConnection};
use std::io::{self, Read};
use std::sync::mpsc::{Receiver, Sender};
use std::io::{stdin, stdout, Write};
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

use crate::parameters::Parameter;
use crate::ui::UiEvent;

pub fn connect_midi(midi_sender: Sender<[u8; 3]>, parameter_clone: Arc<Mutex<HashMap<String, Parameter>>>, parameter_sender: Sender<Parameter>, gui_sender:Sender<UiEvent>) -> Result<MidiInputConnection<()>, Box<dyn Error>> {
    let mut input = String::new();
    let mut midicc_hash: HashMap<u8, String> = HashMap::new();
    for (name, parameter) in parameter_clone.lock().unwrap().iter(){
        midicc_hash.insert(parameter.midicc, name.to_string());
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
    let in_port_name = midi_in.port_name(in_port)?;

    // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
    let _conn_in = midi_in.connect(
        in_port,
        "midir-read-input",
        move |stamp, message, _| {
            // println!("{}: {:?} (len = {})", stamp, message, message.len());
            //check if CC
            if message[0]==176{
                //convert midi 127 to orca 36
                let value = ((message[3]as f32/127.) *36.).floor() as i32;
                let parameter_name = &midicc_hash[&message[1]];
                let mut parameters = parameter_clone.lock().unwrap();
                if let Some(parameter) = parameters.get_mut(parameter_name){
                    parameter.value=value;
                    parameter_sender.send(parameter.clone()).unwrap();
                    gui_sender.send(UiEvent{new_selection: false, selected_index: 0}).unwrap();
                }
            }
            //else send note
            midi_sender.send([message[0], message[1], message[2]]);
        },
        (),
    )?;

    Ok(_conn_in)
}
