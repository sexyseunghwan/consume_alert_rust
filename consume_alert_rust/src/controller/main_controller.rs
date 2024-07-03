use crate::common::*;

use crate::service::es_service::*;
use crate::service::command_service::*;


/*
    ======================================================
    ============= Telegram Bot Controller =============
    ======================================================
*/
pub async fn main_controller() {

    // Select compilation environment
    dotenv().ok();
    
    let es_host: Vec<String> = env::var("ES_DB_URL").expect("'ES_DB_URL' must be set").split(",").map(|s| s.to_string()).collect();
    let es_id = env::var("ES_ID").expect("'ES_ID' must be set");
    let es_pw = env::var("ES_PW").expect("'ES_PW' must be set");

    // Elasticsearch connection
    let es_client: EsHelper = match EsHelper::new(es_host, &es_id, &es_pw) {
        Ok(es_client) => es_client,
        Err(err) => {
            error!("Failed to create mysql client: {:?}", err);
            panic!("Failed to create mysql client: {:?}", err);
        }
    };
    
    // Telegram Bot - Read bot information from the ".env" file.
    let bot = Bot::from_env();

    // It wraps the Elasticsearch connection object with "Arc" for secure use in multiple threads.
    let arc_es_client = Arc::new(es_client);

    // The ability to handle each command.
    teloxide::repl(bot, move |message: Message, bot: Bot| {
        let es_client_ref = Arc::clone(&arc_es_client);
        async move {
            match handle_command(&message, &bot, &es_client_ref).await {
                Ok(_) => (),
                Err(e) => {
                    error!("Error handling message: {:?}", e);
                    bot.send_message(message.chat.id, format!("Error: {}", e))
                       .await
                       .log_on_error()
                       .await;
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
async fn handle_command(message: &Message, bot: &Bot, es_client: &EsHelper) -> Result<(), anyhow::Error> {
    
    if let Some(text) = message.text() {
        if text.starts_with("/c ") {
            command_consumption(message, text, bot, es_client).await?;
        } 
        else if text.starts_with("/cm") {
            command_consumption_per_mon(message, text, bot, es_client).await?;
        }
        else {
            bot.send_message(message.chat.id, "Hello! Use /c <args> to interact.")
                .await
                .context("Failed to send default interaction message")?;
        }
    }
    Ok(())
}