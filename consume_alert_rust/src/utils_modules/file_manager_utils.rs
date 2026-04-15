use crate::common::*;

/// Deletes files at each path in the provided vector.
///
/// # Arguments
///
/// * `path_vec` - Vector of file path strings to delete
///
/// # Errors
///
/// Returns an error if any file removal fails.
pub fn delete_file(path_vec: Vec<String>) -> Result<(), anyhow::Error> {
    for dir_name in path_vec {
        fs::remove_file(Path::new(&dir_name))?;
    }

    Ok(())
}
