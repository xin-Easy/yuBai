use crate::domain::config::Config;
use std::env;

pub fn prefer_mirror(config: Option<&Config>) -> bool {
    let source = config
        .map(|config| config.browser.download_source.trim().to_lowercase())
        .unwrap_or_else(|| "auto".to_string());

    match source.as_str() {
        "official" => false,
        "npmmirror" | "mirror" => true,
        _ => is_china_mainland(),
    }
}

fn is_china_mainland() -> bool {
    let tz_name = if let Ok(tz) = env::var("TZ") {
        tz
    } else if let Ok(link) = std::fs::read_link("/etc/localtime") {
        link.to_string_lossy().to_string()
    } else {
        String::new()
    };

    let china_tz_prefixes = [
        "Asia/Shanghai",
        "Asia/Chongqing",
        "Asia/Harbin",
        "Asia/Urumqi",
        "PRC",
        "ROC",
    ];

    china_tz_prefixes
        .iter()
        .any(|prefix| tz_name.contains(prefix))
}
