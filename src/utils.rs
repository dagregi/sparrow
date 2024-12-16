use transmission_rpc::types::{Priority, TorrentStatus};

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

pub fn convert_priority(priority: &Priority) -> String {
    match priority {
        Priority::Low => "Low".to_string(),
        Priority::Normal => "Normal".to_string(),
        Priority::High => "High".to_string(),
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
    } else if eta == -2 || eta > 86400 * 99 {
        "Inf".to_string()
    } else {
        let mut readable = [
            (eta / 86400, "d"),
            ((eta % 86400) / 3600, "h"),
            ((eta % 3600) / 60, "m"),
            (eta % 60, "s"),
        ]
        .into_iter()
        .filter_map(|(value, unit)| {
            if value > 0 {
                Some(format!("{value}{unit}"))
            } else {
                None
            }
        })
        .collect::<String>();

        if readable.contains('d') {
            readable.truncate(readable.find('d').unwrap() + 1);
        }
        readable
    }
}

pub fn convert_percentage(done: f32) -> String {
    if done >= 1.0 {
        "Done".to_string()
    } else {
        format!("{:.1}%", 100.0 * done)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_bytes() {
        assert_eq!(convert_bytes(0), "0.0 B");
        assert_eq!(convert_bytes(1), "1.0 B");
        assert_eq!(convert_bytes(1023), "1023.0 B");
        assert_eq!(convert_bytes(1024), "1.0 KB");
        assert_eq!(convert_bytes(1024 * 1024), "1.0 MB");
        assert_eq!(convert_bytes(1024 * 1024 * 1024), "1.0 GB");
        assert_eq!(convert_bytes(1024 * 1024 * 1024 * 1024), "1.0 TB");
        assert_eq!(convert_bytes(-1), "-1.0 B");
    }

    #[test]
    fn test_handle_ratio() {
        assert_eq!(handle_ratio(-1.0), "None");
        assert_eq!(handle_ratio(0.0), "0.00");
        assert_eq!(handle_ratio(0.5), "0.50");
        assert_eq!(handle_ratio(1.0), "1.00");
    }

    #[test]
    fn test_convert_priority() {
        assert_eq!(convert_priority(&Priority::Low), "Low");
        assert_eq!(convert_priority(&Priority::High), "High");
        assert_eq!(convert_priority(&Priority::Normal), "Normal");
    }

    #[test]
    fn test_convert_status() {
        assert_eq!(convert_status(TorrentStatus::Stopped), "Stopped");
        assert_eq!(
            convert_status(TorrentStatus::QueuedToVerify),
            "QueuedToVerify"
        );
        assert_eq!(convert_status(TorrentStatus::Verifying), "Verifying");
        assert_eq!(
            convert_status(TorrentStatus::QueuedToDownload),
            "QueuedToDownload"
        );
        assert_eq!(convert_status(TorrentStatus::Downloading), "Downloading");
        assert_eq!(convert_status(TorrentStatus::QueuedToSeed), "QueuedToSeed");
        assert_eq!(convert_status(TorrentStatus::Seeding), "Seeding");
    }

    #[test]
    fn test_convert_eta() {
        assert_eq!(convert_eta(-1), "Unknown");
        assert_eq!(convert_eta(-2), "Inf");
        assert_eq!(convert_eta(86400 * 3600), "Inf");
        assert_eq!(convert_eta(0), "");
        assert_eq!(convert_eta(1), "1s");
        assert_eq!(convert_eta(60), "1m");
        assert_eq!(convert_eta(3600), "1h");
        assert_eq!(convert_eta(86400), "1d");
        assert_eq!(convert_eta(86400 + 3600), "1d");
    }

    #[test]
    fn test_convert_percentage() {
        assert_eq!(convert_percentage(0.0), "0.0%");
        assert_eq!(convert_percentage(0.0003), "0.0%");
        assert_eq!(convert_percentage(0.003), "0.3%");
        assert_eq!(convert_percentage(0.5), "50.0%");
        assert_eq!(convert_percentage(1.0), "Done");
        assert_eq!(convert_percentage(1.1), "Done");
    }
}
