use teloxide::dispatching::dialogue::GetChatId;

use crate::common::*;

use crate::service::calculate_service::*;
use crate::service::es_service::*;
use crate::service::tele_bot_service::*;

use crate::utils_modules::numeric_utils::*;
use crate::utils_modules::time_utils::*;
use crate::utils_modules::file_manager_utils::*;

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
        
        send_message_confirm(bot, 
                            message.chat.id, 
                            true, 
                            "There is a problem with the parameter you entered. Please check again. \nEX) /c snack:15000").await?;

        return Err(anyhow!(format!("ERROR in 'command_consumption()' function - input_data : {}", text)));
    } 

    if let Some(cons_name) = split_args_vec.get(0) {

        if let Some(price) = split_args_vec.get(1) {
            
            if !is_numeric(&price) {
                send_message_confirm(bot, message.chat.id, true, "The second parameter must be numeric. \nEX) /c snack:15000").await?;
                return Err(anyhow!(format!("ERROR in 'command_consumption()' function - input_data : {}", text)));
            }

            consume_name = cons_name;
            consume_cash = price;
        }        

    } else {
        
        send_message_confirm(bot, message.chat.id, true, "There is a problem with the parameter you entered. Please check again. \nEX) /c snack:15000").await?;
        return Err(anyhow!("ERROR in 'command_consumption()' function - input_data : {}", text));
    }
    
    let curr_time = get_current_kor_naive_datetime();
    
    let document = json!({
        "@timestamp": get_str_from_naive_datetime(curr_time),
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
            let start = get_current_kor_naivedate_first_date()?;
            let end = get_lastday_naivedate(start)?;
            let one_month_ago_start = get_add_month_from_naivedate(start, -1)?;
            let one_month_ago_end = get_lastday_naivedate(one_month_ago_start)?;
            (start, end, one_month_ago_start, one_month_ago_end)
        },
        2 if split_args_vec.get(1).map_or(false, |d| validate_date_format(d, r"^\d{4}\.\d{2}$").unwrap_or(false)) => {
            
            let year: i32 = split_args_vec
                                .get(0)
                                .ok_or_else(|| anyhow!("Invalid date - command_consumption_per_mon(): There is a problem with the first element of the 'split_args_vec' vector."))?
                                .parse()?;
            
            let month: u32 = split_args_vec
                                .get(1)
                                .ok_or_else(|| anyhow!("Invalid date - command_consumption_per_mon(): There is a problem with the second element of the 'split_args_vec' vector."))?
                                .parse()?;
            
            let start = get_naivedate(year, month, 1)?;
            let end = get_lastday_naivedate(start)?;
            let one_month_ago_start = get_add_month_from_naivedate(start, -1)?;
            let one_month_ago_end = get_lastday_naivedate(one_month_ago_start)?;
            (start, end, one_month_ago_start, one_month_ago_end)
        },
        _ => {
            send_message_confirm(bot, message.chat.id, true, "Invalid date format. Please use format YYYY.MM like /cm 2023.07").await?;
            return Err(anyhow!("Invalid input: {}", text));
        }
    };
        
    
    let consume_type_vec: Vec<ProdtTypeInfo> = get_classification_consumption_type(es_client, "consuming_index_prod_type").await?;
    let cur_mon_total_cost_infos: (f64, Vec<ConsumeInfo>) = total_cost_detail_specific_period(cur_date_start, 
                                                                                             cur_date_end, 
                                                                                             es_client, 
                                                                                             "consuming_index_prod_new", 
                                                                                             &consume_type_vec).await?;
    
    
    let pre_mon_total_cost_infos: (f64, Vec<ConsumeInfo>) = total_cost_detail_specific_period(one_mon_ago_date_start, 
                                                                                             one_mon_ago_date_end, es_client, 
                                                                                             "consuming_index_prod_new", 
                                                                                             &consume_type_vec).await?;
    
    // Hand over the consumption details to Telegram bot.
    send_message_consume_split(bot, 
                        message.chat.id, 
                        &cur_mon_total_cost_infos.1, 
                        *(&cur_mon_total_cost_infos.0), 
                        cur_date_start, 
                        cur_date_end).await?;  
    
    // ( consumption type information, consumption type graph storage path )
    let comsume_type_infos = get_consume_type_graph(
                                                                *(&cur_mon_total_cost_infos.0), 
                                                                cur_date_start, 
                                                                cur_date_end, 
                                                                &cur_mon_total_cost_infos.1).await?;
    let consume_type_img = comsume_type_infos.1;
    
    let mut python_graph_line_info_cur = ToPythonGraphLine::new(
                                                                "cur", 
                                                                get_str_from_naivedate(cur_date_start).as_str(), 
                                                                get_str_from_naivedate(cur_date_end).as_str(), 
                                                                cur_mon_total_cost_infos.0, 
                                                                cur_mon_total_cost_infos.1)?;


    let mut python_graph_line_info_pre = ToPythonGraphLine::new(
                                                            "pre", 
                                                            get_str_from_naivedate(one_mon_ago_date_start).as_str(), 
                                                            get_str_from_naivedate(one_mon_ago_date_end).as_str(), 
                                                            pre_mon_total_cost_infos.0, 
                                                            pre_mon_total_cost_infos.1)?;
    
    let graph_path = get_consume_detail_graph_double(&mut python_graph_line_info_cur, &mut python_graph_line_info_pre).await?;
    

    send_photo_confirm(bot, message.chat.id, &graph_path).await?;
    send_photo_confirm(bot, message.chat.id, &consume_type_img).await?;
    
    send_message_consume_type(bot, 
                            message.chat.id, 
                            &comsume_type_infos.0, 
                            *(&cur_mon_total_cost_infos.0), 
                            cur_date_start, 
                            cur_date_end).await?;  
    
    
    let delete_target_vec: Vec<String> = vec![consume_type_img, graph_path];
    delete_file(delete_target_vec)?;

    Ok(())

}


/*
    command handler: Checks how much you have consumed during a specific periods -> /ctr
*/
pub async fn command_consumption_per_term(message: &Message, text: &str, bot: &Bot, es_client: &Arc<EsHelper>) -> Result<(), anyhow::Error> {

    let args = &text[3..];
    let split_args_vec: Vec<String> = args.split(" ").map(String::from).collect();
    
    let (date_start, date_end) = match split_args_vec.len() {
        
        2 if split_args_vec.get(1).map_or(false, |d| validate_date_format(d, r"^\d{4}\.\d{2}\.\d{2}-\d{4}\.\d{2}\.\d{2}$").unwrap_or(false)) => {

            let split_bar_vec: Vec<String> = split_args_vec
                                .get(1)
                                .ok_or_else(|| anyhow!("Invalid date - command_consumption_per_term(): There is a problem with the first element of the 'split_args_vec' vector."))?
                                .split("-")
                                .map(String::from)
                                .collect();
            
            let date_start: String = split_bar_vec
                                    .get(0)
                                    .ok_or_else(|| anyhow!("Invalid date - command_consumption_per_term(): There is a problem with 'date_start' variable."))?
                                    .parse()?;
            let date_start_form = get_naive_date_from_str(&date_start, "%Y.%m.%d")?;

            let date_end: String = split_bar_vec
                                .get(1)
                                .ok_or_else(|| anyhow!("Invalid date - command_consumption_per_term(): There is a problem with 'date_end' variable."))?
                                .parse()?;
            let date_end_form = get_naive_date_from_str(&date_end, "%Y.%m.%d")?;

            (date_start_form, date_end_form)
        },
        _ => {
            send_message_confirm(bot, message.chat.id, true, "There is a problem with the parameter you entered. Please check again. \nEX) /ctr 2023.07.07-2023.08.01").await?;
            return Err(anyhow!("Invalid input: {}", text));
        }
    };
    
    let consume_type_vec: Vec<ProdtTypeInfo> = get_classification_consumption_type(es_client, "consuming_index_prod_type").await?;
    
    let cur_mon_total_cost_infos: (f64, Vec<ConsumeInfo>) = total_cost_detail_specific_period(date_start, 
                                                    date_end, 
                                                    es_client, 
                                                    "consuming_index_prod_new", 
                                                    &consume_type_vec).await?;
    
    // Hand over the consumption details to Telegram bot.
    send_message_consume_split(bot, 
        message.chat.id, 
        &cur_mon_total_cost_infos.1, 
        *(&cur_mon_total_cost_infos.0), 
        date_start, 
        date_end).await?;  
    
    // ( consumption type information, consumption type graph storage path )
    let comsume_type_infos = get_consume_type_graph(
        *(&cur_mon_total_cost_infos.0), 
        date_start, 
        date_end, 
        &cur_mon_total_cost_infos.1).await?;
    
    let consume_type_img = comsume_type_infos.1;
    
    let python_graph_line_info = ToPythonGraphLine::new(
        "cur", 
        get_str_from_naivedate(date_start).as_str(), 
        get_str_from_naivedate(date_end).as_str(), 
        cur_mon_total_cost_infos.0, 
        cur_mon_total_cost_infos.1)?;
    
    let graph_path = get_consume_detail_graph_single(&python_graph_line_info).await?;
    
    send_photo_confirm(bot, message.chat.id, &graph_path).await?;
    send_photo_confirm(bot, message.chat.id, &consume_type_img).await?;

    send_message_consume_type(bot, 
        message.chat.id, 
        &comsume_type_infos.0, 
        *(&cur_mon_total_cost_infos.0), 
        date_start, 
        date_end).await?;  
    
    let delete_target_vec: Vec<String> = vec![consume_type_img, graph_path];
    delete_file(delete_target_vec)?;

    Ok(())

}



/*
    command handler: Checks how much you have consumed during a day -> /ct
*/
pub async fn command_consumption_per_day(message: &Message, text: &str, bot: &Bot, es_client: &Arc<EsHelper>) -> Result<(), anyhow::Error> {
    
    let args = &text[3..];
    let split_args_vec: Vec<String> = args.split(" ").map(String::from).collect();
    
    let (start_dt, end_dt) = match split_args_vec.len() {
        1 => {
            let start = get_current_kor_naivedate();
            let end = get_current_kor_naivedate();
            (start, end)
        },
        2 if split_args_vec.get(1).map_or(false, |d| validate_date_format(d, r"^\d{4}\.\d{2}\.\d{2}$").unwrap_or(false)) => {
            
            let year: i32 = split_args_vec
                                .get(0)
                                .ok_or_else(|| anyhow!("Invalid date - command_consumption_per_day(): There is a problem with 'year' variable."))?
                                .parse()?;
            
            let month: u32 = split_args_vec
                                .get(1)
                                .ok_or_else(|| anyhow!("Invalid date - command_consumption_per_day(): There is a problem with 'month' variable."))?
                                .parse()?;

            let day: u32 = split_args_vec
                .get(2)
                .ok_or_else(|| anyhow!("Invalid date - command_consumption_per_day(): There is a problem with 'day' variable."))?
                .parse()?;
            
            let start_dt = get_naivedate(year, month, day)?;
            let end_dt = get_naivedate(year, month, day)?;
            
            (start_dt, end_dt)
        },
        _ => {
            send_message_confirm(bot, message.chat.id, true, "There is a problem with the parameter you entered. Please check again. \nEX) /ct or /ct 2023.11.11").await?;
            return Err(anyhow!("Invalid input: {}", text));
        }
    };
    
    let consume_type_vec: Vec<ProdtTypeInfo> = get_classification_consumption_type(es_client, "consuming_index_prod_type").await?;
    let cur_mon_total_cost_infos: (f64, Vec<ConsumeInfo>) = total_cost_detail_specific_period(start_dt, 
                                                    end_dt, 
                                                    es_client, 
                                                    "consuming_index_prod_new", 
                                                    &consume_type_vec).await?;
    
    send_message_consume_split(bot, 
                        message.chat.id, 
                        &cur_mon_total_cost_infos.1, 
                        *(&cur_mon_total_cost_infos.0), 
                        start_dt, 
                        end_dt).await?;  
    
    // ( consumption type information, consumption type graph storage path )
    let comsume_type_infos = get_consume_type_graph(
                                                                *(&cur_mon_total_cost_infos.0), 
                                                                start_dt, 
                                                                end_dt, 
                                                                &cur_mon_total_cost_infos.1).await?;
    let consume_type_img = comsume_type_infos.1;

    send_photo_confirm(bot, message.chat.id, &consume_type_img).await?;
        
    send_message_consume_type(bot, 
                            message.chat.id, 
                            &comsume_type_infos.0, 
                            *(&cur_mon_total_cost_infos.0), 
                            start_dt, 
                            end_dt).await?;  
    
    
    let delete_target_vec: Vec<String> = vec![consume_type_img];
    delete_file(delete_target_vec)?;
    
    
    Ok(())
}


/*
    command handler: Check the consumption details from the date of payment to the next payment. -> /cs
*/
pub async fn command_consumption_per_salary(message: &Message, text: &str, bot: &Bot, es_client: &Arc<EsHelper>) -> Result<(), anyhow::Error> {

    let args = &text[3..];
    let split_args_vec: Vec<String> = args.split(" ").map(String::from).collect();
    
    let (cur_date_start, cur_date_end, one_mon_ago_date_start, one_mon_ago_date_end) = match split_args_vec.len() {
        
        1 => {
            let cur_day = get_current_kor_naivedate_first_date()?;
            let cur_year = cur_day.year();
            let cur_month = cur_day.month();
            let cur_date = cur_day.day();
            
            let cur_date_start  = if cur_date < 25 { 
                let date = get_naivedate(cur_year, cur_month, 25)?;
                get_add_month_from_naivedate(date, -1)?
            } else { 
                get_naivedate(cur_year, cur_month, 25)?
            };
            
            let cur_date_end  = if cur_date < 25 { 
                get_naivedate(cur_year, cur_month, 25)?
            } else { 
                let date = get_naivedate(cur_year, cur_month, 25)?;
                get_add_month_from_naivedate(date, 1)?
            };
            
            let one_mon_ago_date_start = get_add_month_from_naivedate(cur_date_start, -1)?;
            let one_mon_ago_date_end = get_add_month_from_naivedate(cur_date_end, -1)?;
            
            
            (cur_date_start, cur_date_end, one_mon_ago_date_start, one_mon_ago_date_end)
        },
        2 if split_args_vec.get(1).map_or(false, |d| validate_date_format(d, r"^\d{4}\.\d{2}$").unwrap_or(false)) => {
            
            let year: i32 = split_args_vec
                                .get(0)
                                .ok_or_else(|| anyhow!("Invalid date - command_consumption_per_salary(): There is a problem with 'year' variable."))?
                                .parse()?;
            
            let month: u32 = split_args_vec
                                .get(1)
                                .ok_or_else(|| anyhow!("Invalid date - command_consumption_per_salary(): There is a problem with 'month' variable."))?
                                .parse()?;
            
            let cur_date_end = get_naivedate(year, month, 25)?;
            let cur_date_start = get_add_month_from_naivedate(cur_date_end, -1)?;
            
            let one_mon_ago_date_start = get_add_month_from_naivedate(cur_date_start, -1)?;
            let one_mon_ago_date_end = get_add_month_from_naivedate(cur_date_end, -1)?;

            (cur_date_start, cur_date_end, one_mon_ago_date_start, one_mon_ago_date_end)
        },
        _ => {
            send_message_confirm(bot, message.chat.id, true, "There is a problem with the parameter you entered. Please check again. \nEX) /ct or /ct 2023.11.11").await?;
            return Err(anyhow!("Invalid input: {}", text));
        }
    };
    
    let consume_type_vec: Vec<ProdtTypeInfo> = get_classification_consumption_type(es_client, "consuming_index_prod_type").await?;
    let cur_mon_total_cost_infos: (f64, Vec<ConsumeInfo>) = total_cost_detail_specific_period(cur_date_start, 
                                                                                             cur_date_end, 
                                                                                             es_client, 
                                                                                             "consuming_index_prod_new", 
                                                                                             &consume_type_vec).await?;
    
    
    let pre_mon_total_cost_infos: (f64, Vec<ConsumeInfo>) = total_cost_detail_specific_period(one_mon_ago_date_start, 
                                                                                             one_mon_ago_date_end, es_client, 
                                                                                             "consuming_index_prod_new", 
                                                                                             &consume_type_vec).await?;
    
    // Hand over the consumption details to Telegram bot.
    send_message_consume_split(bot, 
                        message.chat.id, 
                        &cur_mon_total_cost_infos.1, 
                        *(&cur_mon_total_cost_infos.0), 
                        cur_date_start, 
                        cur_date_end).await?;  
    
    // ( consumption type information, consumption type graph storage path )
    let comsume_type_infos = get_consume_type_graph(
                                                                *(&cur_mon_total_cost_infos.0), 
                                                                cur_date_start, 
                                                                cur_date_end, 
                                                                &cur_mon_total_cost_infos.1).await?;
    let consume_type_img = comsume_type_infos.1;
    
    let mut python_graph_line_info_cur = ToPythonGraphLine::new(
                                                                "cur", 
                                                                get_str_from_naivedate(cur_date_start).as_str(), 
                                                                get_str_from_naivedate(cur_date_end).as_str(), 
                                                                cur_mon_total_cost_infos.0, 
                                                                cur_mon_total_cost_infos.1)?;


    let mut python_graph_line_info_pre = ToPythonGraphLine::new(
                                                            "pre", 
                                                            get_str_from_naivedate(one_mon_ago_date_start).as_str(), 
                                                            get_str_from_naivedate(one_mon_ago_date_end).as_str(), 
                                                            pre_mon_total_cost_infos.0, 
                                                            pre_mon_total_cost_infos.1)?;
    
    let graph_path = get_consume_detail_graph_double(&mut python_graph_line_info_cur, &mut python_graph_line_info_pre).await?;
    

    send_photo_confirm(bot, message.chat.id, &graph_path).await?;
    send_photo_confirm(bot, message.chat.id, &consume_type_img).await?;
    
    send_message_consume_type(bot, 
                            message.chat.id, 
                            &comsume_type_infos.0, 
                            *(&cur_mon_total_cost_infos.0), 
                            cur_date_start, 
                            cur_date_end).await?;  
    
    
    let delete_target_vec: Vec<String> = vec![consume_type_img, graph_path];
    delete_file(delete_target_vec)?;


    Ok(())
}



/*
    command handler: command handler: Checks how much you have consumed during a week -> /cw
*/
pub async fn command_consumption_per_week(message: &Message, text: &str, bot: &Bot, es_client: &Arc<EsHelper>) -> Result<(), anyhow::Error> {

    let args = &text[3..];
    let split_args_vec: Vec<String> = args.split(" ").map(String::from).collect();
    
    let (date_start, date_end, one_pre_week_start, one_pre_week_end) = match split_args_vec.len() {
        
        1 =>{

            let now = get_current_kor_naive_datetime();
            let today = now.date();

            let weekday = today.weekday();

            let days_until_monday = Weekday::Mon.num_days_from_monday() as i64 - weekday.num_days_from_monday() as i64;
            let monday = today + chrono::Duration::days(days_until_monday);

            let date_start = monday + chrono::Duration::days(0);
            let date_end = monday + chrono::Duration::days(6);  
            let one_pre_week_start = date_start - chrono::Duration::days(7);
            let one_pre_week_end = date_end - chrono::Duration::days(7);

            println!("date_start = {:?}", date_start);
            println!("date_end = {:?}", date_end);
            println!("one_pre_week_start = {:?}", one_pre_week_start);
            println!("one_pre_week_end = {:?}", one_pre_week_end);

            (date_start, date_end, one_pre_week_start, one_pre_week_end)
        },
        _ => {
            send_message_confirm(bot, message.chat.id, true, "There is a problem with the parameter you entered. Please check again. \nEX) /cw").await?;
            return Err(anyhow!("Invalid input: {}", text));
        }
    };

    
    let consume_type_vec: Vec<ProdtTypeInfo> = get_classification_consumption_type(es_client, "consuming_index_prod_type").await?;
    let cur_mon_total_cost_infos: (f64, Vec<ConsumeInfo>) = total_cost_detail_specific_period(date_start, 
                                                                                            date_end, 
                                                                                             es_client, 
                                                                                             "consuming_index_prod_new", 
                                                                                             &consume_type_vec).await?;
    
    
    let pre_mon_total_cost_infos: (f64, Vec<ConsumeInfo>) = total_cost_detail_specific_period(one_pre_week_start, 
                                                                                            one_pre_week_end, 
                                                                                             es_client, 
                                                                                             "consuming_index_prod_new", 
                                                                                             &consume_type_vec).await?;
    
    // Hand over the consumption details to Telegram bot.
    send_message_consume_split(bot, 
                        message.chat.id, 
                        &cur_mon_total_cost_infos.1, 
                        *(&cur_mon_total_cost_infos.0), 
                        date_start, 
                        date_end).await?;  
    
    // ( consumption type information, consumption type graph storage path )
    let comsume_type_infos = get_consume_type_graph(
                                                                *(&cur_mon_total_cost_infos.0), 
                                                                date_start, 
                                                                date_end, 
                                                                &cur_mon_total_cost_infos.1).await?;
    let consume_type_img = comsume_type_infos.1;
    
    let mut python_graph_line_info_cur = ToPythonGraphLine::new(
                                                                "cur", 
                                                                get_str_from_naivedate(date_start).as_str(), 
                                                                get_str_from_naivedate(date_end).as_str(), 
                                                                cur_mon_total_cost_infos.0, 
                                                                cur_mon_total_cost_infos.1)?;


    let mut python_graph_line_info_pre = ToPythonGraphLine::new(
                                                            "pre", 
                                                            get_str_from_naivedate(one_pre_week_start).as_str(), 
                                                            get_str_from_naivedate(one_pre_week_end).as_str(), 
                                                            pre_mon_total_cost_infos.0, 
                                                            pre_mon_total_cost_infos.1)?;
    
    let graph_path = get_consume_detail_graph_double(&mut python_graph_line_info_cur, &mut python_graph_line_info_pre).await?;
    

    send_photo_confirm(bot, message.chat.id, &graph_path).await?;
    send_photo_confirm(bot, message.chat.id, &consume_type_img).await?;
    
    send_message_consume_type(bot, 
                            message.chat.id, 
                            &comsume_type_infos.0, 
                            *(&cur_mon_total_cost_infos.0), 
                            date_start, 
                            date_end).await?;  
    
    
    let delete_target_vec: Vec<String> = vec![consume_type_img, graph_path];
    delete_file(delete_target_vec)?;

    
    Ok(())
}
