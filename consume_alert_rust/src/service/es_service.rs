use crate::common::*;

use crate::repository::es_repository::*;

#[derive(Debug, Getters, Clone)]
#[getset(get = "pub")]
pub struct EsHelper {
    mon_es_pool: Vec<EsObj>
}

impl EsHelper {
    
    /* 
        Constructor
    */
    pub fn new(es_url_vec: Vec<String>, es_id: &str, es_pw: &str) -> Result<Self, anyhow::Error> {
        
        let mut mon_es_clients: Vec<EsObj> = Vec::new();
        
        for url in es_url_vec {
    
            let parse_url = format!("http://{}:{}@{}", es_id, es_pw, url);
            
            let es_url = Url::parse(&parse_url)?;
            let conn_pool = SingleNodeConnectionPool::new(es_url);
            let transport = TransportBuilder::new(conn_pool)
                .timeout(Duration::new(5,0))
                .build()?;
            
            mon_es_clients.push(EsObj::new(url, Elasticsearch::new(transport)));
        }
        
        Ok(EsHelper{mon_es_pool: mon_es_clients})
    }
    
    /*
        Functions that handle queries at the Elasticsearch Cluster LEVEL - SEARCH
    */
    pub async fn cluster_search_query(&self, es_query: Value, index_name: &str) -> Result<Value, anyhow::Error> {

        for es_obj in self.mon_es_pool.iter() {

            match es_obj.get_search_query(&es_query, index_name).await {
                Ok(resp) => return Ok(resp),
                Err(err) => {
                    error!("{:?}", err);      
                    continue;
                }
            }   
        }
        
        Err(anyhow!("[Elasticsearch Error][cluster_search_query()] All Elasticsearch connections failed"))
    }

    /*
        Functions that handle queries at the Elasticsearch Cluster LEVEL - INDEXING
    */
    pub async fn cluster_post_query(&self, document: Value, index_name: &str) -> Result<(), anyhow::Error> {

        for es_obj in self.mon_es_pool.iter() {

            match es_obj.post_query(&document, index_name).await {
                Ok(resp) => return Ok(resp),
                Err(err) => {
                    error!("{:?}", err);      
                    continue;
                }
            }   
        }
        
        Err(anyhow!("[Elasticsearch Error][cluster_post_query()] All Elasticsearch connections failed"))

    }


    /*
        Functions that handle queries at the Elasticsearch Cluster LEVEL - DELETE
    */
    pub async fn cluster_delete_query(&self, doc_id: &str, index_name: &str) -> Result<(), anyhow::Error> {

        for es_obj in self.mon_es_pool.iter() {

            match es_obj.delete_query(doc_id, index_name).await {
                Ok(resp) => return Ok(resp),
                Err(err) => {
                    error!("{:?}", err);      
                    continue;
                }
            }   
        }
        
        Err(anyhow!("[Elasticsearch Error][cluster_delete_query()] All Elasticsearch connections failed"))                
    }
    
}
