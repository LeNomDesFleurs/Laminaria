
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

pub struct UiEvent{
    pub new_selection: bool,
    pub selected_index: i32,
} 

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

pub fn interaction(parameters: Arc<Mutex<HashMap<String, Parameter>>>, param_sender: Sender<Parameter>, gui_sender: Sender<UiEvent>){

    let mut selected = 0;
    let mut names:Vec<String> = Vec::new();

    //use this to get a name vector, il allow me to refer to a parameter via it's index rather than it's name
    for (name, parameter) in parameters.lock().unwrap().clone().into_iter(){
        names.push(name);
    }
    let number_of_parameters = parameters.lock().unwrap().len();
    let mut parameters_modified = false; 
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
            KeyCode::Left => {parameters.lock().unwrap().get_mut(&names[selected as usize]).unwrap().decrement(); parameters_modified = true}
            KeyCode::Right => {parameters.lock().unwrap().get_mut(&names[selected as usize]).unwrap().increment(); parameters_modified = true}
            KeyCode::Char(char) => {parameters.lock().unwrap().get_mut(&names[selected as usize]).unwrap().set_value(char); parameters_modified= true}
            // Key::Ctrl('q') => self.should_quit = true,
            _ => {},
        }
        //update GUI
        gui_sender.send(UiEvent{new_selection: true, selected_index: selected}).unwrap();
        //update Audio Thread
        if parameters_modified {
            let parameter_copy = &parameters.lock().unwrap()[&names[selected as usize]];
            param_sender.send(parameter_copy.clone()).unwrap()
        }
}}

pub fn ui(parameters: Arc<Mutex<HashMap<String, Parameter>>>, receive_event: Receiver<UiEvent> ){
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
        let event = receive_event.recv().unwrap();
        if event.new_selection{selected=event.selected_index}
    }
}
