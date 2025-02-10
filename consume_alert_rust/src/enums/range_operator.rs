#[derive(Debug, Clone, Copy)]
pub enum RangeOperator {
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
}

impl RangeOperator {
    pub fn as_str(&self) -> &'static str {
        match self {
            RangeOperator::GreaterThan => "gt",
            RangeOperator::GreaterThanOrEqual => "gte",
            RangeOperator::LessThan => "lt",
            RangeOperator::LessThanOrEqual => "lte",
        }
    }
}
