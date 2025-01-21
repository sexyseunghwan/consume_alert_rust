use crate::common::*;

#[async_trait]
pub trait GraphApiService {}

#[derive(Debug, Getters, Clone, new)]
pub struct GraphApiServicePub;

#[async_trait]
impl GraphApiService for GraphApiServicePub {}
