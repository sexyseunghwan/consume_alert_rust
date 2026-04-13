use crate::common::*;

use crate::enums::indexing_type::*;

#[derive(Debug, Clone, Serialize, Deserialize, new)]
pub struct SpentDetailToKafka {
    pub spent_idx: i64,
    pub indexing_type: String,
    pub reg_at: DateTime<Utc>,
}

impl SpentDetailToKafka {
    pub fn convert_indexing_type(&self) -> anyhow::Result<IndexingType> {
        self.indexing_type
            .parse::<IndexingType>()
            .map_err(|e| anyhow::anyhow!(e))
    }
}
