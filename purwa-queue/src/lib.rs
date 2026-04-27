//! Redis-backed job queue for Purwa (Phase 2).
//!
//! This is a **minimal MVP** queue:
//!
//! - **At-least-once** delivery semantics.
//! - Retry with backoff via a retry zset (`queue:{name}:retry`).
//! - Main queue uses a Redis list (`queue:{name}`).
//!
//! The worker needs an application-side registry mapping `job_type` → handler. This crate
//! provides [`JobRegistry`] and the wire format; the app binary wires handlers.

#![forbid(unsafe_code)]

pub use inventory;

mod redis_keys;

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use deadpool_redis::{Config as RedisConfig, Pool, Runtime};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use time::OffsetDateTime;

pub type JobHandleFuture = Pin<Box<dyn Future<Output = Result<(), String>> + Send + 'static>>;
pub type JobHandleFn = fn(Value, JobContext) -> JobHandleFuture;

#[derive(Clone)]
pub struct JobHandlerEntry {
    pub job_type: &'static str,
    pub handle: JobHandleFn,
}

inventory::collect!(JobHandlerEntry);

/// Queue configuration.
#[derive(Clone, Debug)]
pub struct QueueConfig {
    pub redis_url: String,
    pub name: String,
    pub max_attempts: u32,
    pub backoff: Backoff,
    /// Worker blocking pop timeout.
    pub pop_timeout: Duration,
}

impl QueueConfig {
    pub fn new(redis_url: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            redis_url: redis_url.into(),
            name: name.into(),
            max_attempts: 25,
            backoff: Backoff::exponential(Duration::from_secs(1), Duration::from_secs(60)),
            pop_timeout: Duration::from_secs(5),
        }
    }
}

/// Backoff policy for retries.
#[derive(Clone, Copy, Debug)]
pub struct Backoff {
    pub base: Duration,
    pub max: Duration,
}

impl Backoff {
    pub fn exponential(base: Duration, max: Duration) -> Self {
        Self { base, max }
    }

    pub fn delay_for_attempt(self, attempt: u32) -> Duration {
        // attempt starts at 1
        let pow = attempt.saturating_sub(1).min(30);
        let mult = 1u64 << pow;
        let secs = self.base.as_secs().saturating_mul(mult);
        Duration::from_secs(secs.min(self.max.as_secs()))
    }
}

/// Wire format stored in Redis.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JobEnvelope {
    pub job_type: String,
    pub payload: Value,
    pub attempt: u32,
    pub enqueued_at_unix_ms: i128,
}

impl JobEnvelope {
    pub fn new(job_type: impl Into<String>, payload: Value) -> Self {
        Self {
            job_type: job_type.into(),
            payload,
            attempt: 0,
            enqueued_at_unix_ms: OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000,
        }
    }
}

/// Job trait used for enqueue convenience (library-first).
pub trait Job: Serialize {
    const TYPE: &'static str;
}

/// Context passed to job handlers.
#[derive(Clone, Debug)]
pub struct JobContext {
    pub queue_name: String,
}

/// Errors from enqueue/worker operations.
#[derive(Debug, Error)]
pub enum QueueError {
    #[error("redis: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("pool: {0}")]
    Pool(#[from] deadpool_redis::PoolError),
    #[error("serde_json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("unknown job type: {0}")]
    UnknownJobType(String),
    #[error("job handler failed: {0}")]
    JobFailed(String),
    #[error("invalid queue config: {0}")]
    InvalidConfig(String),
}

type HandlerFn = Arc<dyn Fn(Value, JobContext) -> JobHandleFuture + Send + Sync>;

/// Application-side mapping from `job_type` to handler function.
#[derive(Clone, Default)]
pub struct JobRegistry {
    handlers: Arc<HashMap<String, HandlerFn>>,
}

impl JobRegistry {
    pub fn builder() -> JobRegistryBuilder {
        JobRegistryBuilder::default()
    }

    pub fn from_inventory() -> Self {
        let mut handlers: HashMap<String, HandlerFn> = HashMap::new();
        for e in inventory::iter::<JobHandlerEntry> {
            let job_type = e.job_type.to_string();
            let handle = e.handle;
            let fun: HandlerFn = Arc::new(move |payload, ctx| (handle)(payload, ctx));
            handlers.insert(job_type, fun);
        }
        Self {
            handlers: Arc::new(handlers),
        }
    }

    fn handler(&self, job_type: &str) -> Option<&HandlerFn> {
        self.handlers.get(job_type)
    }
}

#[derive(Default)]
pub struct JobRegistryBuilder {
    handlers: HashMap<String, HandlerFn>,
}

impl JobRegistryBuilder {
    /// Register a handler by type string.
    pub fn register_raw<F, Fut>(mut self, job_type: impl Into<String>, f: F) -> Self
    where
        F: Fn(Value, JobContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), String>> + Send + 'static,
    {
        let key = job_type.into();
        let fun: HandlerFn = Arc::new(move |payload, ctx| Box::pin(f(payload, ctx)));
        self.handlers.insert(key, fun);
        self
    }

    pub fn build(self) -> JobRegistry {
        JobRegistry {
            handlers: Arc::new(self.handlers),
        }
    }
}

/// Queue client used by application code to enqueue jobs.
#[derive(Clone)]
pub struct Queue {
    cfg: QueueConfig,
    pool: Pool,
}

impl Queue {
    pub fn connect(cfg: QueueConfig) -> Result<Self, QueueError> {
        if cfg.redis_url.trim().is_empty() {
            return Err(QueueError::InvalidConfig("redis_url is empty".into()));
        }
        let mut rcfg = RedisConfig::from_url(cfg.redis_url.clone());
        rcfg.pool = Some(deadpool_redis::PoolConfig::new(16));
        let pool = rcfg
            .create_pool(Some(Runtime::Tokio1))
            .map_err(|e| QueueError::InvalidConfig(e.to_string()))?;
        Ok(Self { cfg, pool })
    }

    pub async fn enqueue<J: Job>(&self, job: &J) -> Result<(), QueueError> {
        let payload = serde_json::to_value(job)?;
        let env = JobEnvelope::new(J::TYPE, payload);
        self.enqueue_envelope(env).await
    }

    pub async fn enqueue_envelope(&self, env: JobEnvelope) -> Result<(), QueueError> {
        let raw = serde_json::to_string(&env)?;
        let key = redis_keys::queue_list(&self.cfg.name);
        let mut conn = self.pool.get().await?;
        conn.lpush::<_, _, ()>(key, raw).await?;
        Ok(())
    }
}

/// Worker that executes jobs from a queue using a [`JobRegistry`].
pub struct Worker {
    cfg: QueueConfig,
    pool: Pool,
    registry: JobRegistry,
}

impl Worker {
    pub fn new(queue: &Queue, registry: JobRegistry) -> Self {
        Self {
            cfg: queue.cfg.clone(),
            pool: queue.pool.clone(),
            registry,
        }
    }

    pub fn from_inventory(queue: &Queue) -> Self {
        Self::new(queue, JobRegistry::from_inventory())
    }

    pub async fn run(&self) -> Result<(), QueueError> {
        loop {
            self.run_once().await?;
        }
    }

    pub async fn run_once(&self) -> Result<(), QueueError> {
        self.move_due_retries().await?;
        let key = redis_keys::queue_list(&self.cfg.name);
        let mut conn = self.pool.get().await?;

        let res: Option<(String, String)> =
            conn.brpop(key, self.cfg.pop_timeout.as_secs_f64()).await?;
        let Some((_k, raw)) = res else {
            return Ok(());
        };
        drop(conn);

        let env: JobEnvelope = serde_json::from_str(&raw)?;
        self.execute_or_retry(env).await
    }

    async fn move_due_retries(&self) -> Result<(), QueueError> {
        let now_ms = OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000;
        let retry_key = redis_keys::queue_retry_zset(&self.cfg.name);
        let list_key = redis_keys::queue_list(&self.cfg.name);

        let mut conn = self.pool.get().await?;
        // Pull up to 100 due jobs.
        let raws: Vec<String> = redis::cmd("ZRANGEBYSCORE")
            .arg(&retry_key)
            .arg("-inf")
            .arg(now_ms)
            .arg("LIMIT")
            .arg(0)
            .arg(100)
            .query_async(&mut conn)
            .await?;
        if raws.is_empty() {
            return Ok(());
        }
        for raw in raws {
            // Best-effort: remove then push.
            let _: i64 = conn.zrem(&retry_key, &raw).await?;
            conn.lpush::<_, _, ()>(&list_key, raw).await?;
        }
        Ok(())
    }

    async fn execute_or_retry(&self, mut env: JobEnvelope) -> Result<(), QueueError> {
        let Some(handler) = self.registry.handler(&env.job_type) else {
            return Err(QueueError::UnknownJobType(env.job_type));
        };

        env.attempt = env.attempt.saturating_add(1);
        let ctx = JobContext {
            queue_name: self.cfg.name.clone(),
        };
        let result = handler(env.payload.clone(), ctx).await;

        match result {
            Ok(()) => Ok(()),
            Err(msg) => {
                if env.attempt >= self.cfg.max_attempts {
                    return Err(QueueError::JobFailed(msg));
                }
                self.schedule_retry(env).await?;
                Ok(())
            }
        }
    }

    async fn schedule_retry(&self, env: JobEnvelope) -> Result<(), QueueError> {
        let delay = self.cfg.backoff.delay_for_attempt(env.attempt);
        let at_ms = (OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000)
            + delay.as_millis() as i128;
        let retry_key = redis_keys::queue_retry_zset(&self.cfg.name);
        let raw = serde_json::to_string(&env)?;

        let mut conn = self.pool.get().await?;
        // ZADD retry_key score raw
        let _: i64 = redis::cmd("ZADD")
            .arg(&retry_key)
            .arg(at_ms)
            .arg(raw)
            .query_async(&mut conn)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static TESTCALLS: AtomicUsize = AtomicUsize::new(0);

    fn testcontainers_ok_job_handle(_payload: Value, _ctx: JobContext) -> JobHandleFuture {
        Box::pin(async move {
            TESTCALLS.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
    }

    inventory::submit! {
        JobHandlerEntry {
            job_type: "ok-job",
            handle: testcontainers_ok_job_handle,
        }
    }

    #[tokio::test]
    async fn integration_smoke_via_test_redis_url_env() {
        let Ok(url) = std::env::var("TEST_REDIS_URL") else {
            return;
        };
        let url = url.trim().to_string();
        if url.is_empty() {
            return;
        }

        #[derive(Serialize)]
        struct OkJob;
        impl Job for OkJob {
            const TYPE: &'static str = "ok-job";
        }

        let cfg = QueueConfig {
            pop_timeout: Duration::from_millis(50),
            ..QueueConfig::new(url, "test")
        };
        let q = Queue::connect(cfg).unwrap();
        let registry = JobRegistry::builder()
            .register_raw("ok-job", |_payload, _ctx| async move { Ok(()) })
            .build();
        let w = Worker::new(&q, registry);

        q.enqueue(&OkJob).await.unwrap();
        w.run_once().await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn integration_testcontainers_smoke() {
        use testcontainers_modules::redis::Redis;
        use testcontainers_modules::testcontainers::runners::AsyncRunner;

        let node = Redis::default().start().await.unwrap();
        let host_ip = node.get_host().await.unwrap();
        let host_port = node.get_host_port_ipv4(6379).await.unwrap();
        let url = format!("redis://{host_ip}:{host_port}");

        let cfg = QueueConfig {
            pop_timeout: Duration::from_millis(50),
            ..QueueConfig::new(url, "test")
        };
        let q = Queue::connect(cfg).unwrap();

        let w = Worker::from_inventory(&q);
        q.enqueue_envelope(JobEnvelope::new("ok-job", serde_json::json!({})))
            .await
            .unwrap();
        w.run_once().await.unwrap();

        assert_eq!(TESTCALLS.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn backoff_caps() {
        let b = Backoff::exponential(Duration::from_secs(1), Duration::from_secs(10));
        assert_eq!(b.delay_for_attempt(1).as_secs(), 1);
        assert_eq!(b.delay_for_attempt(2).as_secs(), 2);
        assert_eq!(b.delay_for_attempt(5).as_secs(), 10);
    }

    #[test]
    fn envelope_roundtrip() {
        let e = JobEnvelope::new("x", serde_json::json!({ "a": 1 }));
        let raw = serde_json::to_string(&e).unwrap();
        let back: JobEnvelope = serde_json::from_str(&raw).unwrap();
        assert_eq!(back.job_type, "x");
        assert_eq!(back.payload["a"], 1);
    }
}
