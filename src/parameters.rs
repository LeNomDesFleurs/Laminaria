use crate::outils;
use std::ops::{Index, IndexMut};

#[derive(Clone)]
pub struct Parameter {
    pub display_name: String,
    pub value: i32,
    pub midicc: char,
    pub min: f32,
    pub max: f32,
    pub skew: f32,
}

impl Parameter {
    ///return a empty parameter for test purposes
    pub fn new_dummy(name: &str) -> Self {
        Parameter {
            display_name: name.to_string(),
            value: 0,
            midicc: '0',
            min: 0.,
            max: 1.,
            skew: 1.,
        }
    }

    pub fn build_string(&self) -> String {
        let mut string: String = Default::default();
        //CC
        string += &self.midicc.to_string();
        string += " - ";
        //NAME
        string += &self.display_name;
        //MARGIN
        for _i in 0..10 - &self.display_name.len() {
            string += " ";
        }
        string += " - ";
        //value in orca numbers
        string += &outils::get_orca_character(self.value)
            .unwrap_or('0')
            .to_string();
        string += " - ";
        //value vizualisation
        string += &self.display_value();
        //raw value
        string += " ";
        string += &format!("{:.2}", self.get_raw_value()).to_string();
        return string;

        //get lenght après le nom pour les valeur arrive au même endroit (voir mêem centrer le non des variables ?)
    }

    pub fn get_raw_value(&self) -> f32 {
        let mut value = self.value as f32 / 35.;
        value = value.powf(self.skew);
        value *= self.max - self.min;
        value += self.min;
        return value;
    }

    pub fn increment(&mut self) {
        self.value += 1;
        self.value = self.value.clamp(0, 35);
    }

    pub fn decrement(&mut self) {
        self.value -= 1;
        self.value = self.value.clamp(0, 35);
    }

    fn display_value(&self) -> String {
        let mut string: String = Default::default();
        for i in 0..35 {
            if i < self.value {
                string += "|";
            }
            if i >= self.value {
                string += "-";
            }
        }
        return string;
    }
    pub fn set_value(&mut self, character: char) {
        self.value = outils::get_orca_integer(character).unwrap_or(self.value as u8) as i32;
    }
}

pub struct ParameterCapsule {
    pub id: i32,
    pub parameter: Parameter,
}

impl ParameterCapsule {
    pub fn new(
        id: i32,
        display_name: &str,
        default_value: i32,
        cc: char,
        min: f32,
        max: f32,
        skew: f32,
    ) -> Self {
        Self {
            id: id,
            parameter: Parameter {
                display_name: display_name.to_string(),
                value: default_value,
                midicc: cc,
                min: min,
                max: max,
                skew: skew,
            },
        }
    }
}



pub struct Parameters {
    pub parameters: Vec<ParameterCapsule>,
    pub nb_param: usize,
}

impl Parameters {
    // pub fn new() -> Self {
    //     type ID = ParameterID;
    //     type P = ParameterCapsule;

    //     assert!(parameters.no_id_double());
    //     assert!(parameters.no_cc_double());
    //     parameters
    // }

    pub fn no_id_double(&self) -> bool {
        let mut vector: Vec<i32> = Vec::new();
        for parameter_capsule in &self.parameters {
            for id in &vector {
                if parameter_capsule.id == *id {
                    return false;
                }
            }
            vector.push(parameter_capsule.id)
        }
        return true;
    }

    pub fn no_cc_double(&self) -> bool {
        let mut vector: Vec<char> = Vec::new();
        for parameter_capsule in &self.parameters {
            for cc in &vector {
                if parameter_capsule.parameter.midicc == *cc {
                    return false;
                }
            }
            vector.push(parameter_capsule.parameter.midicc)
        }
        return true;
    }

    //search the parameter array for the correct id and set the new parameter
    pub fn set(&mut self, id: i32, mut value: i32) {
        value = value.clamp(0, 35);
        for capsule in self.parameters.iter_mut() {
            if id == capsule.id {
                capsule.parameter.value = value;
                return;
            }
        }
        panic!("No parameter named {:?}", id)
    }
}
impl Index<i32> for Parameters {
    type Output = Parameter;
    fn index(&self, parameter_id: i32) -> &Self::Output {
        for parameter_capsule in self.parameters.iter() {
            if parameter_id == parameter_capsule.id {
                return &parameter_capsule.parameter;
            }
        }
        panic!("No parameter named {:?}", parameter_id)
    }
}

impl IndexMut<i32> for Parameters {
    fn index_mut(&mut self, parameter_id: i32) -> &mut Self::Output {
        for parameter_capsule in self.parameters.iter_mut() {
            if parameter_id == parameter_capsule.id {
                return &mut parameter_capsule.parameter;
            }
        }
        panic!("No parameter named {:?}", parameter_id)
    }
}

enum ParametersEnum {
    Filter,
    Volume,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basics() {
        // Check empty list behaves right
        assert_eq!(true, true);
    }

    #[test]
    fn peek() {}
}
