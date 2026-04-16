pub fn get_add_month_from_naivedate_old(
    dt: DateTime<Utc>,
    add_month: i32,
) -> Result<DateTime<Utc>, anyhow::Error> {
    let naive_date = dt.date_naive();
    let mut new_year: i32 = naive_date.year() + (naive_date.month() as i32 + add_month - 1) / 12;
    let mut new_month: i32 = (naive_date.month() as i32 + add_month - 1) % 12 + 1;

    /* Adjust if the month is out of range */
    if new_month <= 0 {
        new_month += 12;
        new_year -= 1;
    }

    /*
        Handling Date Data Exception.
        ex) 2024-11-31 -> Not exists
    */
    let mut input_day: u32 = naive_date.day();

    let new_date: NaiveDate = NaiveDate::from_ymd_opt(new_year, new_month as u32, 1)
        .ok_or_else(|| anyhow!("[Datetime Parsing Error][get_add_month_from_naivedate()] Invalid date => year: {}, month: {}, day: 1", new_year, new_month))?;
    let next_month = if new_date.month() == 12 {
        NaiveDate::from_ymd_opt(new_date.year() + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(new_date.year(), new_date.month() + 1, 1)
    };

    let last_day: u32 = next_month
        .ok_or_else(|| anyhow!("[Error][get_add_month_from_naivedate()] Problem with variable 'last_day'"))?
        .pred_opt()
        .ok_or_else(|| anyhow!("[Error][get_add_month_from_naivedate()] Problem while converting variable 'last_day'"))?
        .day();

    if input_day > last_day {
        input_day = last_day;
    }

    NaiveDate::from_ymd_opt(new_year, new_month as u32, input_day)
        .ok_or_else(|| anyhow!("[Datetime Parsing Error][get_add_month_from_naivedate()] Invalid date. => new_year: {:?}, new_month: {:?}, day: {:?}",
            new_year,
            new_month,
            input_day))
        .map(|d| d.and_time(NaiveTime::MIN).and_utc())
}