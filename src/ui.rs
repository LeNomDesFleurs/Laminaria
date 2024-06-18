
use crate::parameters::{Parameter, ParameterID, Parameters, NUMBER_OF_PARAMETERS};
use std::os::macos::raw;
use std::sync::{Arc, Mutex, mpsc::Sender, mpsc::Receiver};
use crossterm::event::KeyModifiers;
use crossterm::{
    style::Stylize,
    cursor, 
    terminal, 
    terminal::enable_raw_mode, 
    terminal::disable_raw_mode,
    event,
    event::KeyEvent,
    event::KeyCode, 
    event::Event
};
use std::collections::HashMap;
use std::io::Result;
use std::io::ErrorKind;
use crate::{parameters, ParameterUpdate};
use parameters::ParameterCapsule;

pub type UiEvent= Option<i32>;


//little enum that allow me to simplify the key_code match by deferring all the mutex work to a more convenient and centralized place
#[derive(Clone)]
enum ParameterModified{
    Increment, 
    Decrement,
    SetValue(char),
}

pub fn interaction(parameters: Arc<Mutex<Parameters>>, param_sender: Sender<ParameterUpdate>, gui_sender: Sender<UiEvent>)->Result<()>{
    
    let mut selected: i32 = 0;
    //use this to get a name vector, il allow me to refer to a parameter via it's index rather than it's name

    let mut parameters_modified: Option<ParameterModified>; 
    enable_raw_mode().unwrap();
    while let Event::Key(KeyEvent { code, .. }) = event::read().unwrap_or(Event::Key(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE))) {
        parameters_modified = None;
        match code {
            KeyCode::Enter => {
                disable_raw_mode().unwrap();
                println!("{}",terminal::Clear(terminal::ClearType::All));
                println!("{}", cursor::MoveTo(0, 0));
                break;
            }
            KeyCode::Down => {selected += 1; selected=selected.clamp(0, NUMBER_OF_PARAMETERS as i32 -1);},
            KeyCode::Up => {selected -= 1; selected=selected.clamp(0, NUMBER_OF_PARAMETERS as i32-1);},
            KeyCode::Right => {parameters_modified=Some(ParameterModified::Increment)}
            KeyCode::Left => {parameters_modified=Some(ParameterModified::Decrement)}
            KeyCode::Char(char) => {parameters_modified=Some(ParameterModified::SetValue(char))}
            // Key::Ctrl('q') => self.should_quit = true,
            _ => {},
        }
        
        //update GUI
        gui_sender.send(Some(selected)).map_err(|_err| std::io::Error::new(ErrorKind::Other,"no gui receiver"))?;
        //update Audio Thread
        parameters_modified.map(|modification|{
                //the first unwrap is in the case where a mutex fucks up, the second is for get_mut and only return None if there is no parameter in the Hash map, which cannot happen
                let mut parameters_binding = parameters.lock().unwrap();
                let capsule_binding = parameters_binding.parameters.get_mut(selected as usize).unwrap();
                let parameter = &mut capsule_binding.parameter;
                let id = capsule_binding.id;
                //apply modification
                match modification{
                    ParameterModified::Increment=>{parameter.increment()}
                    ParameterModified::Decrement=>{parameter.decrement()}
                    ParameterModified::SetValue(char)=>{parameter.set_value(char)}
                }
                //get a copy of the parameter and send it to the audio thread
                param_sender.send((id, parameter.get_raw_value())).unwrap();}
        );   
        }
        Ok(())
}

pub fn ui(parameters: Arc<Mutex<Parameters>>, receive_event: Receiver<UiEvent> ) -> Result<()>{
    let mut local_parameters:Vec<Parameter> = Vec::new();
    let mut selected: i32 = 0;
    let mut top_selection_index = 0;
    let default = (10 as u16, 10 as u16);
    //use this to get a name vector, il allow me to refer to a parameter via it's index rather than it's name
    loop{
        //need to be updated a each iteration to get new values 
        local_parameters.clear();
        for parameter in parameters.lock().unwrap().parameters.iter(){
            local_parameters.push(parameter.parameter.clone());
        }
        let terminal_size = crossterm::terminal::size().unwrap_or(default);
        let bottom = top_selection_index + terminal_size.1 as i32 - 2;
        
        // if (selected - 3) < top_selection_index as usize && top_selection_index > 0 {top_selection_index -= 1}
        // if (selected + 3) > bottom as usize && (bottom < local_parameters.len() as i32){top_selection_index+=1}
        //in top to generate the display at least once at the begginning, and then wait for new input
        update_display(&local_parameters, selected, top_selection_index, terminal_size.1 as i32);
        let event = receive_event.recv().map_err(|_err| std::io::Error::new(ErrorKind::Other,"no gui sender"))?;

        //check if the event contain an new index value, if yes, apply
        selected = match event{ 
            Some(new_index) => {new_index}
            None => {selected}
        }
    }
}
fn update_display(parameters:&Vec<Parameter>, selected:i32, top_selection_index:i32, size: i32){
            println!("{}",terminal::Clear(terminal::ClearType::All));
            println!("{}", cursor::MoveTo(0, 0));
            let iterator;
            // if all the parameters fit in the window, print them all
            if size as usize >= parameters.len() {  
                iterator = 0..parameters.len();
            }else
            //else, implement scroll
            {
                iterator = top_selection_index as usize..(top_selection_index+size-2) as usize
            }   
            for i in iterator{
                if i == selected as usize{
                    print!("{}", parameters[i].build_string().bold().italic());
                }
                else {
                    // TODO add a string memory to avoid rebuilding at each refresh
                    print!("{}", parameters[i].build_string());
                }
                print!{"\r\n"};
    }
}
