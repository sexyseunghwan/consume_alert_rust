pub use std::{
    io::Write,
    env, fs, cmp, thread,
    time::Duration,
    sync::{ Arc, Mutex, MutexGuard },
    collections::{HashMap, VecDeque},
    path::Path,
    cmp::Ordering,
    future::Future,
    str::FromStr
};

pub use rand:: {
    prelude::SliceRandom,
    rngs::StdRng,
    SeedableRng
};

pub use tokio::{
    sync::OnceCell,
    task
};

pub use futures::future::Lazy;
pub use log::{info, error};

pub use flexi_logger::{
    Logger, FileSpec, Criterion, Age, Naming, Cleanup, Record
};

pub use chrono::{DateTime, Utc, NaiveDateTime, NaiveDate, Datelike, TimeZone, Weekday, NaiveTime, Timelike};
pub use chrono_tz::Asia::Seoul;

pub use serde::{
    Serialize, Deserialize
};

pub use serde_json::{
    json, Value, from_value
};

pub use serde::de::DeserializeOwned;

pub use dotenv::dotenv;

pub use elasticsearch::{
    Elasticsearch, 
    DeleteByQueryParts,
    http::transport::{ SingleNodeConnectionPool, TransportBuilder, MultiNodeConnectionPool},
    http::Url,
    http::response::Response,
    SearchParts, 
    IndexParts, 
    DeleteParts,
    http::transport::{ Transport, ConnectionPool }
};

pub use anyhow::{
    Result, anyhow, Context
};

pub use getset::Getters;
pub use derive_new::new;

pub use teloxide:: {
    prelude::*,
    types::{ Message, InputFile },
    Bot
};

pub use reqwest::Client;

pub use regex::Regex;

pub use num_format::{Locale, ToFormattedString};

// pub use rdkafka:: {
//     config::ClientConfig,
//     consumer::Consumer,
//     producer::{FutureProducer, FutureRecord},
//     message::Message as KafkaMessage
// };

// pub use kafka::{
//     producer::{Producer, Record as KafkaRecord, RequiredAcks}
// };

pub use kafka::producer::{Producer, Record as KafkaRecord, RequiredAcks};


pub use async_trait::async_trait;

//use crate::repository::es_repository::*;
//use crate::repository::kafka_repository::*;
//pub static ELASTICSEARCH_CLIENT: OnceCell<Arc<EsRepositoryPub>> = OnceCell::new();
//pub static KAFKA_PRODUCER: OnceCell<Arc<KafkaRepositoryPub>> = OnceCell::const_new();

pub use crate::utils_modules::logger_utils::*;

pub use once_cell::sync::Lazy as once_lazy;

pub use strsim::levenshtein;

pub use rayon::prelude::*;


/* Elasticsearch index name to use globally */
pub static CONSUME_DETAIL: &str = "consuming_index_prod_new_v2";
pub static CONSUME_DETAIL_REMOVE: &str = "consuming_index_prod_new_remove";
pub static CONSUME_TYPE: &str = "consuming_index_prod_type_v2";
pub static MEAL_CHECK: &str = "meal_check_index";