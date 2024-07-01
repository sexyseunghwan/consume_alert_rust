/*
Author      : Seunghwan Shin 
Create date : 2023-02-06 
Description : 
    
History     : 2023-02-06 Seunghwan Shin       # first create

*/ 
mod common;
mod controller;
mod utils_modules;
// mod service;
// mod dtos;

use utils_modules::logger_utils::*;
//use controller::main_controller::*;

#[tokio::main]
async fn main() {
    
    // Initiate Logger
    set_global_logger();

    // Start Controller
    //main_controller().await;
}
