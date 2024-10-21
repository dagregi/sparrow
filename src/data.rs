use std::{cell::RefCell, rc::Rc};

use chrono::{DateTime, Utc};
use color_eyre::Result;
use itertools::Itertools;
use transmission_rpc::{types::Id, TransClient};

use crate::{
    app,
    utils::{
        convert_bytes, convert_eta, convert_percentage, convert_priority, convert_status,
        handle_ratio,
    },
};

#[derive(Debug, Clone)]
pub struct Torrent {
    pub id: i64,
    pub is_stalled: bool,
    pub status: String,
    pub name: String,
    pub formatted_name: String,
    pub percent_done: String,
    pub total_size: String,
    pub size_done: String,
    pub uploaded: String,
    pub upload_speed: String,
    pub downloaded: String,
    pub download_speed: String,
    pub ratio: String,
    pub location: String,
    pub hash: String,
    pub added_date: DateTime<Utc>,
    pub done_date: DateTime<Utc>,
    pub eta: String,
    pub error: String,

    pub trackers: Vec<Tracker>,
    pub files: Vec<Files>,
}

#[derive(Debug, Clone)]
pub struct Tracker {
    pub host: String,
    pub is_backup: bool,
    pub next_announce: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Files {
    pub name: String,
    pub downloaded: String,
    pub total_size: String,
    pub priority: String,
    pub wanted: bool,
}

impl Torrent {
    pub const fn ref_array(&self) -> [&String; 6] {
        [
            &self.formatted_name,
            &self.percent_done,
            &self.eta,
            &self.download_speed,
            &self.upload_speed,
            &self.ratio,
        ]
    }

    pub fn formatted_name(&self) -> &str {
        &self.formatted_name
    }
    pub fn percent_done(&self) -> &str {
        &self.percent_done
    }
    pub fn eta(&self) -> &str {
        &self.eta
    }
    pub fn upload_speed(&self) -> &str {
        &self.upload_speed
    }
    pub fn download_speed(&self) -> &str {
        &self.download_speed
    }
    pub fn ratio(&self) -> &str {
        &self.ratio
    }
}

pub async fn map_torrent_data(
    client: &Rc<RefCell<TransClient>>,
    id: Option<i64>,
) -> Result<Vec<Torrent>, app::Error> {
    let res = {
        let mut client = client.borrow_mut();
        async move {
            match id {
                Some(id) => client.torrent_get(None, Some(vec![Id::Id(id)])).await,
                None => client.torrent_get(None, None).await,
            }
        }
    }
    .await;

    let torrents = match res {
        Ok(t) => t.arguments.torrents,
        Err(err) => return Err(app::Error::WithMessage(err.to_string())),
    };

    Ok(torrents
        .iter()
        .filter_map(|t| {
            let t = t.clone();
            let trackers = t
                .tracker_stats?
                .iter()
                .map(|tr| Tracker {
                    host: tr.host.to_string(),
                    is_backup: tr.is_backup,
                    next_announce: tr.next_announce_time,
                })
                .collect_vec();
            let files = t
                .files?
                .iter()
                .enumerate()
                .filter_map(|(i, f)| {
                    let file_stats = t.file_stats.clone()?;
                    Some(Files {
                        name: f.name.to_string(),
                        downloaded: convert_bytes(f.bytes_completed),
                        total_size: convert_bytes(f.length),
                        priority: convert_priority(&file_stats.get(i)?.priority),
                        wanted: file_stats.get(i)?.wanted,
                    })
                })
                .collect_vec();

            let mut raw_name = t.name.clone()?;
            if raw_name.len() > 80 {
                raw_name.truncate(80);
                raw_name.push_str("...");
            }
            let status = convert_status(t.status?);
            let downloaded = convert_bytes(t.size_when_done? - t.left_until_done?);
            let size_done = convert_bytes(t.size_when_done?);
            let formatted_name =
                format!("{raw_name}\nStatus: {status}    Have: {downloaded} of {size_done}");

            Some(Torrent {
                id: t.id?,
                is_stalled: t.is_stalled?,
                status,
                name: t.name?,
                formatted_name,
                eta: convert_eta(t.eta?),
                ratio: handle_ratio(t.upload_ratio?),
                percent_done: convert_percentage(t.percent_done?),
                total_size: convert_bytes(t.total_size?),
                size_done,
                uploaded: convert_bytes(t.uploaded_ever?),
                upload_speed: format!("{}/s", convert_bytes(t.rate_upload?)),
                downloaded,
                download_speed: format!("{}/s", convert_bytes(t.rate_download?)),
                location: t.download_dir?,
                hash: t.hash_string?,
                added_date: DateTime::from_timestamp(t.added_date?, 0)?,
                done_date: DateTime::from_timestamp(t.done_date?, 0)?,
                error: t.error_string?,
                trackers,
                files,
            })
        })
        .sorted_by(|a, b| a.name.cmp(&b.name))
        .collect_vec())
}
