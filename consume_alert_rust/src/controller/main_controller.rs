use crate::common::*;

use crate::service::command_service::*;
use crate::utils_modules::db_connection_utils::*;

pub static ELASTICSEARCH_CLIENT: OnceCell<Arc<CommandService>> = OnceCell::new();


/*
    ======================================================
    ============= Telegram Bot Controller =============
    ======================================================
*/
pub async fn main_controller() {

    // Select compilation environment
    dotenv().ok();
    
    let bot = Bot::from_env();
    let _ = initialize_db_clients().await;

    infok("Program Start").await;

    //The ability to handle each command.
    teloxide::repl(bot, move |message: Message, bot: Bot| {

        async move {
            match handle_command(message, bot).await {
                Ok(_) => (),
                Err(e) => {
                    errork(e).await;
                }
            };
            respond(())
        }

    })
    .await;
    
}


/*
    Functions that handle each command
*/
async fn handle_command(message: Message, bot: Bot) -> Result<(), anyhow::Error> {
    
        
    let command_service = CommandService::new(bot, message);    
    
    let input_text = message
        .text()
        .ok_or_else(|| anyhow!("[Error][handle_commandhandle_command()] The entered value does not exist."))?
        .to_lowercase();
    
    if input_text.starts_with("c ") {
        command_service.command_consumption(message, &input_text, bot).await?;
    } 
    else if input_text.starts_with("cm") {
        command_service.command_consumption_per_mon(message, &input_text, bot).await?;
    }
    else if input_text.starts_with("ctr") {
        command_service.command_consumption_per_term(message, &input_text, bot).await?;
    }
    else if input_text.starts_with("ct") {
        command_service.command_consumption_per_day(message, &input_text, bot).await?;
    }
    else if input_text.starts_with("cs") {
        command_service.command_consumption_per_salary(message, &input_text, bot).await?;
    }
    else if input_text.starts_with("cw") {
        command_service.command_consumption_per_week(message, &input_text, bot).await?;
    }
    else if input_text.starts_with("mc") {
        command_service.command_record_fasting_time(message, &input_text, bot).await?;
    }
    else if input_text.starts_with("mt") {
        command_service.command_check_fasting_time(message, &input_text, bot).await?;
    }
    else if input_text.starts_with("md") {
        command_service.command_delete_fasting_time(message, &input_text, bot).await?;
    }
    else if input_text.starts_with("cy") {
        command_service.command_consumption_per_year(message, &input_text, bot).await?;
    }
    else if input_text.starts_with("list") {
        command_service.command_get_consume_type_list(message, &input_text, bot).await?;
    }
    else 
    {
        command_service.command_consumption_auto(message, &input_text, bot).await?;
    }
    
    Ok(())
}