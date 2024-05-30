


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