use crate::{midi::MidiMessage, parameters::Parameters, ParameterUpdate};

pub trait HasParameters{
    fn get_parameters(&self) -> Parameters;
    fn set_parameter(&mut self, (id, new_value): ParameterUpdate){}
}

pub trait HasEngine{
    fn process(&mut self) -> f32;
}

pub trait HasMidiInput{
    fn set_note(&mut self, message: MidiMessage);
}


pub trait HasConstructor{
    fn new() -> Self where Self:Sized;
    fn init(&mut self, sample_rate:f32);
}

pub trait Synth: HasParameters + HasEngine + HasMidiInput + HasConstructor + Send {

}