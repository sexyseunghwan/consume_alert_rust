/*
Author      : Seunghwan Shin 
Create date : 2024-07-00 
Description : 
    
History     : 2024-07-00 Seunghwan Shin       # first create

*/ 
mod common;
mod controller;
mod utils_modules;
mod service;

use utils_modules::logger_utils::*;
use controller::main_controller::*;




#[tokio::main]
async fn main() {
    
    // Initiate Logger
    set_global_logger();

    // Start Controller
    main_controller().await;
}
