common_process_python_double
-> find_info_filter_roomseq_orderby_aggs_range ()



query: {"aggs":{"aggs_result":{"sum":{"field":"spent_money"}}},"query":{"bool":{"filter":[{"range":{"spent_at":{"gte":"2026-06-30","lte":"2026-07-31"}}},{"term":{"agg_group_seq":1}}]}},"size":10000,"sort":{"spent_at":{"order":"asc"}}}
