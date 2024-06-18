pub enum FilterType {
    BPF,
    HPF,
    LPF,
    PEAK,
}

pub struct Biquad {
    //Parameters
    frequency_cutoff: f32,
    filter_type: FilterType,
    resonance: f32,
    peak_gain: f32,
    modulation: f32,

    //Computation variable
    sample_rate: f32,
    //Buffer memory for feedback and feedforward samples
    b: [f32; 3],
    a: [f32; 3],
    b_gain: [f32; 3],
    a_gain: [f32; 3],
    omega: f32,
    cosomega: f32,
    sinomega: f32,
    alpha: f32,
}

impl Biquad {
    pub fn new(
        sample_rate: f32,
        filter_type: FilterType,
    ) -> Self {
        let mut biquad = Biquad {
            filter_type,
            frequency_cutoff: 440.,
            resonance: 0.7,
            sample_rate,
            peak_gain: 0.,
            b: [0., 0., 0.],
            a: [0., 0., 0.],
            b_gain: [0., 0., 0.],
            a_gain: [0., 0., 0.],
            omega: 0.,
            cosomega: 0.,
            sinomega: 0.,
            alpha: 0.,
            modulation: 0.,
        };
        biquad.compute_coef();
        biquad
    }

    fn compute_lfp_coef(&mut self) {
        self.a_gain[0] = 1. + self.alpha;

        self.b_gain[0] = (1. - self.cosomega) / 2.;
        self.b_gain[0] /= self.a_gain[0];

        self.b_gain[1] = 1. - self.cosomega;
        self.b_gain[1] /= self.a_gain[0];

        self.b_gain[2] = self.b_gain[0];
        self.b_gain[2] /= self.a_gain[0];

        self.a_gain[1] = -2. * self.cosomega;
        self.a_gain[1] /= self.a_gain[0];

        self.a_gain[2] = 1. - self.alpha;
        self.a_gain[2] /= self.a_gain[0];
    }
    fn compute_hpf_coef(&mut self) {
        self.a_gain[0] = 1. + self.alpha;

        self.b_gain[0] = (1. + self.cosomega) / 2.;
        self.b_gain[0] /= self.a_gain[0];

        self.b_gain[1] = -(1. + self.cosomega);
        self.b_gain[1] /= self.a_gain[0];

        self.b_gain[2] = self.b_gain[0];
        self.b_gain[2] /= self.a_gain[0];

        self.a_gain[1] = -2. * self.cosomega;
        self.a_gain[1] /= self.a_gain[0];

        self.a_gain[2] = 1. - self.alpha;
        self.a_gain[2] /= self.a_gain[0];
    }
    fn compute_bpf_coef(&mut self) {
        self.a_gain[0] = 1. + self.alpha;

        self.b_gain[0] = self.alpha * self.resonance;
        self.b_gain[0] /= self.a_gain[0];

        self.b_gain[1] = 0.;
        self.b_gain[1] /= self.a_gain[0];

        self.b_gain[2] = -self.resonance * self.alpha;
        self.b_gain[2] /= self.a_gain[0];

        self.a_gain[1] = -2. * self.cosomega;
        self.a_gain[1] /= self.a_gain[0];

        self.a_gain[2] = 1. - self.alpha;
        self.a_gain[2] /= self.a_gain[0];
    }
    fn compute_peak_coef(&mut self) {
        let v0 = 10.0_f32.powf(self.peak_gain / 20.);
        let k = (std::f32::consts::PI * self.frequency_cutoff / self.sample_rate).tan();
        let k2 = k * k;
        let divide = 1. + (1. / self.resonance) * k + k2;

        self.b_gain[0] = 1. + (v0 / self.resonance) * k + k2;
        self.b_gain[0] /= divide;
        self.b_gain[1] = 2. * (k2 - 1.);
        self.b_gain[1] /= divide;
        self.a_gain[1] = self.b_gain[1];
        self.b_gain[2] = 1. - (v0 / self.resonance) * k + k2;
        self.b_gain[2] /= divide;
        self.a_gain[2] = 1. - (1. / self.resonance) * k + k2;
        self.a_gain[2] /= divide;
    }

    fn compute_coef(&mut self) {
        self.omega = 2.
            * std::f32::consts::PI
            * ((self.frequency_cutoff + self.modulation).clamp(20., 20000.) / self.sample_rate);
        self.cosomega = self.omega.cos();
        self.sinomega = self.omega.sin();
        self.alpha = self.sinomega / (2. * self.resonance);
        match self.filter_type {
            FilterType::PEAK => self.compute_peak_coef(),
            FilterType::LPF => self.compute_lfp_coef(),
            FilterType::HPF => self.compute_hpf_coef(),
            FilterType::BPF => self.compute_bpf_coef(),
        }
    }

    pub fn set_type(&mut self, filter_type: FilterType) {
        self.filter_type = filter_type;
    }

    fn w(self) -> FilterType {
        self.filter_type
    }

    pub fn set_parameters(&mut self, frequency_cutoff: f32, resonance: f32, gain: f32) {
        if self.frequency_cutoff == frequency_cutoff
            && self.resonance == resonance
            && self.peak_gain == gain
        {
            return;
        }
        self.frequency_cutoff = frequency_cutoff;
        self.resonance = resonance;
        self.peak_gain = gain;
        self.compute_coef();
    }

    pub fn set_frequence_and_resonance(&mut self, frequence: f32, resonance: f32) {
        if self.frequency_cutoff == frequence && self.resonance == resonance {
            return;
        }
        self.frequency_cutoff = frequence;
        self.resonance = resonance;
        self.compute_coef();
    }

    pub fn modulate(&mut self, modulation: f32) {
        self.modulation = modulation;
    }

    pub fn set_frequency(&mut self, frequence: f32) {
        if self.frequency_cutoff == frequence {
            return;
        }
        self.frequency_cutoff = frequence;
        self.compute_coef();
    }

    pub fn set_sample_rate(&mut self, _sample_rate: f32) {
        self.sample_rate = _sample_rate;
        self.compute_coef();
    }

    pub fn process(&mut self, mut b0: f32) -> f32 {
        self.compute_coef();
        //feedback & clipping
        let mut feedback = self.a[0];
        //1500 chosed by experimentation w/ sinensis, self osc around Q = 38
        feedback *= self.resonance / 1500.;
        if feedback < -4.5 || feedback > 4.5 {
            feedback /= 10.;
        }
        feedback = feedback.clamp(-5., 5.);
        b0 += feedback;
        //shift new value in
        self.b[2] = self.b[1].clamp(-100., 100.);
        self.b[1] = self.b[0].clamp(-100., 100.);
        self.b[0] = b0;
        self.a[2] = self.a[1].clamp(-100., 100.);
        self.a[1] = self.a[0].clamp(-100., 100.);

        self.a[0] =
            self.b[0] * self.b_gain[0] + self.b[1] * self.b_gain[1] + self.b[2] * self.b_gain[2]
                - self.a[1] * self.a_gain[1]
                - self.a[2] * self.a_gain[2];

        return self.a[0];
    }
}
