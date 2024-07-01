use crate::common::*;

use crate::service::es_service::*;




/*
    ======================================================
    ============= Telegram Bot Controller =============
    ======================================================
*/
pub async fn main_controller() {

    // Select compilation environment
    dotenv().ok();

    //let tele_token: String = env::var("COMPILE_ENV").expect("'COMPILE_ENV' must be set");

    let es_host: Vec<String> = env::var("ES_DB_URL").expect("'ES_DB_URL' must be set").split(",").map(|s| s.to_string()).collect();
    let es_id = env::var("ES_ID").expect("'ES_ID' must be set");
    let es_pw = env::var("ES_PW").expect("'ES_PW' must be set");

    // Elasticsearch connection
    let es_client = match EsHelper::new(es_host, &es_id, &es_pw) {
        Ok(es_client) => es_client,
        Err(err) => {
            error!("Failed to create mysql client: {:?}", err);
            panic!("Failed to create mysql client: {:?}", err);
        }
    };
    
    let bot = Bot::from_env();

    teloxide::repl(bot, |message: Message, bot: Bot| async move {
        match handle_command(&message, &bot).await {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Error handling message: {:?}", e);
                bot.send_message(message.chat.id, format!("Error: {}", e))
                    .await
                    .log_on_error()
                    .await;
            }
        };
        respond(())
    })
    .await;
            
    
}


/*
    
*/
async fn handle_command(message: &Message, bot: &Bot) -> Result<()> {
    if let Some(text) = message.text() {
        if text.starts_with("/c ") {
            let args = &text[3..];
            
            println!("{:?}", args);

            bot.send_message(message.chat.id, format!("Command with args: {}", args))
                .await
                .context("Failed to send command response")?;
            
            bot.send_message(message.chat.id, format!("Command with args: {}", args))
                .await
                .context("Failed to send command response")?;
            
        } else {
            bot.send_message(message.chat.id, "Hello! Use /c <args> to interact.")
                .await
                .context("Failed to send default interaction message")?;
        }
    }
    Ok(())
}