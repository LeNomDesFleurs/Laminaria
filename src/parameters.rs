use std::collections::HashMap;
use crate::buffer;
use crate::envelope;
use crate::outils;


#[derive(Clone)]
pub struct Parameter{
        pub name: String,
        pub value: i32,
        pub midicc: char,
        pub min: f32,
        pub max: f32,
        pub skew: f32,
}

impl Parameter{
    pub fn build_string(&self)->String{
        let mut string:String = Default::default();
        //CC
        string += &self.midicc.to_string();
        string += " - ";
        //NAME
        string += &self.name;
        //MARGIN
        for _i in 0..10-&self.name.len(){
            string+=" ";
        }
        string += " - ";
        //value in orca numbers
        string += &outils::get_orca_character(self.value).unwrap_or('0').to_string();
        string += " - ";
        //value vizualisation
        string += &self.display_value();
        //raw value
        string += " ";
        string += &format!("{:.2}", self.get_raw_value()).to_string();
        return string;
    
    //get lenght après le nom pour les valeur arrive au même endroit (voir mêem centrer le non des variables ?)
    }
    
    pub fn get_raw_value(&self)->f32{
        let mut value = self.value as f32/36.;
        value = value.powf(self.skew);
        value *= self.max - self.min;
        value += self.min;
        return value;
    }

    pub fn increment(&mut self){
        self.value += 1;
        self.value = self.value.clamp(0, 35);
    }
    
    pub fn decrement(&mut self){
        self.value -= 1;
        self.value = self.value.clamp(0, 35);
    }
    
    fn display_value(&self)->String{
        let mut string:String = Default::default();
        for i in 0..35{
            if i < self.value{
                string+="|";
            }
            if i >= self.value {
                string+="-";
            }
        }
        return string;
    }
    pub fn set_value(&mut self, character:char){
        self.value = outils::get_orca_integer(character).unwrap_or(self.value as u8) as i32;
    }
}

// #[macro_export]
// macro_rules! make_param {
//     ( $name: literal, $value: literal, $midicc: literal, $min: literal, $max: literal, $skew: literal) => {
//         {
//     ($name.to_string(), Parameter{name: $name.to_string(), value:$value, midicc:$midicc, min: 20., max: 20000.,skew:0.5})
//         }
//     };
// }

pub fn get_parameters()->HashMap<String, Parameter>{
    return HashMap::from([
        //filter
    // make_param!("fil_freq", 32, 'f', 20., 20000., 0.5),
    ("fil-freq".to_string(), Parameter{name: "fil-freq".to_string(),value:32, midicc:'f', min: 20., max: 20000.,skew:0.5}),
        //osc
    ("osc-tune".to_string(), Parameter{name: "osc-tune".to_string(),value:32, midicc:'0', min: -100., max: 100.,skew:0.5}),
    ("osc-nbhrm".to_string(), Parameter{name: "osc-nbhrm".to_string(), value: 32, midicc:'h', min:0., max:2., skew:1.}),
    ("osc-hrmgn".to_string(), Parameter{name: "osc-hrmgn".to_string(), value: 32, midicc:'g', min:0.01, max:3., skew:1.}),
    ("lfo-freq".to_string(), Parameter{name: "lfo-freq".to_string(), value:32, midicc:'l', min:0., max:5., skew: 0.5}),
    ("lfo-period".to_string(), Parameter{name: "lfo-period".to_string(), value:32, midicc:'p', min:0., max:5., skew: 0.5}),
    //envelop
    ("env-atk".to_string(), Parameter{name: "env-atk".to_string(), value: 3, midicc:'a', min: envelope::MINIMUM_ENVELOPE_TIME,  max: envelope::MAXIMUM_ENVELOPE_TIME, skew: 2.}), 
    ("env-dcy".to_string(), Parameter{name: "env-dcy".to_string(), value: 3, midicc:'d', min: envelope::MINIMUM_ENVELOPE_TIME,  max: envelope::MAXIMUM_ENVELOPE_TIME, skew: 2.}), 
    //Delay
    ("dly-time".to_string(), Parameter{name: "dly-time".to_string(), value:4, midicc:'t', min: buffer::MINIMUM_DELAY_TIME, max: buffer::MAXIMUM_DELAY_TIME, skew: 2. }),
    ("dly-feed".to_string(), Parameter{name: "dly-feed".to_string(), value:4, midicc:'f', min: 0., max: 0.99, skew: 1. }),
    ("dly-wet".to_string(), Parameter{name: "dly-wet".to_string(), value:15, midicc:'w', min: 0., max:1., skew:1.}),
    //global
    ("volume".to_string(), Parameter{name: "volume".to_string(), value:32, midicc:'v', min:0., max:1., skew: 2.}),
    ])
}