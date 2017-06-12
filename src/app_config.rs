//! Standing - config parser
//!
//! Copyright 2017 Keith Wansbrough <keith@lochan.org>

use config::{Config, Value, File, FileFormat};
use std::result::Result;
use std::result::Result::{Ok,Err};

pub struct AppConfig {
    /// Number of segments in the string.
    pub num_components: usize,

    /// Cross-section polygon of the string, as (p,q).
    pub component_pqs: Vec<[f32; 2]>,

    /// P vector for cross-section.
    pub pv: [f32; 3],

    /// Q vector for cross-section.
    pub qv: [f32; 3],

    /// Background colour.
    pub clear_colour: [f32; 4],

    pub strings: Vec<StringConfig>,

    /// Light source location (actually this sets the direction only: from here toward origin).
    pub light_source_location: [f32; 3],

    /// Location of the (perspective) camera.
    pub camera_pos: [f32; 3],

    /// Field of view of camera.
    pub camera_fov_deg: f32,

    /// Camera Z near value.
    pub camera_near: f32,

    /// Camera Z far value.
    pub camera_far: f32,

    /// Temporal frequency.
    pub temporal_freq_hz: f64,

    /// Spatial frequency.
    pub spatial_freq_waves_per_unit: f64,

    /// Amplitude of waves (before model transformation).
    pub amplitude: f32,

    /// Reporting interval (for console reporting of FPS etc).
    pub report_interval_sec: f64,
}

pub struct StringConfig {
    /// String colour.
    pub string_colour: [f32; 4],

    /// String position.
    pub string_pos_1: [f32; 3],

    /// String scale (non-uniform).
    /// Initially x goes from -0.5 to 0.5, y from +/-AMPLITUDE, z small; plus PQ. This allows those
    /// to be adjusted.
    pub string_scale: [f32; 3],
}

fn get_vec2(value: Value) -> Result<[f32; 2], String> {
    let array = value.into_array().ok_or("must be an array")?;
    let vec: Vec<f32> = array.into_iter().map(|x| {
            x.into_float().ok_or("must be a float".to_string()).map(|x| x as f32)
        }).collect::<Result<Vec<f32>, String>>()?;
    if vec.len() != 2 { return Err("must have 2 components".to_string()) }
    Ok([vec[0], vec[1]])
}

fn get_vec3(value: Value) -> Result<[f32; 3], String> {
    let array = value.into_array().ok_or("must be an array")?;
    let vec: Vec<f32> = array.into_iter().map(|x| {
        x.into_float().ok_or("must be a float".to_string()).map(|x| x as f32)
    }).collect::<Result<Vec<f32>, String>>()?;
    if vec.len() != 3 { return Err("must have 3 components".to_string()) }
    Ok([vec[0], vec[1], vec[2]])
}

fn get_vec4(value: Value) -> Result<[f32; 4], String> {
    let array = value.into_array().ok_or("must be an array")?;
    let vec: Vec<f32> = array.into_iter().map(|x| {
        x.into_float().ok_or("must be a float".into()).map(|x| x as f32)
    }).collect::<Result<Vec<f32>, String>>()?;
    if vec.len() != 4 { return Err("must have 4 components".into()) }
    Ok([vec[0], vec[1], vec[2], vec[3]])
}

impl AppConfig {
    /// Read the current config.
    pub fn new() -> Result<AppConfig, String> {
        let mut c = Config::new();
        c.merge(File::new("config.yaml", FileFormat::Yaml).required(true)).expect("Missing config file");
        Ok(AppConfig {
            num_components: c.get("string.num_components").ok_or("")?.into_int().ok_or("")? as _,
            temporal_freq_hz: c.get("string.temporal_freq_hz").ok_or("")?.into_float().ok_or("")? as _,
            spatial_freq_waves_per_unit: c.get("string.spatial_freq_waves_per_unit").ok_or("")?.into_float().ok_or("")? as _,
            amplitude: c.get("string.amplitude").ok_or("")?.into_float().ok_or("")? as _,

            component_pqs: c.get("polygon.component_pqs").ok_or("")?.into_array().ok_or("")?.into_iter().map(|v| {
                get_vec2(v)
            }).collect::<Result<Vec<[f32; 2]>, String>>()?,
            pv: get_vec3(c.get("polygon.pv").ok_or("")?)?,
            qv: get_vec3(c.get("polygon.qv").ok_or("")?)?,

            clear_colour: get_vec4(c.get("scene.background.colour").ok_or("")?)?,

            strings: c.get("scene.strings").ok_or("")?.into_array().ok_or("")?.into_iter().map(|s| {
                StringConfig::new(s)
            }).collect::<Result<Vec<StringConfig>, String>>()?,

            light_source_location: get_vec3(c.get("scene.light.pos").ok_or("")?)?,

            camera_pos: get_vec3(c.get("scene.camera.pos").ok_or("")?)?,
            camera_fov_deg: c.get("scene.camera.fov_deg").ok_or("")?.into_float().ok_or("")? as _,
            camera_near: c.get("scene.camera.near").ok_or("")?.into_float().ok_or("")? as _,
            camera_far: c.get("scene.camera.far").ok_or("")?.into_float().ok_or("")? as _,

            report_interval_sec: c.get("console.report_interval_sec").ok_or("")?.into_int().ok_or("")? as _,
        })
    }
}

impl StringConfig {
    /// Read one string's config.
    pub fn new(value: Value) -> Result<StringConfig, String> {
        let hashmap = value.into_table().ok_or("")?;
        Ok(StringConfig {
            string_colour: get_vec4(hashmap.get("colour").ok_or("")?.clone())?,
            string_pos_1: get_vec3(hashmap.get("pos").ok_or("")?.clone())?,
            string_scale: get_vec3(hashmap.get("scale").ok_or("")?.clone())?,
        })
    }
}
