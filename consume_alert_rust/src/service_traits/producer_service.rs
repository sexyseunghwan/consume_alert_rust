use crate::common::*;

#[async_trait]
pub trait ProducerService {
    async fn produce_object_to_topic<T>(
        &self,
        topic: &str,
        object: &T,
        key: Option<&str>,
    ) -> Result<(), anyhow::Error>
    where
        T: Serialize + Send + Sync;

    async fn produce_objects_to_topic<T, F>(
        &self,
        topic: &str,
        objects: &[T],
        key_fn: Option<F>,
    ) -> Result<(), anyhow::Error>
    where
        T: Serialize + Send + Sync,
        F: Fn(&T) -> String + Send + Sync + 'static;
}
