
use crate::parameters::Parameter;
use std::sync::{Arc, Mutex, mpsc::Sender, mpsc::Receiver};
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
use std::error::Error;
use std::io::Result;
use std::io::ErrorKind;

pub type UiEvent= Option<i32>;

fn update_display(parameters:&Vec<Parameter>, selected:i32){
    println!("{}",terminal::Clear(terminal::ClearType::All));
    println!("{}", cursor::MoveTo(0, 0));
    for i in 0..parameters.len(){
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

//little enum that allow me to simplify the key_code match by deferring all the mutex work to a more convenient and centralized place
enum ParameterModified{
    Increment, 
    Decrement,
    SetValue(char),
}

pub fn interaction(parameters: Arc<Mutex<HashMap<String, Parameter>>>, param_sender: Sender<Parameter>, gui_sender: Sender<UiEvent>)->Result<Box<dyn std::error::Error>>{

    let mut selected = 0;
    let mut names:Vec<String> = Vec::new();

    //use this to get a name vector, il allow me to refer to a parameter via it's index rather than it's name
    for (name, parameter) in parameters.lock().unwrap().into_iter(){
        names.push(name);
    }
    let number_of_parameters = parameters.lock().unwrap().len();
    let mut parameters_modified = None; 
    enable_raw_mode().unwrap();
    let mut line = String::new();
    while let Event::Key(KeyEvent { code, .. }) = event::read().unwrap() {
        match code {
            KeyCode::Enter => {
                disable_raw_mode().unwrap();
                break;
            }
            KeyCode::Down => {selected += 1; selected=selected.clamp(0, number_of_parameters as i32-1);},
            KeyCode::Up => {selected -= 1; selected=selected.clamp(0, number_of_parameters as i32-1);},
            KeyCode::Left => {parameters_modified=Some(ParameterModified::Increment)}
            KeyCode::Right => {parameters_modified = Some(ParameterModified::Decrement)}
            KeyCode::Char(char) => {parameters_modified=Some(ParameterModified::SetValue(char))}
            // Key::Ctrl('q') => self.should_quit = true,
            _ => {},
        }
        
        //update GUI
        gui_sender.send(Some(selected)).map_err(|err| std::io::Error::new(ErrorKind::Other,"no gui receiver"))?;
        //update Audio Thread
        match parameters_modified {
            None => {}
            Some(modification)=>{
                let parameter_name = names[selected as usize];
                //the first unwrap is in the case where a mutex fucks up, the second is for get_mut and only return None if there is no parameter in the Hash map, which cannot happen
                let parameter = parameters.lock().unwrap().get_mut(&parameter_name).unwrap();
                //apply modification
                match modification{
                    Increment=>{parameter.increment()}
                    Decrement=>{parameter.decrement()}
                    ParameterModified::SetValue(char)=>{parameter.set_value(char)}
                }
                //get a copy of the parameter and send it to the audio thread
                param_sender.send(parameter.clone()).unwrap();
            }
            
            
        }
    }
        Ok(())
}

pub fn ui(parameters: Arc<Mutex<HashMap<String, Parameter>>>, receive_event: Receiver<UiEvent> ) -> Result<()>{
    let mut local_parameters:Vec<Parameter> = Vec::new();
    let mut selected = 0;
    //use this to get a name vector, il allow me to refer to a parameter via it's index rather than it's name
    loop{
        local_parameters.clear();
        for (_, parameter) in parameters.lock().unwrap().iter(){
            local_parameters.push(parameter.clone());
        }
        //TODO; find a way to use the hashmap in the display ?
        update_display(&local_parameters, selected);
        let event = receive_event.recv().map_err(|err| std::io::Error::new(ErrorKind::Other,"no gui sender"))?;

        //check if the event contain an new index value, if yes, apply
        selected = match event{ 
            Some(new_index) => {new_index}
            None => {selected}
        }
    }
}
