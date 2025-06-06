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
              2025-06-00 Seunghwan Shin       # [v.3.1.0] Added Shinhan Card Payment Details
*/
mod common;
use common::*;
mod repository;
mod utils_modules;

mod schema;

mod services;
use services::elastic_query_service::*;
use services::graph_api_service::*;
use services::mysql_query_service::*;
use services::process_service::*;
use services::telebot_service::*;

mod controller;
use controller::main_controller::*;

mod configuration;

mod models;

mod enums;

#[tokio::main]
async fn main() {
    /* Select compilation environment */
    dotenv().ok();

    /* Initiate Logger */
    set_global_logger();
    infok("Consume Alert Program Start").await;

    /* Telegram Bot object data */
    let bot: Arc<Bot> = Arc::new(Bot::from_env());

    let graph_api_service: Arc<GraphApiServicePub> = Arc::new(GraphApiServicePub::new());
    let elastic_query_service: Arc<ElasticQueryServicePub> =
        Arc::new(ElasticQueryServicePub::new());
    let mysql_query_service: Arc<MysqlQueryServicePub> = Arc::new(MysqlQueryServicePub::new());
    let process_service: Arc<ProcessServicePub> = Arc::new(ProcessServicePub::new());

    /* As soon as the event comes in, the code below continues to be executed. */
    teloxide::repl(Arc::clone(&bot), move |message: Message, bot: Arc<Bot>| {
        let graph_api_service_clone: Arc<GraphApiServicePub> = Arc::clone(&graph_api_service);
        let elastic_query_service_clone: Arc<ElasticQueryServicePub> =
            Arc::clone(&elastic_query_service);
        let mysql_query_service_clone: Arc<MysqlQueryServicePub> = Arc::clone(&mysql_query_service);
        let process_service_clone: Arc<ProcessServicePub> = Arc::clone(&process_service);

        async move {
            let tele_bot_service: TelebotServicePub = TelebotServicePub::new(bot, message);
            let main_controller: MainController<
                GraphApiServicePub,
                ElasticQueryServicePub,
                MysqlQueryServicePub,
                TelebotServicePub,
                ProcessServicePub,
            > = MainController::new(
                graph_api_service_clone,
                elastic_query_service_clone,
                mysql_query_service_clone,
                tele_bot_service,
                process_service_clone,
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
    .await;
}
