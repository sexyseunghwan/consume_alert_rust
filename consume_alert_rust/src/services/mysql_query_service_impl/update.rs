use crate::repository::mysql_repository::*;

use super::MysqlQueryServiceImpl;

impl<R: MysqlRepository + Send + Sync> MysqlQueryServiceImpl<R> {}
