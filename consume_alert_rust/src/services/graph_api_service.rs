use crate::common::*;

static HTTP_CLIENT: once_lazy<Client> = once_lazy::new(|| initialize_http_clients());

#[doc = "Function to initialize the HTTP client"]
fn initialize_http_clients() -> Client {
    reqwest::Client::new()
}

#[async_trait]
pub trait GraphApiService {
    
}

#[derive(Debug, Getters, Clone)]
pub struct GraphApiServicePub {
    graph_api_url: String,
}

impl GraphApiServicePub {
    pub fn new() -> Self {
        let graph_api_url: String = env::var("GRAPH_API_URL").expect(
            "[ENV file read Error][GraphApiServicePub -> new()] 'GRAPH_API_URL' must be set",
        );
        
        Self { graph_api_url }
    }
}

#[async_trait]
impl GraphApiService for GraphApiServicePub {}
