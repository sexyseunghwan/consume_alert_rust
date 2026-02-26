/*
Author      : Seunghwan Shin
Create date : 2023-05-01
Description : Code that can perform various functions through Telegram

History     : 2023-05-04 Seunghwan Shin       # [v.1.0.0] first create
              2023-05-20 Seunghwan Shin       # [v.1.1.0] Applying Logging Algorithms
              2023-07-28 Seunghwan Shin       # [v.1.2.0] Add consumption pattern function
              2023-07-29 Seunghwan Shin       # [v.1.3.0] Change standard time to Korean time zone
              2023-07-30 Seunghwan Shin       # [v.1.4.0]
                                              # 1) Set access rights for each TELEGRAM group
                                              # 2) Changed to create and manage Elasticsearch-only objects
                                              # 3) When you want to see the money spent in a specific month,
                                                   if you do not pass the parameter, change to show the consumption for the current month
              2023-08-02 Seunghwan Shin       # [v.1.5.0] Change the source so that you can look up the amount of money consumed by payday
              2023-08-04 Seunghwan Shin       # [v.1.6.0] Change the source to look up weekly consumption amount
              2023-08-06 Seunghwan Shin       # [v.1.7.0] Added function to record meal time
              2023-08-07 Seunghwan Shin       # [v.1.8.0] Added a function to check how long the fasting time has been
              2023-08-08 Seunghwan Shin       # [v.1.9.0] Added a function to remove the last data from the index if meal time is entered incorrectly
              2023-08-11 Seunghwan Shin       # [v.1.10.0] Add payment cancellation processing
              2023-08-13 Seunghwan Shin       # [v.1.10.1]
                                              # "ERROR: Message is too long" problem solving -> Changed the text to be cut off at regular intervals and sent to the chat room
                                              # Change the source code so that the telegram bot sends a message by creating a telebot internal method.
                                              # Add "exc_info=True" statement to exception handling -> When an exception occurs, you can find out which line it occurred on.
              2023-08-14 Seunghwan Shin       # [v.1.10.2] Change time format to "%Y-%m-%dT%H:%M:%SZ"
              2023-08-21 Seunghwan Shin       # [v.1.11.0] Modify source code to check yearly consumption details
              2023-08-22 Seunghwan Shin       # [v.1.11.1]
                                              # The command parameter check was confirmed to be unnecessary and removed.
                                              # Add exception handling statement to All function
              2023-08-23 Seunghwan Shin       # [v.1.12.0] Add logic to input specific time to meal check function
              2023-08-25 Seunghwan Shin       # [v.1.12.1] When entering a specific time for meal time, an issue occurred where the confirmation time was displayed as the current time,
                                                so the problem was corrected.
              2023-08-27 Seunghwan Shin       # [v.1.12.1]
                                                1) Changed the return value of the get_consume_total_cost() function to be returned after converting it from the existing json format to an integer format.
                                                2) Implementation of a function that shows yearly consumption details
              2023-11-27 Seunghwan Shin       # [v.1.12.2]
                                                1) Modify source code to change logging algorithm -> Changed so that logger can be used globally
                                                2) Change the permission information storage to MongoDB
                                                3) Perform overall source code refactoring
                                                4) A "TIMEOUT ERROR" occurs when searching for a long period of time.
              2023-11-30 Seunghwan Shin       # [v.1.12.3]
                                                1) Added a function to view consumption details on a specific date
                                                2) Fixed an issue where messages were not sent if there was no consumption history
              2024-01-13 Seunghwan Shin       # [v.1.12.4]
                                                1) If the fasting time is long, a problem occurs when accessing the meal_check_index index
                                                    => Previously, only data within 24 hours was searched.
                                                    If there is no data within 24 hours, search in 48 hours.
                                                    If there is no data within 48 hours, it is 72 hours. Use logic to query.
                                                2) Create a meeting-related index (promise_check_index) and add logic to index data into the index.
              2024-05-28 Seunghwan Shin       # [v.2.0.0] Change source code to manage information such as db connection as a ".env" file. -> RUST transfer
              2024-05-30 Seunghwan Shin       # [v.2.1.0] Developing a function to graph consumption trends.
              2024-06-22 Seunghwan Shin       # [v.2.1.1] Increase the size of the consumption graph.
              2024-08-24 Seunghwan Shin       # [v.2.1.2] Add code to exclude from aggregation if consumption by category is zero
              2024-09-01 Seunghwan Shin       # [v.2.1.3] If there is no consumption details during the entered period, do not show consumption-related graphs
              2024-09-08 Seunghwan Shin       # [v.2.1.4] Change command calls in a simpler way
              2024-09-12 Seunghwan Shin       # [v.2.2.0] Add list command
              2024-09-17 Seunghwan Shin       # [v.2.2.1] Manage logs with ''KAFKA'' -> Elasticsearch with 'logstash'
              2024-09-19 Seunghwan Shin       # [v.2.2.2] Lowercase Input Processing
              2025-01-28 Seunghwan Shin       # [v.3.0.0] Change the overall code structure
              2025-02-03 Seunghwan Shin       # [v.3.0.1] Identify and correct aggregation problems
              2025-02-10 Seunghwan Shin       # [v.3.0.2] Modifying code because there is a problem with the command 'ct'
              2025-02-10 Seunghwan Shin       # [v.3.0.3] Changed the code to disable Kafka for a while.
              2025-06-07 Seunghwan Shin       # [v.3.1.0] Added Shinhan Card Payment Details
              2025-10-04 Seunghwan Shin       # [v.3.1.1] Prevented negative values from being displayed in the pie chart by hiding those sections.
              2026-00-00 Seunghwan Shin       # [v.4.0.0]
*/
mod common;
use common::*;

mod config;
use config::AppConfig;

mod repository;
use repository::{es_repository::*, kafka_repository::*, mysql_repository::*, redis_repository::*};

mod utils_modules;

mod schema;

mod services;

use services::{
    elastic_query_service::*, graph_api_service::*, mysql_query_service::*, process_service::*,
    producer_service::*, redis_service::*, telebot_service::*,
};

mod controller;
use controller::main_controller::*;

mod configuration;

mod models;

mod enums;

mod entity;

mod views;

/* ─── Concrete service types used throughout main ─────────────────────────── */
type AppRedisService = RedisServiceImpl<RedisRepositoryImpl>;
type AppElasticService = ElasticQueryServicePub<EsRepositoryPub>;
type AppMysqlService = MysqlQueryServiceImpl<MysqlRepositoryImpl>;
type AppProducerService = ProducerServiceImpl<KafkaRepositoryImpl>;
/* ─────────────────────────────────────────────────────────────────────────── */

#[tokio::main]
async fn main() {
    /* Select compilation environment */
    dotenv().ok();

    /* Initiate Logger */
    set_global_logger();
    info!("Consume Alert Program Start");

    /* Initialize global configuration */
    AppConfig::init().expect("Failed to initialize AppConfig");

    let elastic_conn: EsRepositoryPub = match EsRepositoryPub::new() {
        Ok(elastic_conn) => elastic_conn,
        Err(e) => {
            error!("[main] elastic_conn: {:#}", e);
            panic!("[main] elastic_conn: {:#}", e)
        }
    };

    let mysql_conn: MysqlRepositoryImpl = match MysqlRepositoryImpl::new().await {
        Ok(mysql_conn) => mysql_conn,
        Err(e) => {
            error!("[main] mysql_conn: {:#}", e);
            panic!("[main] mysql_conn: {:#}", e)
        }
    };

    let kafka_conn: KafkaRepositoryImpl = match KafkaRepositoryImpl::new() {
        Ok(kafka_conn) => kafka_conn,
        Err(e) => {
            error!("[main] kafka_conn: {:#}", e);
            panic!("[main] kafka_conn: {:#}", e)
        }
    };

    let redis_conn: RedisRepositoryImpl = match RedisRepositoryImpl::new().await {
        Ok(redis_conn) => redis_conn,
        Err(e) => {
            error!("[main] redis_conn: {:#}", e);
            panic!("[main] redis_conn: {:#}", e)
        }
    };

    /* Shared services — wrapped in Arc so each bot task can clone cheaply. */
    let redis_service: Arc<AppRedisService> = Arc::new(AppRedisService::new(redis_conn));
    let graph_api_service: Arc<GraphApiServicePub> = Arc::new(GraphApiServicePub::new());
    let elastic_query_service: Arc<AppElasticService> =
        Arc::new(AppElasticService::new(elastic_conn));
    let mysql_query_service: Arc<AppMysqlService> = Arc::new(AppMysqlService::new(mysql_conn));
    let process_service: Arc<ProcessServicePub> = Arc::new(ProcessServicePub::new());
    let producer_service: Arc<AppProducerService> = Arc::new(AppProducerService::new(kafka_conn));

    /* Build one Bot per token listed in BOT_TOKENS.
     * Each bot runs its own independent teloxide::repl loop in a separate
     * tokio task, but all bots share the same service instances via Arc. */
    let app_config: &AppConfig = AppConfig::global();
    let bots: Vec<Arc<Bot>> = app_config
        .bot_tokens()
        .iter()
        .map(|token| Arc::new(Bot::new(token)))
        .collect();

    info!("[main] Starting {} bot(s)", bots.len());

    let mut handles: Vec<task::JoinHandle<()>> = Vec::new();

    for bot in bots {
        let handle = tokio::spawn({
            /* Clone Arc pointers — cheap, no data is copied. */
            let graph_api_service: Arc<GraphApiServicePub> = Arc::clone(&graph_api_service);
            let elastic_query_service: Arc<AppElasticService> = Arc::clone(&elastic_query_service);
            let mysql_query_service: Arc<AppMysqlService> = Arc::clone(&mysql_query_service);
            let process_service: Arc<ProcessServicePub> = Arc::clone(&process_service);
            let producer_service: Arc<AppProducerService> = Arc::clone(&producer_service);
            let redis_service: Arc<AppRedisService> = Arc::clone(&redis_service);

            async move {
                info!(
                    "[main] Bot polling started (token prefix: {}...)",
                    &bot.token()[..8]
                );

                /* Each bot runs its own repl loop.
                 * teloxide::repl polls the Telegram API and dispatches messages
                 * to the handler closure one at a time (per bot). */
                teloxide::repl(bot, move |message: Message, bot: Arc<Bot>| {
                    let graph_api_service: Arc<GraphApiServicePub> = Arc::clone(&graph_api_service);
                    let elastic_query_service: Arc<AppElasticService> =
                        Arc::clone(&elastic_query_service);
                    let mysql_query_service: Arc<AppMysqlService> =
                        Arc::clone(&mysql_query_service);
                    let process_service: Arc<ProcessServicePub> = Arc::clone(&process_service);
                    let producer_service: Arc<AppProducerService> = Arc::clone(&producer_service);
                    let redis_service: Arc<AppRedisService> = Arc::clone(&redis_service);

                    async move {
                        let tele_bot_service: TelebotServicePub =
                            TelebotServicePub::new(bot, message);
                        let main_controller: MainController<
                            GraphApiServicePub,
                            ElasticQueryServicePub<EsRepositoryPub>,
                            MysqlQueryServiceImpl<MysqlRepositoryImpl>,
                            TelebotServicePub,
                            ProcessServicePub,
                            ProducerServiceImpl<KafkaRepositoryImpl>,
                            RedisServiceImpl<RedisRepositoryImpl>,
                        > = MainController::new(
                            graph_api_service,
                            elastic_query_service,
                            mysql_query_service,
                            tele_bot_service,
                            process_service,
                            producer_service,
                            redis_service,
                        );

                        match main_controller.main_call_function().await {
                            Ok(_) => {
                                info!("respond success.");
                            }
                            Err(e) => {
                                errork(e).await;
                            }
                        };

                        respond(())
                    }
                })
                .await
            }
        });

        handles.push(handle);
    }

    /* Wait for all bot tasks.  In normal operation they run forever;
     * if one task panics the error is logged but the others keep running. */
    for handle in handles {
        if let Err(e) = handle.await {
            error!("[main] Bot task ended unexpectedly: {:?}", e);
        }
    }
}
