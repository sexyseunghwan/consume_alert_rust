use teloxide::dispatching::dialogue::GetChatId;

use crate::common::*;

use crate::service::calculate_service::*;
use crate::service::es_service::*;
use crate::service::tele_bot_service::*;

use crate::utils_modules::numeric_utils::*;
use crate::utils_modules::time_utils::*;

use crate::dtos::dto::*;



/*
    command handler: Writes the expenditure details to the index in ElasticSearch. -> /c
*/
pub async fn command_consumption(message: &Message, text: &str, bot: &Bot, es_client: &Arc<EsHelper>) -> Result<(), anyhow::Error> {

    let args = &text[3..];
    let split_args_vec: Vec<String> = args.split(":").map(|s| s.to_string()).collect();
    let mut consume_name = "";
    let mut consume_cash = "";

    if split_args_vec.len() != 2 {
        
        send_message_confirm(bot, message.chat.id, true, "There is a problem with the parameter you entered. Please check again. \nEX) /c snack:15000","").await?;
        return Err(anyhow!(format!("ERROR in 'command_consumption()' function - input_data : {}", text)));
    } 

    if let Some(cons_name) = split_args_vec.get(0) {

        if let Some(price) = split_args_vec.get(1) {
            
            if !is_numeric(&price) {
                send_message_confirm(bot, message.chat.id, true, "The second parameter must be numeric. \nEX) /c snack:15000", "").await?;
                return Err(anyhow!(format!("ERROR in 'command_consumption()' function - input_data : {}", text)));
            }

            consume_name = cons_name;
            consume_cash = price;
        }        

    } else {
        
        send_message_confirm(bot, message.chat.id, true, "There is a problem with the parameter you entered. Please check again. \nEX) /c snack:15000", "").await?;
        return Err(anyhow!("ERROR in 'command_consumption()' function - input_data : {}", text));
    }
    
    let curr_time = get_current_korean_time_str("%Y-%m-%dT%H:%M:%S%.3fZ");
    
    let document = json!({
        "@timestamp": curr_time,
        "prodt_name": consume_name,
        "prodt_money": convert_numeric(consume_cash)
    });

    es_client.cluster_post_query(document, "consuming_index_prod_new").await?;
    
    Ok(())
}


/*
    command handler: Checks how much you have consumed during a month -> /cm
*/
pub async fn command_consumption_per_mon(message: &Message, text: &str, bot: &Bot, es_client: &Arc<EsHelper>) -> Result<(), anyhow::Error> {

    let args = &text[3..];
    let split_args_vec: Vec<String> = args.split(" ").map(String::from).collect();

    let (cur_date_start, cur_date_end, one_mon_ago_date_start, one_mon_ago_date_end) = match split_args_vec.len() {
        1 => {
            let start = get_current_korean_time_str("%Y-%m-01");
            let end = get_last_date_str(&start, "%Y-%m-%d")?;
            let one_month_ago_start = get_one_month_ago_kr_str(&start, "%Y-%m-01")?;
            let one_month_ago_end = get_last_date_str(&one_month_ago_start, "%Y-%m-%d")?;
            (start, end, one_month_ago_start, one_month_ago_end)
        },
        2 if split_args_vec.get(1).map_or(false, |d| validate_date_format(d, r"^\d{4}\.\d{2}$").unwrap_or(false)) => {
            let start = format!("{}-01", split_args_vec[1]);
            let end = get_last_date_str(&start, "%Y-%m-%d")?;
            let one_month_ago_start = get_one_month_ago_kr_str(&start, "%Y-%m-%d")?;
            let one_month_ago_end = get_last_date_str(&one_month_ago_start, "%Y-%m-%d")?;
            (start, end, one_month_ago_start, one_month_ago_end)
        },
        _ => {
            send_message_confirm(bot, message.chat.id, true, "Invalid date format. Please use format YYYY.MM like /cm 2023.07", "").await?;
            return Err(anyhow!("Invalid input: {}", text));
        }
    };
    
    let consume_type_vec = get_classification_consumption_type(es_client, "consuming_index_prod_type").await?;
    let cur_mon_total_cost_infos = total_cost_detail_specific_period(&cur_date_start, &cur_date_end, es_client, "consuming_index_prod_new", &consume_type_vec).await?;
    let pre_mon_total_cost_infos = total_cost_detail_specific_period(&one_mon_ago_date_start, &one_mon_ago_date_end, es_client, "consuming_index_prod_new", &consume_type_vec).await?;
    
    // Hand over the consumption details to Telegram bot.
    send_message_consume_split(bot, message.chat.id, &cur_mon_total_cost_infos.1, *(&cur_mon_total_cost_infos.0), &cur_date_start, &cur_date_end).await?;  

    println!("1");

    // ( consumption type information, consumption type graph storage path )
    let comsume_type_infos = get_consume_type_graph(*(&cur_mon_total_cost_infos.0), &cur_date_start, &cur_date_end, &cur_mon_total_cost_infos.1).await?;
    println!("{:?} // {:?} // {:?} // {:?}", &cur_date_start, &cur_date_end, cur_mon_total_cost_infos.0, cur_mon_total_cost_infos.1);
    let python_graph_line_info_cur = ToPythonGraphLine::new("cur", &cur_date_start, &cur_date_end, cur_mon_total_cost_infos.0, cur_mon_total_cost_infos.1)?;
    println!("2");
    let python_graph_line_info_pre = ToPythonGraphLine::new("pre", &one_mon_ago_date_start, &one_mon_ago_date_end, pre_mon_total_cost_infos.0, pre_mon_total_cost_infos.1)?;
    println!("3");
    let graph_path = get_consume_detail_graph_double(python_graph_line_info_cur, python_graph_line_info_pre).await?;
    println!("4");


    println!("{:?}", comsume_type_infos.1);
    println!("{:?}", graph_path);

    //send_photo_confirm(bot, message.chat.id, &graph_path).await?;
    //send_photo_confirm(bot, message.chat.id, &comsume_type_infos.1).await?;
    
    //send_message_consume_type(bot, message.chat.id, &comsume_type_infos.0, *(&cur_mon_total_cost_infos.0), &cur_date_start, &cur_date_end).await?;  

    
    Ok(())

}



/*
    command handler: Checks how much you have consumed during a month -> /cm
*/
// pub async fn command_consumption_per_mon(message: &Message, text: &str, bot: &Bot, es_client: &Arc<EsHelper>) -> Result<(), anyhow::Error> {

//     let args = &text[3..];
//     let split_args_vec: Vec<String> = args.split(" ").map(|s| s.to_string()).collect();

//     let mut cur_date_start = String::from("");   
//     let mut cur_date_end = String::from("");  
//     let mut one_mon_ago_date_start = String::from("");    
//     let mut one_mon_ago_date_end = String::from("");    

//     // 1. Initialize current date, month prior date
//     if split_args_vec.len() == 1 {
        
//         // == [case1] /cm == 
//         cur_date_start = get_current_korean_time_str("%Y-%m-01");
//         cur_date_end = get_last_date_str(cur_date_start.as_str(), "%Y-%m-%d")?;
//         one_mon_ago_date_start = get_one_month_ago_kr_str(cur_date_start.as_str(), "%Y-%m-01")?;
//         one_mon_ago_date_end = get_last_date_str(one_mon_ago_date_start.as_str(), "%Y-%m-%d")?;
        
//     } else if split_args_vec.len() == 2 {

//         // == [case2] /cm 2024.01 ==
//         if let Some(input_date) = split_args_vec.get(1) {

//             let input_date_own = input_date.to_string();

//             let date_format_yn = validate_date_format(input_date, r"^\d{4}\.\d{2}$")?;

//             if !date_format_yn {
//                 send_message_confirm(bot, message.chat.id, true, "There is a problem with the parameter you entered. Please check again. \nEX01) /cm 2023.07\nEX02) /cm","").await?;
//                 return Err(anyhow!(format!("ERROR in 'command_consumption_per_mon()' function - input_data : {}", text)));
//             }
            
//             cur_date_start = input_date_own + "-01";
//             cur_date_end = get_last_date_str(cur_date_start.as_str(), "%Y-%m-%d")?;
//             one_mon_ago_date_start = get_one_month_ago_kr_str(cur_date_start.as_str(), "%Y-%m-%d")?;
//             one_mon_ago_date_end = get_last_date_str(one_mon_ago_date_start.as_str(), "%Y-%m-%d")?;

//         } else {
//             return Err(anyhow!(format!("ERROR in 'command_consumption_per_mon()' function - input_data : {}", text)));
//         }
        
//     } else {
//         send_message_confirm(bot, message.chat.id, true, "There is a problem with the parameter you entered. Please check again. \nEX01) /cm 2023.07.01\nEX02) /cm", "").await?;
//         return Err(anyhow!(format!("The input parameter value of the 'command_consumption_per_mon()' function does not satisfy the specified date format. - input_val : {}", text)));
//     }

    
//     // 2. It calculates the total amount of consumption.
//     //let chat_id = message.chat.id;
//     let consume_type_vec: Vec<ProdtTypeInfo> = get_classification_consumption_type(es_client, "consuming_index_prod_type").await.unwrap();

//     let cur_mon_total_cost = total_cost_detail_specific_period(cur_date_start.as_str(), cur_date_end.as_str(), es_client, "consuming_index_prod_new", &consume_type_vec).await?;
//     let pre_mon_total_cost = total_cost_detail_specific_period(one_mon_ago_date_start.as_str(), one_mon_ago_date_end.as_str(), es_client, "consuming_index_prod_new", &consume_type_vec).await?;
    
//     send_message_consume_split(bot, message.chat.id, &cur_mon_total_cost.1, &cur_mon_total_cost.0, &cur_date_start, &cur_date_end).await?;
    
//     let comparison_info = ComparisonConsumeInfo::new(one_mon_ago_date_start, one_mon_ago_date_end, pre_mon_total_cost.0, pre_mon_total_cost.1, (&cur_date_start).to_string(), (&cur_date_end).to_string(), *(&cur_mon_total_cost.0), cur_mon_total_cost.clone().1);
//     let comsume_type_infos = get_consume_type_graph(*(&cur_mon_total_cost.0), &cur_date_start, &cur_date_end, &cur_mon_total_cost.1).await?;
            


//     Ok(())
// }



