use crate::common::*;

use crate::repository::es_repository::*;

/*
    Function to use elasticsearch connection in a single tone
*/
pub fn get_elastic_conn() -> Result<Arc<EsRepositoryPub>, anyhow::Error> {

    let es_client = ELASTICSEARCH_CLIENT
        .get()
        .ok_or_else(|| anyhow!("[DB Connection Error][command_consumption()] Cannot connect Elasticsearch"))?;
            
    Ok(Arc::clone(es_client))
}