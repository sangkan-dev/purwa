pub fn queue_list(name: &str) -> String {
    format!("queue:{name}")
}

pub fn queue_retry_zset(name: &str) -> String {
    format!("queue:{name}:retry")
}
