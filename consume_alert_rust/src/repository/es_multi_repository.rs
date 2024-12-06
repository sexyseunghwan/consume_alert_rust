use crate::common::*;


#[doc = "Elasticsearch connection object to be used in a single tone"]
static ELASTICSEARCH_MULTI_CLIENT: once_lazy<Arc<EsMultiRepositoryPub>> = once_lazy::new(|| {
    Arc::new(initialize_elastic_multi_clients())
});


#[doc = "Function to initialize Elasticsearch connection instances"]
pub fn initialize_elastic_multi_clients() -> EsMultiRepositoryPub {

    let es_host: Vec<String> = env::var("ES_DB_URL")
        .expect("[ENV file read Error][initialize_db_clients()] 'ES_DB_URL' must be set")
        .split(',')
        .map(|s| s.to_string())
        .collect();
    
    let es_id = env::var("ES_ID").expect("[ENV file read Error][initialize_db_clients()] 'ES_ID' must be set");
    let es_pw = env::var("ES_PW").expect("[ENV file read Error][initialize_db_clients()] 'ES_PW' must be set");
    
    // let mut url_vec: Vec<Url> = Vec::new();
    
    // for host in &es_host {

    //     let url = match Url::parse( format!("http://{}:{}@{}", es_id, es_pw, host).as_str()) {
    //         Ok(url) => {
    //             info!("Configuring connection for URL: {}", url);
    //             url
    //         },
    //         Err(e) => {
    //             error!("[Error][initialize_elastic_multi_clients()] {:?}", e);
    //             panic!("{:?}", e);
    //         }
    //     };

    //     url_vec.push(url);
    // }
    
    let es_urls = vec![
        Url::parse("http://elastic:156452@221.149.34.65:2025").unwrap(),
        Url::parse("http://elastic:156452@221.149.34.65:2026").unwrap(),
        Url::parse("http://elastic:156452@221.149.34.65:2027").unwrap(),
    ];


    let connection_pool = MultiNodeConnectionPool::round_robin(es_urls, Some(Duration::from_secs(60)));
    
    let transport = match TransportBuilder::new(connection_pool)
        .timeout(Duration::new(5,0))
        .build() {
            Ok(transport) => transport,
            Err(e) => {
                error!("[Error][initialize_elastic_multi_clients()] {:?}", e);
                panic!("{:?}", e);
            }
        };
    
    let elastic_conn = Elasticsearch::new(transport);
    
    EsMultiRepositoryPub::new(elastic_conn)

}


#[doc = "Function to get elasticsearch connection"]
pub fn get_elastic_conn() -> Arc<EsMultiRepositoryPub> {
    let es_client = &ELASTICSEARCH_MULTI_CLIENT;
    Arc::clone(&es_client)
}


#[async_trait]
pub trait EsMultiRepository {
    async fn get_search_query(&self, es_query: &Value, index_name: &str) -> Result<Value, anyhow::Error>;
    async fn post_query(&self, document: &Value, index_name: &str) -> Result<(), anyhow::Error>;
    async fn delete_query(&self, doc_id: &str, index_name: &str) -> Result<(), anyhow::Error>;
}


#[derive(Debug, Getters, Clone, new)]
pub struct EsMultiRepositoryPub {
    es_conn: Elasticsearch
}


#[async_trait]
impl EsMultiRepository for EsMultiRepositoryPub {


    #[doc = "Function that EXECUTES elasticsearch queries - search"]
    async fn get_search_query(&self, es_query: &Value, index_name: &str) -> Result<Value, anyhow::Error> {
        
        println!("self.es_conn: {:?}", self.es_conn);

        let response = self
            .es_conn
            .search(SearchParts::Index(&[index_name]))
            .body(es_query)
            .send()
            .await?;

        //println!("response= {:?}", response);

        if response.status_code().is_success() { 
            let response_body = response.json::<Value>().await?;
            Ok(response_body)
        } else {
            let error_body = response.text().await?;
            Err(anyhow!("[Elasticsearch Error][node_search_query()] response status is failed: {:?}", error_body))
        }
    }
    

    #[doc = "Function that EXECUTES elasticsearch queries - indexing"]
    async fn post_query(&self, document: &Value, index_name: &str) -> Result<(), anyhow::Error> {
        
        let response = self
            .es_conn
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

    #[doc = "Function that EXECUTES elasticsearch queries - delete"]
    async fn delete_query(&self, doc_id: &str, index_name: &str) -> Result<(), anyhow::Error> {
        
        let response = self
            .es_conn
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