use crate::domain::browser::BrowserFingerprint;

pub fn launch_args(fingerprint: &Option<BrowserFingerprint>) -> Vec<String> {
    let Some(fingerprint) = fingerprint else {
        return Vec::new();
    };

    let mut args = Vec::new();
    push_string(&mut args, "fingerprint-brand", &fingerprint.browser_brand);
    push_string(&mut args, "fingerprint-platform", &fingerprint.os.to_lowercase());
    push_string(&mut args, "fingerprint-country", &fingerprint.country);
    push_string(&mut args, "fingerprint-language", &fingerprint.language);
    push_string(&mut args, "fingerprint-user-agent", &fingerprint.user_agent);
    push_string(
        &mut args,
        "fingerprint-accept-language",
        &fingerprint.accept_language,
    );
    push_string(&mut args, "fingerprint-js-platform", &fingerprint.platform);
    push_string(&mut args, "fingerprint-js-vendor", &fingerprint.vendor);
    push_string(&mut args, "fingerprint-timezone", &fingerprint.timezone);
    push_resolution(&mut args, fingerprint.resolution_width, fingerprint.resolution_height);
    push_screen_available(&mut args, fingerprint.avail_width, fingerprint.avail_height);
    push_window_size(&mut args, fingerprint.window_width, fingerprint.window_height);
    push_window_position(&mut args, fingerprint.window_x, fingerprint.window_y);
    push_float(
        &mut args,
        "fingerprint-device-scale-factor",
        fingerprint.device_scale_factor,
    );
    push_number(&mut args, "fingerprint-color-depth", fingerprint.color_depth);
    push_number(
        &mut args,
        "fingerprint-hardware-concurrency",
        fingerprint.cpu_cores,
    );
    push_number(&mut args, "fingerprint-gpu-memory", fingerprint.gpu_memory);
    push_string(&mut args, "fingerprint-disk-type", &fingerprint.disk_type);
    push_string(&mut args, "fingerprint-keyboard-layout", &fingerprint.keyboard_layout);
    push_string(&mut args, "fingerprint-webgl-renderer", &fingerprint.webgl_renderer);
    push_string(&mut args, "fingerprint-webgl-vendor", &fingerprint.webgl_vendor);
    push_string(&mut args, "fingerprint-canvas-policy", &fingerprint.canvas_policy);
    push_string(&mut args, "fingerprint-webgl-policy", &fingerprint.webgl_policy);
    push_string(&mut args, "fingerprint-webrtc-policy", &fingerprint.web_rtc_policy);
    push_string(&mut args, "fingerprint-geo-policy", &fingerprint.geo_policy);
    push_geo(
        &mut args,
        &fingerprint.geo_policy,
        fingerprint.geo_latitude,
        fingerprint.geo_longitude,
        fingerprint.geo_accuracy,
    );
    push_string(&mut args, "fingerprint-font-policy", &fingerprint.font_policy);
    push_string_list(&mut args, "fingerprint-font-list", &fingerprint.font_list);
    push_number(
        &mut args,
        "fingerprint-microphone-count",
        fingerprint.microphone_count,
    );
    push_number(&mut args, "fingerprint-speaker-count", fingerprint.speaker_count);
    push_number(&mut args, "fingerprint-camera-count", fingerprint.camera_count);
    push_bool(&mut args, "fingerprint-audio-codec", fingerprint.audio_codec);
    push_bool(&mut args, "fingerprint-camera", fingerprint.camera);
    push_bool(&mut args, "fingerprint-do-not-track", fingerprint.do_not_track);
    push_bool(&mut args, "fingerprint-ua-random", fingerprint.ua_random);
    push_bool(&mut args, "fingerprint-ldp-enabled", fingerprint.ldp_enabled);
    if fingerprint.start_maximized {
        args.push("--start-maximized".to_string());
    }
    push_string(&mut args, "user-agent", &fingerprint.user_agent);
    push_string(&mut args, "accept-lang", &fingerprint.accept_language);
    push_string(&mut args, "lang", &fingerprint.language);
    args
}

fn push_string(args: &mut Vec<String>, key: &str, value: &str) {
    let value = value.trim();
    if !value.is_empty() {
        args.push(format!("--{key}={value}"));
    }
}

fn push_number<T: std::fmt::Display>(args: &mut Vec<String>, key: &str, value: T) {
    args.push(format!("--{key}={value}"));
}

fn push_float(args: &mut Vec<String>, key: &str, value: f64) {
    if value != 0.0 {
        args.push(format!("--{key}={value}"));
    }
}

fn push_string_list(args: &mut Vec<String>, key: &str, values: &[String]) {
    let value = values
        .iter()
        .map(|item| item.trim())
        .filter(|item| !item.is_empty())
        .collect::<Vec<_>>()
        .join(",");
    if !value.is_empty() {
        args.push(format!("--{key}={value}"));
    }
}

fn push_bool(args: &mut Vec<String>, key: &str, value: bool) {
    args.push(format!("--{key}={}", if value { "true" } else { "false" }));
}

fn push_geo(args: &mut Vec<String>, policy: &str, latitude: f64, longitude: f64, accuracy: i32) {
    if policy.trim() != "custom" {
        return;
    }
    args.push(format!("--fingerprint-geo-latitude={latitude}"));
    args.push(format!("--fingerprint-geo-longitude={longitude}"));
    if accuracy > 0 {
        args.push(format!("--fingerprint-geo-accuracy={accuracy}"));
    }
}

fn push_resolution(args: &mut Vec<String>, width: i32, height: i32) {
    if width <= 0 || height <= 0 {
        return;
    }
    args.push(format!("--fingerprint-screen-width={width}"));
    args.push(format!("--fingerprint-screen-height={height}"));
}

fn push_screen_available(args: &mut Vec<String>, width: i32, height: i32) {
    if width <= 0 || height <= 0 {
        return;
    }
    args.push(format!("--fingerprint-avail-width={width}"));
    args.push(format!("--fingerprint-avail-height={height}"));
}

fn push_window_size(args: &mut Vec<String>, width: i32, height: i32) {
    if width <= 0 || height <= 0 {
        return;
    }
    args.push(format!("--fingerprint-window-width={width}"));
    args.push(format!("--fingerprint-window-height={height}"));
    args.push(format!("--window-size={width},{height}"));
}

fn push_window_position(args: &mut Vec<String>, x: i32, y: i32) {
    if x < 0 || y < 0 {
        return;
    }
    args.push(format!("--fingerprint-window-x={x}"));
    args.push(format!("--fingerprint-window-y={y}"));
    args.push(format!("--window-position={x},{y}"));
}
