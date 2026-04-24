use crate::common::*;

use crate::enums::indexing_type::*;

#[derive(Debug, Clone, Serialize, Deserialize, new)]
pub struct SpentDetailToKafka {
    pub spent_idx: i64,
    pub indexing_type: String,
    pub reg_at: DateTime<Utc>,
}

impl SpentDetailToKafka {
    /// Parses the `indexing_type` string field into an `IndexingType` enum variant.
    ///
    /// # Returns
    ///
    /// Returns `Ok(IndexingType)` on successful parsing.
    ///
    /// # Errors
    ///
    /// Returns an error if the `indexing_type` string does not match any known variant.
    #[allow(dead_code)]
    pub fn to_indexing_type(&self) -> anyhow::Result<IndexingType> {
        self.indexing_type
            .parse::<IndexingType>()
            .map_err(|e| anyhow::anyhow!(e))
    }
}
