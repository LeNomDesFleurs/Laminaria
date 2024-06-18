use crate::{buffer::DelayLine, outils};


const NUMBER_OF_ALLPASS:usize = 6;
pub struct Reverb{
    allpasses: Vec<DelayLine>,
    pub dry_wet: f32,
}

impl Reverb{

    pub fn new(sample_rate: f32)->Self{
        let mut allpasses: Vec<DelayLine> = vec![];
        let time = [0.010, 0.020, 0.011, 0.050, 0.033, 0.027];
        for i in 0..NUMBER_OF_ALLPASS{
            allpasses.push(DelayLine::new_allpass(sample_rate, 0.100));
            allpasses[i].set_delay_time(time[i]);
        }
        Reverb{
        allpasses: allpasses,
        dry_wet: 0.5,
        }
    }

    pub fn set_reverb_time(&mut self, rt60:f32){
        for allpass in self.allpasses.iter_mut(){
            allpass.set_rt60(rt60);
        }
    }

    pub fn process(&mut self, input_sample:f32)->f32{
        let mut output_sample = input_sample;
        for allpass in self.allpasses.iter_mut(){
            output_sample = allpass.process(output_sample);
            output_sample *= 0.5;
        }
        return outils::equal_power_crossfade(input_sample, output_sample, self.dry_wet);
    }
}
