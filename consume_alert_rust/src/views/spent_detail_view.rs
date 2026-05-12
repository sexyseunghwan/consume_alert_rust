use crate::common::*;
use std::fmt;

#[doc = "Structure containing spent detail information."]
#[derive(Debug, Serialize, Deserialize, Clone, Getters, Setters, new)]
#[getset(get = "pub", set = "pub")]
pub struct SpentDetailView {
    pub spent_name: String,
    pub spent_money: String,
    pub spent_at: DateTime<FixedOffset>,
    pub consume_keyword_type_nm: String,
}

impl SpentDetailView {
    /// Formats the spending detail as a Telegram-friendly message string.
    ///
    /// # Returns
    ///
    /// Returns a formatted string containing the spending name, amount, time, and category.
    pub fn to_telegram_string(&self) -> String {
        format!(
            "사용처: \"{}\"\n사용한 현금: \"{}\"\n사용시간: \"{}\"\n소비타입: \"{}\"",
            self.spent_name,
            self.spent_money,
            self.spent_at.format("%Y-%m-%dT%H:%M"),
            self.consume_keyword_type_nm,
        )
    }

    /// Formats the spending detail as a Telegram-friendly deletion message string.
    ///
    /// # Returns
    ///
    /// Returns a formatted string containing the spending name, amount, time, category, and the current KST deletion time.
    pub fn to_telegram_string_to_delete(&self) -> String {
        let deleted_at: String = Utc::now()
            .with_timezone(&Seoul)
            .format("%Y-%m-%dT%H:%M")
            .to_string();

        format!(
            "[삭제된 결제 정보]\n사용처: \"{}\"\n사용한 현금: \"{}\"\n사용시간: \"{}\"\n소비타입: \"{}\"\n삭제시각: \"{}\"",
            self.spent_name,
            self.spent_money,
            self.spent_at.format("%Y-%m-%dT%H:%M"),
            self.consume_keyword_type_nm,
            deleted_at,
        )
    }
}

impl fmt::Display for SpentDetailView {
    /// Formats the `SpentDetailView` using the Telegram message string representation.
    ///
    /// # Arguments
    ///
    /// * `f` - The formatter to write into
    ///
    /// # Errors
    ///
    /// Returns an error if the write operation fails.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_telegram_string())
    }
}
