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
    
    let es_host: Vec<String> = env::var("ES_DB_URL").expect("[ENV file read Error] 'ES_DB_URL' must be set").split(',').map(|s| s.to_string()).collect();
    let es_id = env::var("ES_ID").expect("[ENV file read Error] 'ES_ID' must be set");
    let es_pw = env::var("ES_PW").expect("[ENV file read Error] s'ES_PW' must be set");

    // Elasticsearch connection
    let es_client: EsHelper = match EsHelper::new(es_host, &es_id, &es_pw) {
        Ok(es_client) => es_client,
        Err(err) => {
            error!("[DB Connection Error] Failed to create mysql client - main_controller() // {:?}", err);
            panic!("[DB Connection Error] Failed to create mysql client - main_controller() // {:?}", err);
        }
    };
    
    // Telegram Bot - Read bot information from the ".env" file.
    let bot = Bot::from_env();
    
    // It wraps the Elasticsearch connection object with "Arc" for secure use in multiple threads.
    let arc_es_client: Arc<EsHelper> = Arc::new(es_client);

    //The ability to handle each command.
    teloxide::repl(bot, move |message: Message, bot: Bot| {

        let arc_es_client_clone = arc_es_client.clone();

        async move {
            match handle_command(&message, &bot, &arc_es_client_clone).await {
                Ok(_) => (),
                Err(e) => {
                    error!("{:?}", e);
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
async fn handle_command(message: &Message, bot: &Bot, arc_es_client_clone: &Arc<EsHelper>) -> Result<(), anyhow::Error> {
    
    if let Some(text) = message.text() {
        if text.starts_with("/c ") {
            command_consumption(message, text, bot, arc_es_client_clone).await?;
        } 
        else if text.starts_with("/cm") {
            command_consumption_per_mon(message, text, bot, arc_es_client_clone).await?;
        }
        else if text.starts_with("/ctr") {
            command_consumption_per_term(message, text, bot, arc_es_client_clone).await?;
        }
        else if text.starts_with("/ct") {
            command_consumption_per_day(message, text, bot, arc_es_client_clone).await?;
        }
        else if text.starts_with("/cs") {
            command_consumption_per_salary(message, text, bot, arc_es_client_clone).await?;
        }
        else if text.starts_with("/cw") {
            command_consumption_per_week(message, text, bot, arc_es_client_clone).await?;
        }
        else if text.starts_with("/mc") {
            command_record_fasting_time(message, text, bot, arc_es_client_clone).await?;
        }
        else if text.starts_with("/mt") {
            command_check_fasting_time(message, text, bot, arc_es_client_clone).await?;
        }
        else if text.starts_with("/md") {
            command_delete_fasting_time(message, text, bot, arc_es_client_clone).await?;
        }
        else if text.starts_with("/cy") {
            command_consumption_per_year(message, text, bot, arc_es_client_clone).await?;
        }
        else if text.starts_with("/a") {
            command_consumption_auto(message, text, bot, arc_es_client_clone).await?;
        }
        else 
        {
            bot.send_message(message.chat.id, "Hello! Use /c <args> to interact.")
                .await
                .context("[handle command Error] Failed to send default interaction message - handle_command() // {:?}")?;
        }
    }
    Ok(())
}