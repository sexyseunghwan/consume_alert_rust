use crate::common::*;

#[doc = "Function to convert structure to JSON value"]
/// # Arguments
/// * input_struct -
///
/// # Returns
/// * Result<Value, anyhow::Error>
pub fn convert_json_from_struct<T: Serialize>(input_struct: &T) -> Result<Value, anyhow::Error> {
    serde_json::to_value(input_struct).map_err(|err| {
        anyhow!(
            "[Error][convert_json_from_struct()] Failed to serialize struct to JSON: {}",
            err
        )
    })
}

#[doc = ""]
/// # Arguments
/// * value -
///
/// # Returns
/// * String
pub fn format_number(value: i64) -> String {
    value.to_formatted_string(&Locale::en)
}

#[doc = "Function that takes a particular value from a vector - Access by Index"]
/// # Arguments
/// * `vec`     - Vector data
/// * `index`   - Parameters for which number of elements to be drawn from the vector
///
/// # Returns
/// * Result<T, anyhow::Error>
pub fn get_parsed_value_from_vector<T: FromStr>(
    vec: &Vec<String>,
    index: usize,
) -> Result<T, anyhow::Error>
where
    T::Err: std::fmt::Debug,
{
    let value_str = vec.get(index).ok_or_else(|| {
        anyhow!(
            "[Index Out Of Range Error] The {}th element does not exist.",
            index
        )
    })?;

    value_str.parse::<T>().map_err(|e| {
        anyhow!(
            "[Parse Error] Failed to parse the value at index {}: {:?}",
            index,
            e
        )
    })
}

#[doc = "Function to delete files"]
/// # Arguments
/// * `path_vec` - Image path vector to delete
///
/// # Returns
/// * Result<(), anyhow::Error>
pub fn delete_file(path_vec: Vec<String>) -> Result<(), anyhow::Error> {
    for dir_name in path_vec {
        fs::remove_file(Path::new(&dir_name))?;
    }

    Ok(())
}
