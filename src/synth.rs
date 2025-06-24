use crate::{midi::MidiMessage, parameters::Parameters, ParameterUpdate};



pub trait HasParameters{
    fn get_parameters() -> Parameters;
    fn set_parameter(&mut self, (id, new_value): ParameterUpdate){}
}

pub trait HasEngine{
    fn process(&mut self) -> f32;
}

pub trait HasMidiInput{
    fn set_note(&mut self, message: MidiMessage);
}

// trait Synth: HasParameters + HasEngine + HasMidiInput;