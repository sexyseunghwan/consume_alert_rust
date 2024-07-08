pub use std::io::{Read, Write};
pub use std::env;
pub use std::time::{Duration, Instant};
pub use std::sync::{Arc, Mutex};

pub use log::{info, error};

pub use flexi_logger::{Logger, FileSpec, Criterion, Age, Naming, Cleanup, Record};

pub use chrono::{DateTime, Utc, NaiveDateTime, NaiveDate, Timelike, Datelike, TimeZone};
pub use chrono_tz::Asia::Seoul;
pub use chrono::offset::LocalResult;

pub use serde::{Serialize, Deserialize};
pub use serde_json::{json, Value};
pub use serde_json::ser::State;

pub use dotenv::dotenv;


pub use elasticsearch::{
    Elasticsearch, Error, http::transport::{Transport, SingleNodeConnectionPool}
};
pub use elasticsearch::http::transport::TransportBuilder;
pub use elasticsearch::http::Url;
pub use elasticsearch::{SearchParts, IndexParts, CountParts};

pub use plotters::prelude::*;

pub use anyhow::{Result, anyhow, Context};

pub use getset::{Getters, Setters};
pub use derive_new::new;

pub use teloxide::prelude::*;
pub use teloxide::types::Message;
pub use teloxide::Bot;

pub use regex::Regex;

pub use once_cell::sync::Lazy;


use crate::service::es_service::*;

pub static ES_CLIENT: Lazy<Arc<Mutex<EsHelper>>> = Lazy::new(|| {
    
    dotenv().ok();

    let es_host: Vec<String> = env::var("ES_DB_URL")
        .expect("'ES_DB_URL' must be set")
        .split(',')
        .map(|s| s.to_string())
        .collect();

    let es_id = env::var("ES_ID").expect("'ES_ID' must be set");
    let es_pw = env::var("ES_PW").expect("'ES_PW' must be set");

    let es_client = EsHelper::new(es_host, &es_id, &es_pw)
        .expect("Failed to create Elasticsearch client");

    Arc::new(Mutex::new(es_client))
});