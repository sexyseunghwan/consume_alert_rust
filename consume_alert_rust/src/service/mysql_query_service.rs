use crate::common::*;

pub trait MySqlQueryService {}

#[derive(Debug, Getters, Clone, new)]
pub struct MySqlQueryServicePub;

impl MySqlQueryService for MySqlQueryServicePub {}
