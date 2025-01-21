use crate::common::*;

use crate::repository::es_repository::*;

#[async_trait]
pub trait EsQueryService {}

#[derive(Debug, Getters, Clone, new)]
pub struct EsQueryServicePub;

#[async_trait]
impl EsQueryService for EsQueryServicePub {}
