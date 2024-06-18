use crate::outils::{self, rt60_to_gain};
#[derive(Clone, Copy)]
enum InterpolationMode {
    None,
    Linear,
    Allpass,
}

// #[derive(Copy, Clone)]
pub struct RingBuffer {
    interpolation_mode: InterpolationMode,
    pub freezed: bool,
    reverse: bool,
    sample_rate: f32,
    buffer: Vec<f32>,
    read: f32,
    write: f32,
    i_read: i32,
    i_read_next: i32,
    step_size: f32,
    size_goal: i32,
    buffer_size: i32,
    actual_size: f32,
    size_on_freeze: f32,
    frac: f32,
    output_sample: f32,
    // self.buffer_size en base 0
}

impl RingBuffer {
    ///Buffer size in seconds
    pub fn new(sample_rate: f32, max_time: f32) -> Self {
        let buffer_size: usize = (sample_rate * max_time) as usize;

        RingBuffer {
            interpolation_mode: InterpolationMode::Linear,
            freezed: false,
            reverse: false,
            sample_rate,
            buffer: vec![0.; buffer_size],
            buffer_size: (buffer_size - 1) as i32,
            write: (buffer_size / 2) as f32,
            actual_size: (buffer_size / 2) as f32,
            size_goal: (buffer_size / 2) as i32,
            read: 0.,
            i_read_next: 1,
            i_read: 0,
            step_size: 1.,
            size_on_freeze: 0.,
            frac: 0.,
            output_sample: 0.,
        }
    }

    /// @brief increment pointer and set its int, incremented int and frac value
    fn increment_read_pointer(&mut self) {
        self.read += self.step_size;
        self.check_for_read_index_overflow();
        if self.read > self.buffer_size as f32 {
            self.read -= self.buffer_size as f32
        }
        // in case of reverse read
        else if self.read < 0. {
            self.read += self.buffer_size as f32
        }
    }

    /// increment read pointer and return sample from interpolation
    pub fn read_sample(&mut self) -> f32 {
        if self.reverse {
            self.step_size = 0. - self.step_size;
        }

        if self.freezed {
            self.freeze_increment_read_pointer();
            self.freezed_update_step_size();
        } else {
            self.update_step_size();
            self.increment_read_pointer();
        }

        self.fractionalize_read_index();

        // those functions modify the self.output_sample value
        match self.interpolation_mode {
            InterpolationMode::None => self.no_interpolation(),
            InterpolationMode::Linear => self.linear_interpolation(),
            InterpolationMode::Allpass => self.allpass_interpolation(),
        }

        if self.freezed && self.step_size < 1.0{
            self.output_sample /= self.step_size.powf(1.5);
        }

        return self.output_sample;
    }

    fn no_interpolation(&mut self) {
        self.output_sample = self.buffer[self.i_read as usize];
    }

    /// Interpolation lineaire du buffer a un index flottant donne
    fn linear_interpolation(&mut self) {
        // S[n]=frac * Buf[i+1]+(1-frac)*Buf[i]
        self.output_sample = (self.frac * self.buffer[self.i_read_next as usize])
            + ((1. - self.frac) * self.buffer[self.i_read as usize]);
    }

    /// Interpolation passe-tout, recursion
    fn allpass_interpolation(&mut self) {
        // S[n]=Buf[i+1]+(1-frac)*Buf[i]-(1-frac)*S[n-1]
        self.output_sample = (self.buffer[(self.i_read + 1) as usize])
            + ((1. - self.frac) * self.buffer[(self.i_read) as usize])
            - ((1. - self.frac) * self.output_sample);
    }

    /// increment write pointer and write input sample in buffer
    /// input_sample
    pub fn write_sample(&mut self, input_sample: f32) {
        if !self.freezed {
            if self.write > (self.buffer_size - 1) as f32 {
                self.write = 0.;
            } else {
                self.write += 1.
            };
            self.buffer[self.write as usize] = input_sample;
            // self.buffer[0] = input_sample;
        }
    }

    pub fn set_step_size(&mut self, step_size: f32) {
        self.step_size = step_size;
    }

    /// Triggered at each sample, update the step size and the self.actual_size
    /// to keep up with change of size goal
    fn update_step_size(&mut self) {
        let correction_offset: f32 = 0.;
        if self.actual_size > (self.size_goal - 5) as f32
            && self.actual_size < (self.size_goal + 5) as f32
        {
            self.step_size = 1.0;
        } else if self.actual_size > self.size_goal as f32 {
            self.step_size = 1.5;
            self.actual_size -= 0.5;
            // update the step size but with slew for clean repitch
        } else if self.actual_size < self.size_goal as f32 {
            self.step_size = 0.5;
            self.actual_size += 0.5;
        }

        // self.step_size = noi::Outils::slewValue(correction_offset, self.step_size,
        // 0.999);

        // if (self.step_size > 0.999 && self.step_size < 1.0001) {
        //   self.step_size = 1.0;
        // }

        // if (!freezed){
        // if (self.step_size > 1) {
        //   self.actual_size -= self.step_size - 1;

        // } else if (self.step_size < 1) {
        //   self.actual_size += 1 - self.step_size;
        // }
        // update the step size and update the actual delay time
        // }
    }

    /// Take a delay time in milliseconds, clip it within the defined max
    /// buffer size and set the goal to reach.
    /// delay_time in milliseconds
    pub fn set_delay_time(&mut self, delay_time: f32) {
        let delay_in_samples: i32 =
            outils::convert_ms_to_sample(delay_time, self.sample_rate) as i32;
        //   adding some 4 samples padding just to be sure.
        self.size_goal = (delay_in_samples.clamp(4, self.buffer_size as i32 - 4)) as i32;
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }

    pub fn set_freezed(&mut self, freezed: bool) {
        // avoid updating the self.size_on_freeze
        if !self.freezed {
            self.size_on_freeze = self.actual_size;
        }
        self.freezed = freezed;
    }

    fn freezed_update_step_size(&mut self) {
        self.step_size = self.size_on_freeze / self.size_goal as f32;
    }

    fn check_for_read_index_overflow(&mut self) {
        if self.read < 0. {
            self.read += self.buffer_size as f32;
        }
        if self.read > self.buffer_size as f32 {
            self.read -= self.buffer_size as f32;
        }
    }

    fn fractionalize_read_index(&mut self) {
        // get sample
        self.i_read = self.read.floor() as i32;
        // get fraction
        self.frac = self.read - (self.i_read as f32);
        // Get next sample
        self.i_read_next = (self.i_read + 1) % (self.buffer_size - 1);
    }

    fn freeze_increment_read_pointer(&mut self) {
        self.read += self.step_size;
        // buffer over and under flow
        self.check_for_read_index_overflow();
        self.actual_size -= self.step_size;

        // In freezed case, self.read only iterate on the last buffer size,
        //  hence it's like a little ringBuffer in the bigger ringBuffer
        //  so more buffer over and under flow
        if self.actual_size < 0. {
            self.read -= self.write - self.size_on_freeze;
            self.check_for_read_index_overflow();
            self.actual_size = self.size_on_freeze;
        } else if self.actual_size > self.size_on_freeze {
            self.read = self.write;
            self.actual_size = 0.;
        }
    }
}

pub static MAXIMUM_DELAY_TIME: f32 = 10.;
pub static MINIMUM_DELAY_TIME: f32 = 0.01;

enum DelayMode {
    //delay line => dry wet
    DelayLine,
    //flat frequency feedback
    Allpass,
    //basic feedback
    Comb,
}
// #[derive(Clone, Copy)]
pub struct DelayLine {
    buffer: RingBuffer,
    feedback: f32,
    dry_wet: f32,
    delay_time: f32,
    delay_mode: DelayMode,
}

impl DelayLine {
    pub fn set_rt60(&mut self, rt60: f32) {
        self.feedback = rt60_to_gain(rt60, self.delay_time)
    }
    //max_time in seconds
    pub fn new(sample_rate: f32, max_time: f32) -> Self {
        DelayLine {
            buffer: RingBuffer::new(sample_rate, max_time),
            feedback: 0.5,
            dry_wet: 0.5,
            delay_mode: DelayMode::DelayLine,
            delay_time: max_time,
        }
    }
    //max_time in seconds
    pub fn new_comb(sample_rate: f32, max_time: f32)->Self{
        let mut delay = DelayLine::new(sample_rate, max_time);
        delay.delay_mode=DelayMode::Comb;
        delay
    }

    //max_time in seconds
    pub fn new_allpass(sample_rate: f32, max_time: f32)->Self{
        let mut delay = DelayLine::new(sample_rate, max_time);
        delay.delay_mode=DelayMode::Allpass;
        delay
    }

    pub fn process(&mut self, input_sample: f32) -> f32 {
        let mut delay = self.buffer.read_sample();
        delay = delay.clamp(-1.0, 1.0);
        let mut buf_in = 0.0;
        let mut buf_out = 0.0;
        // buffer.writeSample(buf_in);
        // return buf_out;
        match self.delay_mode{
            DelayMode::DelayLine=>{
                //buf_in = input_sample + delay * feedback
                //buf_out = drywet(input_sample, delay)
                buf_in = input_sample + delay * self.feedback;
                buf_out = outils::equal_power_crossfade(input_sample, delay, self.dry_wet);
            }
            DelayMode::Allpass=>{
                // float buf_in = (delay * m_gain) + input;
                // float buf_out = delay + (input * -m_gain);
                buf_in = input_sample + delay * self.feedback;
                buf_out = delay + (input_sample * -self.feedback);
            }
            DelayMode::Comb=>{
                //buf_in = input_sample + delay * feedback
                //buf_out = delay
                buf_in = input_sample + (delay * self.feedback);
            }
        }
        self.buffer.write_sample(buf_in);
        buf_out
    }
    ///time in seconds
    pub fn set_delay_time(&mut self, delay_time: f32) {
        self.delay_time = delay_time;
        self.buffer.set_delay_time(delay_time * 1000.);
    }

    ///O = dry, 1 = wet
    pub fn set_dry_wet(&mut self, dry_wet: f32) {
        self.dry_wet = dry_wet;
    }

    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback;
    }

    pub fn set_freeze(&mut self, freeze: bool) {
        self.buffer.set_freezed(freeze)
    }
}