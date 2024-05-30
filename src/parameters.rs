

pub struct Parameters{
        pub name: String,
        pub value: i32,
        pub midicc: i8,
        pub min: f32,
        pub max: f32,
        pub skew: f32,
}

impl Parameter{

    pub fn build_string(&self)->String{
        let mut string:String = Default::default();
        //string += &self.midicc to string;
        string += &self.name;
        for _i in 0..10-&self.name.len(){
            string+=" ";
        }
        string += " - ";
        string += &self.get_orca_letter();
        string += " - ";
        string += &self.display_value();
        return string;
    
    //get lenght après le nom pour les valeur arrive au même endroit (voir mêem centrer le non des variables ?)
    }
    
    pub fn increment(&mut self){
        self.value += 1;
        self.value = self.value.clamp(0, 35);
    }
    
    pub fn decrement(&mut self){
        self.value -= 1;
        self.value = self.value.clamp(0, 35);
    }
    
    fn get_orca_letter(&self)->String{
        let value = self.value.clamp(0, 35);
        let letter_array: [&str; 36] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z"];
        return letter_array[value as usize].to_string();
    }
    
    fn display_value(&self)->String{
        let mut string:String = Default::default();
        for i in 0..35{
            if i <=self.value{
                string+="|";
            }
            if i > self.value {
                string+="-";
            }
        }
        return string;
    }
    pub fn set_value(&mut self, char:char){
        let letter_array: [&str; 36] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z"];
        for i in 0..36{
            if char == letter_array[i].chars().next().expect("string is empty"){
                self.value = i as i32;
                break;
            }
        }
    }
}
    