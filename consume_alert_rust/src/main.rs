/*
Author      : Seunghwan Shin 
Create date : 2023-05-01 
Description : Code that can perform various functions through Telegram
    
History     : 2023-05-04 Seunghwan Shin       # first create
              2023-05-20 Seunghwan Shin       # Applying Logging Algorithms
              2023-07-28 Seunghwan Shin       # Add consumption pattern function
              2023-07-29 Seunghwan Shin       # Change standard time to Korean time zone
              2023-07-30 Seunghwan Shin       # 1) Set access rights for each TELEGRAM group
                                              # 2) Changed to create and manage Elasticsearch-only objects
                                              # 3) When you want to see the money spent in a specific month, 
                                                   if you do not pass the parameter, change to show the consumption for the current month
              2023-08-02 Seunghwan Shin       # Change the source so that you can look up the amount of money consumed by payday  
              2023-08-04 Seunghwan Shin       # Change the source to look up weekly consumption amount  
              2023-08-06 Seunghwan Shin       # Added function to record meal time 
              2023-08-07 Seunghwan Shin       # Added a function to check how long the fasting time has been
              2023-08-08 Seunghwan Shin       # Added a function to remove the last data from the index if meal time is entered incorrectly
              2023-08-11 Seunghwan Shin       # Add payment cancellation processing
              2023-08-13 Seunghwan Shin       # "ERROR: Message is too long" problem solving -> Changed the text to be cut off at regular intervals and sent to the chat room
                                              # Change the source code so that the telegram bot sends a message by creating a telebot internal method.
                                              # Add "exc_info=True" statement to exception handling -> When an exception occurs, you can find out which line it occurred on.
              2023-08-14 Seunghwan Shin       # Change time format to "%Y-%m-%dT%H:%M:%SZ"
              2023-08-21 Seunghwan Shin       # Modify source code to check yearly consumption details
              2023-08-22 Seunghwan Shin       # The command parameter check was confirmed to be unnecessary and removed.
                                              # Add exception handling statement to All function
              2023-08-23 Seunghwan Shin       # Add logic to input specific time to meal check function
              2023-08-25 Seunghwan Shin       # When entering a specific time for meal time, an issue occurred where the confirmation time was displayed as the current time, 
                                              # so the problem was corrected.
              2023-08-27 Seunghwan Shin       # 1) Changed the return value of the get_consume_total_cost() function to be returned after converting it from the existing json format to an integer format.
                                              # 2) Implementation of a function that shows yearly consumption details
              2023-11-27 Seunghwan Shin       # 1) Modify source code to change logging algorithm -> Changed so that logger can be used globally
                                              # 2) Change the permission information storage to MongoDB 
                                              # 3) Perform overall source code refactoring 
                                              # 4) A "TIMEOUT ERROR" occurs when searching for a long period of time.
              2023-11-30 Seunghwan Shin       # 1) Added a function to view consumption details on a specific date
                                              # 2) Fixed an issue where messages were not sent if there was no consumption history 
              2024-01-13 Seunghwan Shin       # 1) If the fasting time is long, a problem occurs when accessing the meal_check_index index 
                                                    => Previously, only data within 24 hours was searched. 
                                                    If there is no data within 24 hours, search in 48 hours. 
                                                    If there is no data within 48 hours, it is 72 hours. Use logic to query.
                                                2) Create a meeting-related index (promise_check_index) and add logic to index data into the index.
              2024-05-28 Seunghwan Shin       # Change source code to manage information such as db connection as a ".env" file.
              2024-05-30 Seunghwan Shin       # Developing a function to graph consumption trends.      
              2024-06-02 Seunghwan Shin       # Increase the size of the consumption graph. 
              2024-06-22 Seunghwan Shin       # Increase the size of the consumption graph. 
              2024-08-24 Seunghwan Shin       # Add code to exclude from aggregation if consumption by category is zero
              2024-09-01 Seunghwan Shin       # If there is no consumption details during the entered period, do not show consumption-related graphs
              2024-09-08 Seunghwan Shin       # Change command calls in a simpler way
              2024-09-12 Seunghwan Shin       # Add list command
              2024-09-17 Seunghwan Shin       # Manage logs with ''KAFKA'' -> Elasticsearch with 'logstash'
*/ 
mod common;
mod controller;
mod utils_modules;
mod service;
mod dtos;

use utils_modules::logger_utils::*;
use controller::main_controller::*;
//use controller::test_controller::*;


#[tokio::main]
async fn main() {
    
    // Initiate Logger
    set_global_logger();

    // Start Controller
    main_controller().await;

    //test_controller().await;
}
