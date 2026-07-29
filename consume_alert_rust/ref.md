command_consumption_auto
-> modify_by_consume_filter


modify_samsung_card
-> to_string_vector_by_replace


command_consumption_auto
-> modify_by_consume_filter



modify_samsung_card
-> to_string_vector_by_replacecommand_consumption_auto
-> modify_by_consume_filter


modify_samsung_card
-> to_string_vector_by_replacecommand_consumption_auto
-> modify_by_consume_filter


modify_samsung_card
-> to_string_vector_by_replacecommand_consumption_auto
-> modify_by_consume_filter


modify_samsung_card
-> to_string_vector_by_replacecommand_consumption_auto
-> modify_by_consume_filter


modify_samsung_card
-> to_string_vector_by_replacecommand_consumption_auto
-> modify_by_consume_filter


modify_samsung_card
-> to_string_vector_by_replacecommand_consumption_auto
-> modify_by_consume_filter


modify_samsung_card
-> to_string_vector_by_replacecommand_consumption_auto
-> modify_by_consume_filter


modify_samsung_card
-> to_string_vector_by_replacecommand_consumption_auto
-> modify_by_consume_filter


modify_samsung_card
-> to_string_vector_by_replacecommand_consumption_auto
-> modify_by_consume_filter


modify_samsung_card
-> to_string_vector_by_replacecommand_consumption_auto
-> modify_by_consume_filter


modify_samsung_card
-> to_string_vector_by_replace



let cur_python_graph_info: ToPythonGraphLine = ToPythonGraphLine::new(
            "cur",
            permon_datetime.date_start,
            permon_datetime.date_end,
            &spent_detail_info_kst,
        )?;


find_python_matplot_asset_history



let cur_python_graph_info: ToPythonGraphLine = ToPythonGraphLine::new(
    "cur",
    permon_datetime.date_start,
    permon_datetime.date_end,
    *spent_detail_info_kst.agg_result(),
    spent_detail_info_kst.source_list(),
    LineAggregation::Cumulative,
)?;



let consume_detail_graph: Vec<u8> = self
    .graph_api_service
    .find_python_matplot_consume_detail_double(
        &cur_python_graph_info,
        &versus_python_graph_info,
    )
    .await?;

let consume_detail_graph_img: FileInfo =
    FileInfo::new(String::from("consume_detail"), consume_detail_graph);

let img_files: Vec<FileInfo> = vec![consume_detail_graph_img, circle_img];

self.tele_bot_service.input_photo_confirm(img_files).await?;


daily_python_graph_info
weekly_python_graph_info
monthly_python_graph_info
quarterly_python_graph_info
half_python_graph_info
yearly_python_graph_info