use crate::common::*;
use crate::service_traits::{
    cache_service::*, elastic_query_service::*, graph_api_service::*, mysql_query_service::*,
    process_service::*, producer_service::*, redis_service::*, telebot_service::*,
};

pub struct MainControllerServicesDto<G, E, M, T, P, KP, R, C>
where
    G: GraphApiService,
    E: ElasticQueryService,
    M: MysqlQueryService,
    T: TelebotService,
    P: ProcessService,
    KP: ProducerService,
    R: RedisService,
    C: CacheService,
{
    pub graph_api_service: Arc<G>,
    pub elastic_query_service: Arc<E>,
    pub mysql_query_service: Arc<M>,
    pub tele_bot_service: T,
    pub process_service: Arc<P>,
    pub producer_service: Arc<KP>,
    pub redis_service: Arc<R>,
    pub cache_service: Arc<C>,
}
