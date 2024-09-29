use crate::common::*;


#[async_trait]
pub trait EsRepository {
    async fn get_search_query(&self, es_query: &Value, index_name: &str) -> Result<Value, anyhow::Error>;
    async fn post_query(&self, document: &Value, index_name: &str) -> Result<(), anyhow::Error>;
    async fn delete_query(&self, doc_id: &str, index_name: &str) -> Result<(), anyhow::Error>;
}


// #[derive(Debug, Getters, Clone, new)]
// #[getset(get = "pub")]
// pub struct EsObj {
//     pub es_host: String,
//     pub es_pool: Elasticsearch
// }

#[derive(Debug, Getters, Clone)]
pub struct EsRepositoryPub {
    es_clients: Vec<Elasticsearch>,
}

impl EsRepositoryPub {
    
    pub fn new(es_url_vec: Vec<String>, es_id: &str, es_pw: &str) -> Result<Self, anyhow::Error> {

        let mut es_clients: Vec<Elasticsearch> = Vec::new();

        for url in es_url_vec {
    
            let parse_url = format!("http://{}:{}@{}", es_id, es_pw, url);
            
            let es_url = Url::parse(&parse_url)?;
            let conn_pool = SingleNodeConnectionPool::new(es_url);
            let transport = TransportBuilder::new(conn_pool)
                .timeout(Duration::new(5,0))
                .build()?;
            
            let elastic_conn = Elasticsearch::new(transport);
            es_clients.push(elastic_conn);
            //mon_es_clients.push(EsObj::new(url, Elasticsearch::new(transport)));
        }
        
        Ok(EsRepositoryPub{es_clients})
    }


    // Common logic: common node failure handling and node selection
    async fn execute_on_any_node<F, Fut>(&self, operation: F) -> Result<Response, anyhow::Error>
    where
        F: Fn(&Elasticsearch) -> Fut + Send + Sync,
        Fut: Future<Output = Result<Response, anyhow::Error>> + Send,
    {
        let mut last_error = None;
        let mut rng = rand::thread_rng();
        let mut shuffled_clients: Vec<Elasticsearch> = self.es_clients.clone();
        shuffled_clients.shuffle(&mut rng);
        
        for es_client in shuffled_clients {
            match operation(&es_client).await {
                Ok(response) => return Ok(response),
                Err(err) => {
                    last_error = Some(err);
                }
            }
        }

        Err(anyhow::anyhow!(
            "All Elasticsearch nodes failed. Last error: {:?}",
            last_error
        ))
    }
}



#[async_trait]
impl EsRepository for EsRepositoryPub {

    /*
        Function that EXECUTES elasticsearch queries - search
    */
    async fn get_search_query(&self, es_query: &Value, index_name: &str) -> Result<Value, anyhow::Error> {
        
        let response = self.execute_on_any_node(|es_client| async move {
            
            let response = es_client
                .search(SearchParts::Index(&[index_name]))
                .body(es_query)
                .send()
                .await?;

            Ok(response)
        })
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

        let response = self.execute_on_any_node(|es_client| async move {
        
            let response = es_client
                .index(IndexParts::Index(index_name))
                .body(document)
                .send()
                .await?;

            Ok(response)
        })
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
        
        let response = self.execute_on_any_node(|es_client| async move {
    
            let response = es_client
                .delete(DeleteParts::IndexId(index_name, doc_id))
                .send()
                .await?;

            Ok(response)
        })
        .await?;

        if response.status_code().is_success() {
            Ok(())
        } else {
            let error_message = format!("[Elasticsearch Error][node_delete_query()] Failed to delete document: Status Code: {}, Document ID: {}", response.status_code(), doc_id);
            Err(anyhow!(error_message))
        }
        
    }



}



// #[async_trait]
// impl EsRepository for EsObj {
    
//     /*
//         Function that EXECUTES elasticsearch queries - search
//     */
//     async fn get_search_query(&self, es_query: &Value, index_name: &str) -> Result<Value, anyhow::Error> {

//         // Response Of ES-Query
//         let response = self.es_pool
//             .search(SearchParts::Index(&[index_name]))
//             .body(es_query)
//             .send()
//             .await?;

//         if response.status_code().is_success() { 
//             let response_body = response.json::<Value>().await?;
//             Ok(response_body)
//         } else {
//             let error_body = response.text().await?;
//             Err(anyhow!("[Elasticsearch Error][node_search_query()] response status is failed: {:?}", error_body))
//         }
//     }

//     /*
//         Function that EXECUTES elasticsearch queries - indexing
//     */
//     async fn post_query(&self, document: &Value, index_name: &str) -> Result<(), anyhow::Error> {

//         let response = self.es_pool
//             .index(IndexParts::Index(index_name))
//             .body(document)
//             .send()
//             .await?;
        
//         if response.status_code().is_success() {
//             Ok(())
//         } else {
//             let error_message = format!("[Elasticsearch Error][node_post_query()] Failed to index document: Status Code: {}", response.status_code());
//             Err(anyhow!(error_message))
//         }
//     }
    
    
//     /*
//         Function that EXECUTES elasticsearch queries - delete
//     */
//     async fn delete_query(&self, doc_id: &str, index_name: &str) -> Result<(), anyhow::Error> {

//         let response = self.es_pool
//             .delete(DeleteParts::IndexId(index_name, doc_id))
//             .send()
//             .await?;
        
        
//         if response.status_code().is_success() {
//             Ok(())
//         } else {
//             let error_message = format!("[Elasticsearch Error][node_delete_query()] Failed to delete document: Status Code: {}, Document ID: {}", response.status_code(), doc_id);
//             Err(anyhow!(error_message))
//         }
        
//     }
// }