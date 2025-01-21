use crate::common::*;

#[async_trait]
pub trait MysqlQueryService {}

#[derive(Debug, Getters, Clone, new)]
pub struct MysqlQueryServicePub;

#[async_trait]
impl MysqlQueryService for MysqlQueryServicePub {}
