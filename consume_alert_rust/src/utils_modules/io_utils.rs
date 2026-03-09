use crate::common::*;

#[doc = "Function that takes a particular value from a vector - Access by Index"]
/// # Arguments
/// * `vec`     - Vector data
/// * `index`   - Parameters for which number of elements to be drawn from the vector
///
/// # Returns
/// * Result<T, anyhow::Error>
pub fn get_parsed_value_from_vector<T: FromStr>(
    vec: &[String],
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

#[doc = "Function that deletes files from the given path list"]
/// # Arguments
/// * `path_vec` - Vector of file paths to delete
///
/// # Returns
/// * Result<(), anyhow::Error>
pub fn delete_file(path_vec: Vec<String>) -> Result<(), anyhow::Error> {
    for path in path_vec {
        if std::path::Path::new(&path).exists() {
            std::fs::remove_file(&path).map_err(|e| {
                anyhow!(
                    "[Error][delete_file()] Failed to delete file '{}': {:?}",
                    path,
                    e
                )
            })?;
        }
    }
    Ok(())
}
