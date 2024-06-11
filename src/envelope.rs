use crate::envelope::Segment::{Attack, Off, Release, Sustain};
use crate::outils::convert_ms_to_sample;
//in milliseconds
pub static MAXIMUM_ENVELOPE_TIME: f32 = 10000.;
pub static MINIMUM_ENVELOPE_TIME: f32 = 10.;

#[derive(PartialEq, Copy, Clone)]
pub enum Segment {
    Attack,
    Sustain,
    Release,
    Off,
}
#[derive(Clone, Copy)]
pub struct Envelope {
    value: f32,
    pub status: Segment,
    //in sample
    sample_rate: i32,
    increment: f32,
    decrement: f32,
}


impl Envelope {
    pub fn new(sample_rate: i32) -> Self {
        Self {
            value: 0.,
            status: Off,
            increment: 0.001,
            decrement: 0.001,
            sample_rate: sample_rate,
        }
    }
    ///time in ms
    pub fn set_attack(&mut self, time: f32) {
        self.set_segment_length(time, Attack)
    }
    ///time in ms
    pub fn set_release(&mut self, time: f32) {
        self.set_segment_length(time, Release)
    }
    //generalize the process, specialize the interface
    fn set_segment_length(&mut self, time: f32, segment: Segment) {
        assert!(
            segment == Attack || segment == Release,
            "can only set time of attack or release"
        );
        let clamped_time = time.clamp(MINIMUM_ENVELOPE_TIME, MAXIMUM_ENVELOPE_TIME);
        let samples = convert_ms_to_sample(clamped_time, self.sample_rate as f32);
        let step = 1. / samples;
        match segment {
            Segment::Attack => self.increment = step,
            Segment::Release => self.decrement = step,
            _ => {}
        }
    }
    pub fn note_on(&mut self){
        self.status = Attack
    }

    pub fn note_off(&mut self){
        self.status = Release
    }

    pub fn process(&mut self) -> f32 {
        match self.status {
            Off => {}
            Attack => {
                self.value += self.increment;
                if self.value >= 1. {
                    self.status = Segment::Sustain;
                }
            }
            Sustain => {}
            Release => {
                self.value -= self.decrement;
                if self.value <= 0. {
                    self.status = Segment::Off
                }
            }
        }
        return self.value;
        // .powf(0.7);
    }
}
