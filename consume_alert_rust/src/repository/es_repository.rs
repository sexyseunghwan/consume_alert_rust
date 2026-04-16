use crate::common::*;

#[async_trait]
pub trait EsRepository {
    /// Executes an Elasticsearch search query against the given index and returns the raw JSON response.
    ///
    /// # Arguments
    ///
    /// * `es_query` - The Elasticsearch query DSL as a JSON value
    /// * `index_name` - The name of the index to search
    ///
    /// # Returns
    ///
    /// Returns `Ok(Value)` with the raw Elasticsearch response body on success.
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or the response indicates a non-success status.
    async fn get_search_query(
        &self,
        es_query: &Value,
        index_name: &str,
    ) -> Result<Value, anyhow::Error>;

    /// Deletes a document identified by `doc_id` from the specified Elasticsearch index.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - The document ID to delete
    /// * `index_name` - The name of the index containing the document
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP request fails or the response indicates a non-success status.
    #[allow(dead_code)]
    async fn delete_query(&self, doc_id: &str, index_name: &str) -> Result<(), anyhow::Error>;
}

#[derive(Debug, Getters, Clone)]
pub struct EsRepositoryPub {
    es_client: Elasticsearch,
}

impl EsRepositoryPub {
    /// Creates a new `EsRepositoryPub` by reading Elasticsearch connection settings from environment variables.
    ///
    /// # Returns
    ///
    /// Returns `Ok(EsRepositoryPub)` on successful connection setup.
    ///
    /// # Errors
    ///
    /// Returns an error if any required environment variable is missing or the transport cannot be built.
    pub fn new() -> anyhow::Result<Self> {
        let es_host: Vec<String> = env::var("ES_DB_URL")
            .expect("[EsRepositoryPub::new] 'ES_DB_URL' must be set")
            .split(',')
            .map(|s| s.to_string())
            .collect();

        let es_id: String = env::var("ES_ID").inspect_err(|e| {
            error!("[EsRepositoryPub::new] 'ES_ID' must be set: {:#}", e);
        })?;

        let es_pw: String = env::var("ES_PW").inspect_err(|e| {
            error!("[EsRepositoryPub::new] 'ES_PW' must be set: {:#}", e);
        })?;

        let cluster_urls: Vec<Url> = es_host
            .iter()
            .map(|host| Url::parse(&format!("http://{}", host)))
            .collect::<Result<_, _>>()
            .map_err(|e| anyhow!("[EsRepositoryPub::new] {:?}", e))?;

        let conn_pool: MultiNodeConnectionPool =
            MultiNodeConnectionPool::round_robin(cluster_urls, None);

        let mut builder: TransportBuilder =
            TransportBuilder::new(conn_pool).timeout(Duration::from_secs(30));

        if !es_id.is_empty() && !es_pw.is_empty() {
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
