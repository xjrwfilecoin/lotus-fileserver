use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SectorNotifyForm {
    pub filename: String,
    pub sector_id: String,
}
