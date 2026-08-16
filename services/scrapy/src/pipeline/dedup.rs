pub fn key(job_id: &str, url: &str) -> String {
    format!("{job_id}:{url}")
}
