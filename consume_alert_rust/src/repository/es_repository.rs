use crate::common::*;


#[async_trait]
pub trait EsRepository {
    async fn get_search_query(&self, es_query: &Value, index_name: &str) -> Result<Value, anyhow::Error>;
    async fn post_query(&self, document: &Value, index_name: &str) -> Result<(), anyhow::Error>;
    async fn delete_query(&self, doc_id: &str, index_name: &str) -> Result<(), anyhow::Error>;
}


#[derive(Debug, Getters, Clone, new)]
#[getset(get = "pub")]
pub struct EsObj {
    pub es_host: String,
    pub es_pool: Elasticsearch
}


#[async_trait]
impl EsRepository for EsObj {
    
    /*
        Function that EXECUTES elasticsearch queries - search
    */
    async fn get_search_query(&self, es_query: &Value, index_name: &str) -> Result<Value, anyhow::Error> {

        // Response Of ES-Query
        let response = self.es_pool
            .search(SearchParts::Index(&[index_name]))
            .body(es_query)
            .send()
            .await?;

        if response.status_code().is_success() { 
            let response_body = response.json::<Value>().await?;
            Ok(response_body)
        } else {
            let error_body = response.text().await?;
            Err(anyhow!("[Elasticsearch Error][node_search_query()] response status is failed: {:?}", error_body))
        }
    }

    /*
        Function that EXECUTES elasticsearch queries - indexing
    */
    async fn post_query(&self, document: &Value, index_name: &str) -> Result<(), anyhow::Error> {

        let response = self.es_pool
            .index(IndexParts::Index(index_name))
            .body(document)
            .send()
            .await?;
        
        if response.status_code().is_success() {
            Ok(())
        } else {
            let error_message = format!("[Elasticsearch Error][node_post_query()] Failed to index document: Status Code: {}", response.status_code());
            Err(anyhow!(error_message))
        }
    }
    
    
    /*
        Function that EXECUTES elasticsearch queries - delete
    */
    async fn delete_query(&self, doc_id: &str, index_name: &str) -> Result<(), anyhow::Error> {

        let response = self.es_pool
            .delete(DeleteParts::IndexId(index_name, doc_id))
            .send()
            .await?;
        
        
        if response.status_code().is_success() {
            Ok(())
        } else {
            let error_message = format!("[Elasticsearch Error][node_delete_query()] Failed to delete document: Status Code: {}, Document ID: {}", response.status_code(), doc_id);
            Err(anyhow!(error_message))
        }
        
    }
}