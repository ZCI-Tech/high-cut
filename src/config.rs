use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub silence_threshold_db: f32,
    pub min_silence_duration: f64,
    pub min_clip_length: f64,
    pub max_clip_length: f64,
    pub margin_s: f64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            silence_threshold_db: -35.0,
            min_silence_duration: 1.0,
            min_clip_length: 1.0,
            max_clip_length: 60.0,
            margin_s: 0.5,
        }
    }
}
