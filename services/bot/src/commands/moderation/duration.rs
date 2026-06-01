use std::time::Duration;

pub fn parse_duration(input: &str) -> Option<Duration> {
    let input = input.trim().to_lowercase();
    let (num_str, unit) = input.split_at(input.len().saturating_sub(1));
    let num: u64 = num_str.parse().ok()?;

    match unit {
        "s" => Some(Duration::from_secs(num)),
        "m" => Some(Duration::from_secs(num * 60)),
        "h" => Some(Duration::from_secs(num * 3600)),
        "d" => Some(Duration::from_secs(num * 86400)),
        _ => None,
    }
}
