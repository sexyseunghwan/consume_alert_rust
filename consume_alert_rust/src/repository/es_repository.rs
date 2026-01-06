use crate::common::*;

#[async_trait]
pub trait EsRepository {
    async fn get_search_query(
        &self,
        es_query: &Value,
        index_name: &str,
    ) -> Result<Value, anyhow::Error>;
    async fn post_query(&self, document: &Value, index_name: &str) -> Result<(), anyhow::Error>;
    async fn delete_query(&self, doc_id: &str, index_name: &str) -> Result<(), anyhow::Error>;
}

#[derive(Debug, Getters, Clone)]
pub struct EsRepositoryPub {
    es_client: Elasticsearch,
}

impl EsRepositoryPub {
    pub fn new() -> anyhow::Result<Self> {
        let es_host: Vec<String> = env::var("ES_DB_URL")
            .expect("[EsRepositoryPub::new] 'ES_DB_URL' must be set")
            .split(',')
            .map(|s| s.to_string())
            .collect();

        let es_id = env::var("ES_ID").expect("[EsRepositoryPub::new] 'ES_ID' must be set");
        let es_pw = env::var("ES_PW").expect("[EsRepositoryPub::new] 'ES_PW' must be set");

        let cluster_urls: Vec<Url> = es_host
            .iter()
            .map(|host| Url::parse(&format!("http://{}", host)))
            .collect::<Result<_, _>>()
            .map_err(|e| anyhow!("[EsRepositoryPub::new] {:?}", e))?;

        let conn_pool: MultiNodeConnectionPool =
            MultiNodeConnectionPool::round_robin(cluster_urls, None);

        let mut builder: TransportBuilder =
            TransportBuilder::new(conn_pool).timeout(Duration::from_secs(30));

        if es_id != "" && es_pw != "" {
            builder = builder.auth(EsCredentials::Basic(es_id.to_string(), es_pw.to_string()));
        }

        let transport: Transport = builder
            .build()
            .map_err(|e| anyhow!("[EsRepositoryPub::new] {:?}", e))?;

        let es_client: Elasticsearch = Elasticsearch::new(transport);

        Ok(EsRepositoryPub { es_client })
    }
}

#[async_trait]
impl EsRepository for EsRepositoryPub {
    #[doc = "Function that EXECUTES elasticsearch queries - search"]
    async fn get_search_query(&self, es_query: &Value, index_name: &str) -> anyhow::Result<Value> {
        let response: Response = self
            .es_client
            .search(SearchParts::Index(&[index_name]))
            .body(es_query)
            .send()
            .await?;

        if response.status_code().is_success() {
            let response_body: Value = response.json::<Value>().await?;
            Ok(response_body)
        } else {
            let error_body: String = response.text().await?;
            Err(anyhow!(
                "[EsRepositoryPub::node_search_query] response status is failed: {:?}",
                error_body
            ))
        }
    }

    #[doc = "Function that EXECUTES elasticsearch queries - indexing"]
    async fn post_query(&self, document: &Value, index_name: &str) -> anyhow::Result<()> {
        let response: Response = self
            .es_client
            .index(IndexParts::Index(index_name))
            .body(document)
            .send()
            .await?;

        if response.status_code().is_success() {
            info!("[EsRepositoryPub::post_query] index_name: {}", index_name);
            Ok(())
        } else {
            let error_message = format!(
                "[EsRepositoryPub::post_query] Failed to index document: Status Code: {}",
                response.status_code()
            );
            Err(anyhow!(error_message))
        }
    }

    #[doc = "Function that EXECUTES elasticsearch queries - delete"]
    async fn delete_query(&self, doc_id: &str, index_name: &str) -> anyhow::Result<()> {
        let response = self
            .es_client
            .delete(DeleteParts::IndexId(index_name, doc_id))
            .send()
            .await?;

        if response.status_code().is_success() {
            info!(
                "[EsRepositoryPub::delete_query] index name: {}, doc_id: {}",
                index_name, doc_id
            );
            Ok(())
        } else {
            let error_message = format!("[EsRepositoryPub::delete_query] Failed to delete document: Status Code: {}, Document ID: {}", response.status_code(), doc_id);
            Err(anyhow!(error_message))
        }
    }
}
