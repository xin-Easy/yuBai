use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserFingerprint {
    pub browser_brand: String,
    pub os: String,
    pub country: String,
    pub language: String,
    #[serde(default = "default_empty_string")]
    pub user_agent: String,
    #[serde(default = "default_accept_language")]
    pub accept_language: String,
    #[serde(default = "default_platform")]
    pub platform: String,
    #[serde(default = "default_vendor")]
    pub vendor: String,
    pub timezone: String,
    pub resolution_width: i32,
    pub resolution_height: i32,
    #[serde(default = "default_avail_width")]
    pub avail_width: i32,
    #[serde(default = "default_avail_height")]
    pub avail_height: i32,
    #[serde(default = "default_window_width")]
    pub window_width: i32,
    #[serde(default = "default_window_height")]
    pub window_height: i32,
    #[serde(default = "default_window_x")]
    pub window_x: i32,
    #[serde(default = "default_window_y")]
    pub window_y: i32,
    #[serde(default = "default_device_scale_factor")]
    pub device_scale_factor: f64,
    pub color_depth: i32,
    pub cpu_cores: i32,
    pub gpu_memory: i32,
    pub disk_type: String,
    pub keyboard_layout: String,
    pub webgl_renderer: String,
    pub webgl_vendor: String,
    #[serde(default = "default_canvas_policy")]
    pub canvas_policy: String,
    #[serde(default = "default_webgl_policy")]
    pub webgl_policy: String,
    #[serde(default = "default_webrtc_policy")]
    pub web_rtc_policy: String,
    #[serde(default)]
    pub geo_latitude: f64,
    #[serde(default)]
    pub geo_longitude: f64,
    #[serde(default = "default_geo_accuracy")]
    pub geo_accuracy: i32,
    #[serde(default = "default_geo_policy")]
    pub geo_policy: String,
    #[serde(default = "default_font_policy")]
    pub font_policy: String,
    #[serde(default = "default_font_list")]
    pub font_list: Vec<String>,
    #[serde(default = "default_microphone_count")]
    pub microphone_count: i32,
    #[serde(default = "default_speaker_count")]
    pub speaker_count: i32,
    #[serde(default)]
    pub camera_count: i32,
    #[serde(default)]
    pub start_maximized: bool,
    pub audio_codec: bool,
    pub camera: bool,
    pub do_not_track: bool,
    pub ua_random: bool,
    pub ldp_enabled: bool,
}

fn default_empty_string() -> String { String::new() }
fn default_accept_language() -> String { "zh-CN,zh;q=0.9,en;q=0.8".to_string() }
fn default_platform() -> String { "Win32".to_string() }
fn default_vendor() -> String { "Google Inc.".to_string() }
fn default_avail_width() -> i32 { 1920 }
fn default_avail_height() -> i32 { 1040 }
fn default_window_width() -> i32 { 1280 }
fn default_window_height() -> i32 { 900 }
fn default_window_x() -> i32 { 80 }
fn default_window_y() -> i32 { 60 }
fn default_device_scale_factor() -> f64 { 1.0 }
fn default_canvas_policy() -> String { "real".to_string() }
fn default_webgl_policy() -> String { "custom".to_string() }
fn default_webrtc_policy() -> String { "proxy".to_string() }
fn default_geo_accuracy() -> i32 { 100 }
fn default_geo_policy() -> String { "ask".to_string() }
fn default_font_policy() -> String { "system".to_string() }
fn default_font_list() -> Vec<String> {
    vec![
        "Arial".to_string(),
        "Calibri".to_string(),
        "Times New Roman".to_string(),
        "Microsoft YaHei".to_string(),
        "SimSun".to_string(),
    ]
}
fn default_microphone_count() -> i32 { 1 }
fn default_speaker_count() -> i32 { 1 }
