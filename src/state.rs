use crate::db::DbExecutor;
use crate::sessions::session_manager::SessionManager;
use crate::store::ObjectStore;
use crate::Configuration;
use actix::{Actor, Addr, SyncArbiter};
use actix_redis::RedisActor;
use std::sync::Arc;

#[derive(Clone)]
pub struct State {
    db: Addr<DbExecutor>,
    mem: Addr<RedisActor>,
    pub sessions: Addr<SessionManager>,
    store: Addr<ObjectStore>,
    pub config: Arc<Configuration>,
}

impl State {
    pub fn new(config: &Configuration) -> State {
        // r2d2 pool
        let manager = diesel::r2d2::ConnectionManager::new(config.database_url());
        let pool = diesel::r2d2::Pool::new(manager).unwrap();

        // Start db executor actors
        let db_addr = SyncArbiter::start(3, move || DbExecutor(pool.clone()));
        let redis_addr = RedisActor::start(config.redis_url());
        println!("REDIS: {}", config.redis_url());

        let session_actor = SessionManager {
            redis: redis_addr.clone(),
            pg: db_addr.clone(),
        };

        let session_addr = session_actor.start();

        let store_actor = ObjectStore::new_with_s3_credentials(
            config.s3_access_key_id(),
            config.s3_secret_access_key(),
        )
        .expect("No TLS errors starting store_actor");
        let store_addr = store_actor.start();

        State {
            db: db_addr.clone(),
            mem: redis_addr.clone(),
            sessions: session_addr.clone(),
            store: store_addr.clone(),
            config: Arc::new(config.clone()),
        }
    }
}
