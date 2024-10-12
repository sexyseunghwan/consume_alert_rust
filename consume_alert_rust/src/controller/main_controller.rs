use crate::common::*;

use crate::service::command_service::*;

use crate::utils_modules::common_function::*;

/*
    ======================================================
    ============= Telegram Bot Controller =============
    ======================================================
*/
pub async fn main_controller() {

    // Select compilation environment
    dotenv().ok();
    let bot = Bot::from_env();
    
    initialize_db_connection();
    
    //info!("{:?}", get_kafka_producer());
    infok("Consume Alert Program Start").await;

    // let mut handles = vec![];

    // for i in 1..5 {
    //     // 스레드에서 비동기 작업 실행
    //     let handle = task::spawn(async move {
    //         let msg = format!("test {}", i);
    //         infok(&msg).await;
    //     });

    //     // 핸들 저장
    //     handles.push(handle);
    // }

    // // 모든 작업이 끝날 때까지 기다림
    // for handle in handles {
    //     handle.await.unwrap();  // 비동기 작업이 완료될 때까지 대기
    // }

    // 여러쓰레드에서 문제없이 작동하는지 확인해야 함.
    


    //The ability to handle each command.
    // teloxide::repl(bot, move |message: Message, bot: Bot| {

    //     async move {
    //         match handle_command(message, bot).await {
    //             Ok(_) => (),
    //             Err(e) => {
    //                 errork(e).await;
    //             }
    //         };
    //         respond(())
    //     }
    
    // })
    // .await;    
}

/*
    Functions that handle each command
*/
// async fn handle_command(message: Message, bot: Bot) -> Result<(), anyhow::Error> {

//     let command_service = CommandService::new(bot, message)?;    
//     let input_text = command_service.input_text;
    
//     if input_text.starts_with("c ") {
//         command_service.command_consumption().await?;
//     }
//     else if input_text.starts_with("cm") {
//         command_service.command_consumption_per_mon().await?;
//     }
//     else if input_text.starts_with("ctr") {
//         command_service.command_consumption_per_term().await?;
//     }
//     else if input_text.starts_with("ct") {
//         command_service.command_consumption_per_day().await?;
//     }
//     else if input_text.starts_with("cs") {
//         command_service.command_consumption_per_salary().await?;
//     }
//     else if input_text.starts_with("cw") {
//         command_service.command_consumption_per_week().await?;
//     }
//     else if input_text.starts_with("mc") {
//         command_service.command_record_fasting_time().await?;
//     }
//     else if input_text.starts_with("mt") {
//         command_service.command_check_fasting_time().await?;
//     }
//     else if input_text.starts_with("md") {
//         command_service.command_delete_fasting_time().await?;
//     }
//     else if input_text.starts_with("cy") {
//         command_service.command_consumption_per_year().await?;
//     }
//     else if input_text.starts_with("list") {
//         command_service.command_get_consume_type_list().await?;
//     }
//     else 
//     {
//         command_service.command_consumption_auto().await?;
//     }
    
//     Ok(())
// }