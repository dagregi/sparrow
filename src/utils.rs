use transmission_rpc::types::TorrentStatus;

pub fn convert_bytes(bytes: i64) -> String {
    ["B", "KB", "MB", "GB", "TB"]
        .iter()
        .enumerate()
        .find_map(|(i, &unit)| {
            if bytes < 1024 << (i * 10) {
                Some(format!(
                    "{:.1} {}",
                    bytes as f64 / 1024.0_f64.powi(i as i32),
                    unit
                ))
            } else {
                None
            }
        })
        .unwrap_or(format!("{bytes} B"))
}

pub fn handle_ratio(ratio: f32) -> String {
    if ratio == -1_f32 {
        "None".to_string()
    } else {
        format!("{ratio:.2}")
    }
}

pub fn convert_status(status: TorrentStatus) -> String {
    match status {
        TorrentStatus::Stopped => "Stopped".to_string(),
        TorrentStatus::QueuedToVerify => "QueuedToVerify".to_string(),
        TorrentStatus::Verifying => "Verifying".to_string(),
        TorrentStatus::QueuedToDownload => "QueuedToDownload".to_string(),
        TorrentStatus::Downloading => "Downloading".to_string(),
        TorrentStatus::QueuedToSeed => "QueuedToSeed".to_string(),
        TorrentStatus::Seeding => "Seeding".to_string(),
    }
}

pub fn convert_eta(eta: i64) -> String {
    if eta == -1 {
        "Unknown".to_string()
    } else {
        let seconds = eta as u64;
        [
            (seconds / 86400, "d"),
            ((seconds % 86400) / 3600, "h"),
            ((seconds % 3600) / 60, "m"),
            (seconds % 60, "s"),
        ]
        .into_iter()
        .filter_map(|(value, unit)| {
            if value > 0 {
                Some(format!("{value}{unit}"))
            } else {
                None
            }
        })
        .collect::<String>()
    }
}

pub fn convert_percentage(done: f32) -> String {
    if done > 1.0 {
        "Done".to_string()
    } else {
        format!("{:.1}%", 100.0 * done)
    }
}
