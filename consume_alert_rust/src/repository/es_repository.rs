use std::os::windows::io::AsRawHandle;

use crate::common::*;

static ELASTICSEARCH_CLIENT: once_lazy<Arc<EsRepositoryPub>> = once_lazy::new(|| {
    initialize_elastic_clients()
});


/*
    Function to initialize Elasticsearch connection instances
*/
pub fn initialize_elastic_clients() -> Arc<EsRepositoryPub> {

    let es_host: Vec<String> = env::var("ES_DB_URL").expect("[ENV file read Error][initialize_db_clients()] 'ES_DB_URL' must be set").split(',').map(|s| s.to_string()).collect();
    let es_id = env::var("ES_ID").expect("[ENV file read Error][initialize_db_clients()] 'ES_ID' must be set");
    let es_pw = env::var("ES_PW").expect("[ENV file read Error][initialize_db_clients()] 'ES_PW' must be set");

    // Elasticsearch connection
    let es_client: EsRepositoryPub = match EsRepositoryPub::new(es_host, &es_id, &es_pw) {
        Ok(es_client) => es_client,
        Err(err) => {
            error!("[DB Connection Error][initialize_db_clients()] Failed to create Elasticsearch client : {:?}", err);
            panic!("[DB Connection Error][initialize_db_clients()] Failed to create Elasticsearch client : {:?}", err);
        }
    };

    Arc::new(es_client)
}


/*
    Function to get elasticsearch connection
*/
pub fn get_elastic_conn() -> Arc<EsRepositoryPub> {

    let es_client = &ELASTICSEARCH_CLIENT;
    Arc::clone(&es_client)

}


#[async_trait]
pub trait EsRepository {
    async fn get_search_query(&self, es_query: &Value, index_name: &str) -> Result<Value, anyhow::Error>;
    async fn post_query(&self, document: &Value, index_name: &str) -> Result<(), anyhow::Error>;
    async fn delete_query(&self, doc_id: &str, index_name: &str) -> Result<(), anyhow::Error>;
}


#[derive(Debug, Getters, Clone)]
pub struct EsRepositoryPub {
    es_clients: Vec<EsClient>,
}

#[derive(Debug, Getters, Clone, new)]
pub(crate) struct EsClient {
    host: String,
    es_conn: Elasticsearch
}

impl EsRepositoryPub {
    
    pub fn new(es_url_vec: Vec<String>, es_id: &str, es_pw: &str) -> Result<Self, anyhow::Error> {
        
        let mut es_clients: Vec<EsClient> = Vec::new();

        for url in es_url_vec {
    
            let parse_url = format!("http://{}:{}@{}", es_id, es_pw, url);
            
            let es_url = Url::parse(&parse_url)?;
            let conn_pool = SingleNodeConnectionPool::new(es_url);
            let transport = TransportBuilder::new(conn_pool)
                .timeout(Duration::new(5,0))
                .build()?;
            
            let elastic_conn = Elasticsearch::new(transport);
            let es_client = EsClient::new(url, elastic_conn);
            
            es_clients.push(es_client);
        }
        
        Ok(EsRepositoryPub{es_clients})
    }


    // Common logic: common node failure handling and node selection
    async fn execute_on_any_node<F, Fut>(&self, operation: F) -> Result<Response, anyhow::Error>
    where
        F: Fn(EsClient) -> Fut + Send + Sync,
        Fut: Future<Output = Result<Response, anyhow::Error>> + Send,
    {
        let mut last_error = None;
        
        // StdRng를 사용하여 Send 트레잇 문제 해결
        let mut rng = StdRng::from_entropy(); // 랜덤 시드로 생성
        let mut shuffled_clients = self.es_clients.clone();
        shuffled_clients.shuffle(&mut rng);
        
        for es_client in shuffled_clients {
            match operation(es_client).await {
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
                .es_conn
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
                .es_conn
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
                .es_conn
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