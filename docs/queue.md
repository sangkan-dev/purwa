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

This writes a job module under `src/app/jobs/` and ensures `src/app/jobs/mod.rs` has deterministic
markers for registration. Wire your job logic inside the generated handler.

