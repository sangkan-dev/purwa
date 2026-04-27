## Queue & Jobs (Phase 2 MVP)

This document describes the **minimal** Purwa queue integration built around **Redis**.

### Concepts

- **at-least-once** delivery: a job may run more than once; write handlers to be idempotent.
- **main queue**: Redis list `queue:{name}`
- **retry queue**: Redis zset `queue:{name}:retry` (score = unix ms when job becomes runnable)

### Configuration

In `purwa.toml`:

```toml
[queue]
name = "default"
redis_url = "redis://127.0.0.1:6379"
```

Or set `REDIS_URL` as an environment variable (used when `[queue].redis_url` is unset).

### Enqueue from app code (library-first)

In a handler or service:

```rust
let cfg = purwa::AppConfig::load()?;
let redis_url = cfg
    .queue_redis_url()
    .ok_or(\"missing queue redis url\")?;
let qcfg = purwa_queue::QueueConfig::new(redis_url, cfg.queue.name.clone());
let queue = purwa_queue::Queue::connect(qcfg)?;

// job payload must be serde-serializable
queue.enqueue(&MyJob { /* ... */ }).await?;
```

### Worker

Scaffolded apps include a `queue-worker` binary. Run it with:

```bash
empu queue:work
```

Under the hood this runs `cargo run --bin queue-worker` in your app.

### Job generator

Create a job skeleton:

```bash
empu make:job SendEmail
```

This writes a job module under `src/app/jobs/`. Jobs auto-register via the `#[job]` proc-macro
(inventory), so the worker only needs to ensure your `jobs` modules are compiled.

The generated job includes a `perform(self, ctx)` method you can fill in.

### Macro: `#[job]`

Purwa registers jobs via `inventory`, similar to route registration.

Example:

```rust
use purwa::job;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[job(type = "send-email")]
pub struct SendEmail {
    pub to: String,
}

impl SendEmail {
    pub async fn perform(self, _ctx: purwa_queue::JobContext) -> Result<(), String> {
        Ok(())
    }
}
```

### Testing

- **Local Redis / CI service**: set `TEST_REDIS_URL` and run `cargo test -p purwa-queue`.\n+- **Docker testcontainers**: there is an ignored test you can run explicitly:\n+\n+```bash\n+cargo test -p purwa-queue -- --ignored\n+```\n 

