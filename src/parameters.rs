use crate::buffer;
use crate::envelope;
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
    pub id: ParameterID,
    pub parameter: Parameter,
}

impl ParameterCapsule {
    pub fn new(
        id: ParameterID,
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

//replace with variant_count if it someday hit stable release
pub const NUMBER_OF_PARAMETERS: usize = 12;

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum ParameterID {
    OscHarmonicRatio,
    OscHarmonicGain,
    EnvelopeAttack,
    EnvelopeRelease,
    FilterCutoff,
    DelayTime,
    DelayFeedback,
    DelayDryWet,
    ReverbTime,
    ReverbDryWet,
    Volume,
    FftTrehsold,
}
pub struct Parameters {
    pub parameters: [ParameterCapsule; NUMBER_OF_PARAMETERS],
}

impl Parameters {
    pub fn new() -> Self {
        type ID = ParameterID;
        type P = ParameterCapsule;

        let parameters = Self {
            parameters: [
                P::new(ID::OscHarmonicRatio, "osc-hrmrat", 32, 'h', 0.2, 2., 1.4),
                P::new(ID::OscHarmonicGain, "osc-hrmgn", 32, 'g', 0.01, 3., 1.),
                //envelope
                P::new(
                    ID::EnvelopeAttack,
                    "env-atk",
                    3,
                    'a',
                    envelope::MINIMUM_ENVELOPE_TIME,
                    envelope::MAXIMUM_ENVELOPE_TIME,
                    2.,
                ),
                P::new(
                    ID::EnvelopeRelease,
                    "env-dcy",
                    3,
                    'd',
                    envelope::MINIMUM_ENVELOPE_TIME,
                    envelope::MAXIMUM_ENVELOPE_TIME,
                    2.,
                ),
                P::new(ID::FilterCutoff, "cutoff", 36, 'c', 20., 20000., 4.),
                //Delay
                P::new(
                    ID::DelayTime,
                    "dly-time",
                    4,
                    't',
                    buffer::MINIMUM_DELAY_TIME,
                    buffer::MAXIMUM_DELAY_TIME,
                    2.,
                ),
                P::new(ID::DelayFeedback, "dly-feed", 4, 'f', 0., 1.0, 1.),
                P::new(ID::DelayDryWet, "dly-wet", 0, 'w', 0., 1., 1.),
                P::new(ID::ReverbDryWet, "rvb-wet", 0, 'r', 0., 1., 1.),
                P::new(ID::FftTrehsold, "fft-thr", 0, '8', 0., 1., 1.),
                P::new(ID::ReverbTime, "rvb-time", 0, '9', 0., 0.99, 1.),
                //global
                P::new(ID::Volume, "volume", 14, 'v', 0., 2., 2.),
            ],
        };

        assert!(parameters.no_id_double());
        assert!(parameters.no_cc_double());
        parameters
    }

    fn no_id_double(&self) -> bool {
        let mut vector: Vec<ParameterID> = Vec::new();
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

    fn no_cc_double(&self) -> bool {
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
    pub fn set(&mut self, id: ParameterID, mut value: i32) {
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
impl Index<ParameterID> for Parameters {
    type Output = Parameter;
    fn index(&self, parameter_id: ParameterID) -> &Self::Output {
        for parameter_capsule in self.parameters.iter() {
            if parameter_id == parameter_capsule.id {
                return &parameter_capsule.parameter;
            }
        }
        panic!("No parameter named {:?}", parameter_id)
    }
}

impl IndexMut<ParameterID> for Parameters {
    fn index_mut(&mut self, parameter_id: ParameterID) -> &mut Self::Output {
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
