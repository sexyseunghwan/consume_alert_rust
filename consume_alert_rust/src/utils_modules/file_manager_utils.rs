use crate::common::*;

/*
    Function to delete files
*/
pub fn delete_file(path_vec: Vec<String>) -> Result<(), anyhow::Error> {
    for dir_name in path_vec {
        fs::remove_file(Path::new(&dir_name))?;
    }

    Ok(())
}
