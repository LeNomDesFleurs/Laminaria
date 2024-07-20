use crate::outils::get_orca_character;
use crate::parameters::{Parameter, ParameterID, Parameters, NUMBER_OF_PARAMETERS};
use crate::{parameters, ParameterUpdate};
use crossterm::execute;
use crossterm::{
    cursor, event, event::Event, event::KeyCode, event::KeyEvent, event::KeyModifiers,
    style::Stylize, terminal, terminal::disable_raw_mode, terminal::enable_raw_mode,
};
use std::io::ErrorKind;
use std::io::Result;
use std::sync::{mpsc::Receiver, mpsc::Sender, Arc, Mutex};
// pub type UiEvent = Option<i32>;
use crate::midi::{connect_midi, MidiMessage};

///Event sent by the keyboard loop and midi callback to update et refresh the UI
pub enum UiEvent {
    UpdateSelection(i32),
    Refresh,
    UpdateMidiportName(String),
    UpdateMidiChannel(u8),
}

//little enum that allow me to simplify the key_code match by deferring all the mutex work to a more convenient and centralized place
#[derive(Clone)]
enum ParameterModified {
    Increment,
    Decrement,
    SetValue(char),
}

pub fn keyboard_input(
    parameters: Arc<Mutex<Parameters>>,
    param_sender: Sender<ParameterUpdate>,
    gui_sender: Sender<UiEvent>,
    midi_sender: Sender<MidiMessage>,
    midi_channel: u8,
) -> Result<()> {
    enable_raw_mode().unwrap();
    execute!(std::io::stdout(), cursor::Hide)?;

    let mut selected: i32 = 0;
    let midi_channel = Arc::new(Mutex::new(midi_channel));
    // need to get the midi as a variable to keep it in scope
    let mut _midi_connection = match connect_midi(
        midi_sender.clone(),
        parameters.clone(),
        param_sender.clone(),
        gui_sender.clone(),
        midi_channel.clone(),
    ) {
        Ok((midi_connection, port_name)) => {
            gui_sender
                .send(UiEvent::UpdateMidiportName(port_name))
                .unwrap();
            midi_connection
        }
        Err(error) => panic!("can't connect to midi: {:?}", error),
    };

    gui_sender
        .send(UiEvent::Refresh)
        .map_err(|_err| std::io::Error::new(ErrorKind::Other, "no gui receiver"))?;
    gui_sender
        .send(UiEvent::UpdateMidiChannel(*midi_channel.lock().unwrap()))
        .map_err(|_err| std::io::Error::new(ErrorKind::Other, "no gui receiver"))?;
    

    let mut parameters_modified: Option<ParameterModified>;
    let mut ui_event = UiEvent::Refresh;
    loop {
        if let Event::Key(KeyEvent { code, .. }) =
            event::read().unwrap_or(Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE)))
        {
            //got to refresh at the end, will be modified if the event involve more modification
            ui_event = UiEvent::Refresh;
            parameters_modified = None;
            match code {
                KeyCode::Esc => {
                    disable_raw_mode().unwrap();
                    execute!(std::io::stdout(), cursor::Show)?;
                    println!("{}", terminal::Clear(terminal::ClearType::All));
                    println!("{}", cursor::MoveTo(0, 0));
                    break;
                }
                KeyCode::Down => {
                    if selected < NUMBER_OF_PARAMETERS as i32 - 1 {
                        selected += 1;
                    }
                    ui_event = UiEvent::UpdateSelection(selected);
                }
                KeyCode::Up => {
                    if selected > 0 {
                        selected -= 1;
                    }
                    ui_event = UiEvent::UpdateSelection(selected);
                }
                KeyCode::Right => parameters_modified = Some(ParameterModified::Increment),
                KeyCode::Left => parameters_modified = Some(ParameterModified::Decrement),
                KeyCode::Char(char) => {
                    if char == '<' {
                        let mut midi_chan = midi_channel.lock().unwrap();
                        //min channel 0
                        if *midi_chan > 0 {
                            *midi_chan -= 1;
                            ui_event = UiEvent::UpdateMidiChannel(*midi_chan);
                        }
                    } else if char == '>' {
                        let mut midi_chan = midi_channel.lock().unwrap();
                        //max channel 15
                        if *midi_chan < 15 {
                            *midi_chan += 1;
                            ui_event = UiEvent::UpdateMidiChannel(*midi_chan);
                        }
                    } else {
                        parameters_modified = Some(ParameterModified::SetValue(char))
                    }
                }
                KeyCode::Tab => {
                    _midi_connection = match connect_midi(
                        midi_sender.clone(),
                        parameters.clone(),
                        param_sender.clone(),
                        gui_sender.clone(),
                        midi_channel.clone(),
                    ) {
                        Ok((midi_connection, port_name)) => {
                            ui_event = UiEvent::UpdateMidiportName(port_name);
                            *midi_channel.lock().unwrap() = 0;
                            midi_connection
                        }
                        Err(error) => panic!("can't connect to midi: {:?}", error),
                    };
                }
                // Key::Ctrl('q') => self.should_quit = true,
                _ => {}
            }

            //update GUI
            gui_sender
                .send(ui_event)
                .map_err(|_err| std::io::Error::new(ErrorKind::Other, "no gui receiver"))?;

            //update Audio Thread if a parameter is modified
            parameters_modified.map(|modification| {
                //the first unwrap is in the case where a mutex fucks up, the second is for get_mut and only return None if there is no parameter in the Hash map, which cannot happen
                let mut parameters_binding = parameters.lock().unwrap();
                let capsule_binding = parameters_binding
                    .parameters
                    .get_mut(selected as usize)
                    .unwrap();
                let parameter = &mut capsule_binding.parameter;
                let id = capsule_binding.id;
                //apply modification
                match modification {
                    ParameterModified::Increment => parameter.increment(),
                    ParameterModified::Decrement => parameter.decrement(),
                    ParameterModified::SetValue(char) => parameter.set_value(char),
                }
                //get a copy of the parameter and send it to the audio thread
                param_sender.send((id, parameter.get_raw_value())).unwrap();
            });
        }
    }
    Ok(())
}

pub fn gui(parameters: Arc<Mutex<Parameters>>, receive_event: Receiver<UiEvent>) -> Result<()> {
    let mut local_parameters: Vec<Parameter> = Vec::new();
    let mut midi_channel: u8 = 0;
    let mut midi_port_name: String = "midi port".to_string();
    let mut selected: i32 = 0;
    let mut top_selection_index = 0;
    let default = (10 as u16, 10 as u16);
    //use this to get a name vector, il allow me to refer to a parameter via it's index rather than it's name
    loop {
        let event = receive_event
            .recv()
            .map_err(|_err| std::io::Error::new(ErrorKind::Other, "no gui sender"))?;

        //check if the event contain an new index value, if yes, apply
        type UI = UiEvent;
        match event {
            UI::UpdateSelection(new_index) => selected = new_index,
            UI::UpdateMidiChannel(channel) => midi_channel = channel,
            UI::UpdateMidiportName(port_name) => {
                midi_port_name = port_name;
                midi_channel = 0
            }
            UI::Refresh => {}
        };
        //need to be updated a each iteration to get new values
        local_parameters.clear();
        for parameter in parameters.lock().unwrap().parameters.iter() {
            local_parameters.push(parameter.parameter.clone());
        }
        let terminal_size = crossterm::terminal::size().unwrap_or(default);
        //magic number to compensate the title bar (midi port and channel)
        let bottom = top_selection_index + terminal_size.1 as i32 - 4;

        if (selected - 2) < top_selection_index && top_selection_index > 0 {
            top_selection_index -= 1
        }
        if (selected + 2) > bottom && (bottom < local_parameters.len() as i32) {
            top_selection_index += 1
        }
        //in top to generate the display at least once at the begginning, and then wait for new input
        update_display(
            &local_parameters,
            midi_channel,
            &midi_port_name,
            selected,
            top_selection_index,
            terminal_size.1 as i32,
        );
    }
}
fn update_display(
    parameters: &Vec<Parameter>,
    midi_channel: u8,
    midi_port_name: &String,
    selected: i32,
    top_selection_index: i32,
    size: i32,
) {
    println!("{}", terminal::Clear(terminal::ClearType::All));
    println!("{}", cursor::MoveTo(0, 0));
    let orca_midi_channel = get_orca_character(midi_channel as i32);
    print! {"{} --- {} {}", midi_port_name, "channel".to_string().italic(), orca_midi_channel.unwrap()};
    print! {"\r\n"};
    print! {"\r\n"};
    let iterator;
    // if all the parameters fit in the window, print them all
    if size as usize >= parameters.len() {
        iterator = 0..parameters.len();
    } else
    //else, implement scroll
    {
        let bottom = (top_selection_index + size - 4).clamp(0, NUMBER_OF_PARAMETERS as i32);
        iterator = top_selection_index as usize..bottom as usize
    }
    for i in iterator {
        if i == selected as usize {
            print!("{}", parameters[i].build_string().bold().italic());
        } else {
            // TODO add a string memory to avoid rebuilding at each refresh
            print!("{}", parameters[i].build_string());
        }
        print! {"\r\n"};
    }
}
