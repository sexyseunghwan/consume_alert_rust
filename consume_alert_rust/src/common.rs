pub use std::io::Write;
pub use std::{env, fs, cmp, thread};
pub use std::time::Duration;
pub use std::sync::Arc;
pub use std::collections::HashMap;
pub use std::path::Path;
pub use std::cmp::Ordering;
pub use std::future::Future;

pub use rand::prelude::SliceRandom;

pub use tokio::sync::OnceCell;

pub use futures::future::Lazy;
pub use log::{info, error};

pub use flexi_logger::{Logger, FileSpec, Criterion, Age, Naming, Cleanup, Record};

pub use chrono::{DateTime, Utc, NaiveDateTime, NaiveDate, Datelike, TimeZone, Weekday, NaiveTime, Timelike};
pub use chrono_tz::Asia::Seoul;

pub use serde::{Serialize, Deserialize};
pub use serde_json::{json, Value, from_value};
pub use serde::de::DeserializeOwned;

pub use dotenv::dotenv;

pub use elasticsearch::{
    Elasticsearch, http::transport::SingleNodeConnectionPool
};
pub use elasticsearch::http::transport::TransportBuilder;
pub use elasticsearch::http::Url;
pub use elasticsearch::{SearchParts, IndexParts, DeleteParts};
pub use elasticsearch::http::transport::Transport;
pub use elasticsearch::http::transport::ConnectionPool;
pub use anyhow::{Result, anyhow, Context};
pub use elasticsearch::http::response::Response;

pub use getset::Getters;
pub use derive_new::new;

pub use teloxide::prelude::*;
pub use teloxide::types::Message;
pub use teloxide::Bot;
pub use teloxide::types::InputFile;

pub use regex::Regex;

pub use num_format::{Locale, ToFormattedString};

pub use rdkafka::config::ClientConfig;
pub use rdkafka::consumer::Consumer;
pub use rdkafka::producer::{FutureProducer, FutureRecord};
pub use rdkafka::message::Message as KafkaMessage;

pub use async_trait::async_trait;

use crate::repository::es_repository::*;
use crate::service::kafka_service::ProduceBroker;

//pub static ELASTICSEARCH_CLIENT: OnceCell<Arc<EsHelper>> = OnceCell::const_new();
pub static ELASTICSEARCH_CLIENTS: OnceCell<Arc<EsRepositoryPub>> = OnceCell::new();
pub static KAFKA_PRODUCER: OnceCell<Arc<ProduceBroker>> = OnceCell::const_new();

pub use crate::utils_modules::logger_utils::*;

