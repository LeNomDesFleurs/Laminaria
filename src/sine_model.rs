use crate::{midi::MidiMessage, parameters::Parameters, ParameterUpdate};


const NUMBER_OF_VOICES: usize = 4;
const VOICE_ITERATOR: std::ops::Range<usize> = 0..NUMBER_OF_VOICES;

const NB_SYNTH_PARAM: usize = 11;

#[derive(PartialEq, Debug, Copy, Clone, FromPrimitive)] //from primitive allow me to cast i32 as enum
pub enum SineModelParamID {
    EnvelopeAttack,
    EnvelopeRelease,
    Volume,
}

pub struct SineModel {
    envelopes: [Envelope; NUMBER_OF_VOICES],
    oscillators: [HarmonicOscillator; NUMBER_OF_VOICES],
    midibuffer: PolyMidiBuffer,
    //parameters
    volume: f32,
}

impl SineModel {
    pub fn new(sample_rate: f32) -> Self {
        SineModel {
            envelopes: [Envelope::new(sample_rate as i32); NUMBER_OF_VOICES],
            oscillators: [SineWave::new(sample_rate); NUMBER_OF_VOICES],
            midibuffer: PolyMidiBuffer::new(NUMBER_OF_VOICES),
            volume: 0.5,
            // parameters: Parameters {},
        }
    }
}


pub trait HasParameters{

     fn get_parameters() -> Parameters {
        type ID = SineModelParamID;
        type P = ParameterCapsule;

        let params = Parameters {
            parameters: vec![
                //envelope
                P::new(
                    ID::EnvelopeAttack as i32,
                    "env-atk",
                    3,
                    'a',
                    envelope::MINIMUM_ENVELOPE_TIME,
                    envelope::MAXIMUM_ENVELOPE_TIME,
                    2.,
                ),
                P::new(
                    ID::EnvelopeRelease as i32,
                    "env-dcy",
                    3,
                    'd',
                    envelope::MINIMUM_ENVELOPE_TIME,
                    envelope::MAXIMUM_ENVELOPE_TIME,
                    2.,
                ),
                //global
                P::new(ID::Volume as i32, "volume", 14, 'v', 0., 2., 2.),
            ],
            nb_param: NB_SYNTH_PARAM,
        };
        assert!(params.no_id_double());
        assert!(params.no_cc_double());
        params
    }

    fn set_parameter(&mut self, (id, new_value): ParameterUpdate) {
        //need to find the parameter description to know the min max

        type ID = SineModelParamID;
        let typed_id: SineModelParamID = num::FromPrimitive::from_i32(id).unwrap();
        match typed_id {
            ID::Volume => self.volume = new_value,
            // envelope
            ID::EnvelopeAttack => self
                .envelopes
                .iter_mut()
                .for_each(|env| env.set_attack(new_value)),
            ID::EnvelopeRelease => self
                .envelopes
                .iter_mut()
                .for_each(|env| env.set_release(new_value)),
            }
        }
    }
}

pub trait HasEngine{
    fn process(&mut self) -> f32 {
        let mut sample: f32 = 0.;
        for i in VOICE_ITERATOR {
            match self.envelopes[i].status {
                envelope::Segment::Off => {}
                _ => sample += self.oscillators[i].process() * self.envelopes[i].process(),
            }
        }
        sample /= 4.;

        //vca
        sample *= self.volume;

        return sample;
    }
}

pub trait HasMidiInput{
        fn set_note(&mut self, message: MidiMessage) {
        match message {
            MidiMessage::NoteOff(midi_note) => self.midibuffer.remove_note(midi_note),
            MidiMessage::NoteOn(midi_note) => self.midibuffer.add_note(midi_note),
            _ => {}
        }

        for i in VOICE_ITERATOR {
            match self.midibuffer.notes.get(i) {
                None => self.envelopes[i].note_off(),
                Some(midi_note) => {
                    self.envelopes[i].note_on();
                    self.oscillators[i].set_note(*midi_note)
                }
            }
        }
    }
}

// trait Synth: HasParameters + HasEngine + HasMidiInput;