/*
  ==============================================================================

    Outils.h
    Created: 11 Mar 2023 5:41:09pm
    Author:  thoma

  ==============================================================================
*/

/// @brief Slow value change of a parameter, slew factor working best between
/// 0.8 - 0.99
/// @param new_value
/// @param old_value
/// @param slew_factor a bigger slew factor means a slower change, must be <1 to
/// keep stability
/// @return

pub fn slew_value(new_value: f32, old_value: f32, slew_factor: f32) -> f32 {
    return (new_value * (1.0 - slew_factor)) + (old_value * (slew_factor));
}

/// @brief convert milliseconds to samples
/// @param time in seconds
/// @param sample_rate sample / secondes in Hz
/// @return
pub fn convert_ms_to_sample(time: f32, sample_rate: f32) -> f32 {
    return (sample_rate / 1000.) * time;
}

pub fn map_value_float_to_int(
    in_min: f32,
    in_max: f32,
    value: f32,
    out_min: i32,
    out_max: i32,
) -> i32 {
    let ratio = (out_max - out_min) as f32 / (in_max - in_min);
    let offset = out_min as f32 - (in_min * ratio);
    let output = (value * ratio + offset) as i32;
    return output;
}

pub fn map_value(value: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    let ratio = (out_max - out_min) / (in_max - in_min);
    let offset = out_min - (in_min * ratio);
    let output = value * ratio + offset;
    return output;
}

pub fn linear_crossfade(dry: f32, wet: f32, parameter: f32) -> f32 {
    return (dry * (1.0 - parameter)) + (wet * parameter);
}

pub fn equal_power_crossfade(dry: f32, wet: f32, mut parameter: f32) -> f32 {
    parameter = 1. - parameter;
    parameter = (parameter - 0.5) * 2.;
    let volumes_dry = (0.5 * (1. + parameter)).sqrt();
    let volumes_wet = (0.5 * (1. - parameter)).sqrt();
    return (dry * volumes_dry) + (wet * volumes_wet);
}
