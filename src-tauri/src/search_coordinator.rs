use crate::providers;
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SearchProvider {
    TraceMoe,
    GoogleLens,
    YandexImages,
    Pinterest,
}

fn parse_provider_id(id: &str) -> Result<SearchProvider, String> {
    match id.trim().to_ascii_lowercase().as_str() {
        "tracemoe" | "trace.moe" | "trace_moe" => Ok(SearchProvider::TraceMoe),
        "googlelens" | "google-lens" | "google_lens" | "lens" => Ok(SearchProvider::GoogleLens),
        "yandeximages" | "yandex-images" | "yandex_images" | "yandex" => Ok(SearchProvider::YandexImages),
        "pinterest" => Ok(SearchProvider::Pinterest),
        other => Err(format!("unknown visual search provider: {other}")),
    }
}

fn is_transient_error(error: &str) -> bool {
    let lower = error.to_ascii_lowercase();
    if lower.contains("timeout")
        || lower.contains("timed out")
        || lower.contains("request failed")
        || lower.contains("request:")
        || lower.contains("upload failed")
    {
        return true;
    }

    lower
        .split_whitespace()
        .any(|word| word.trim_matches(|c: char| !c.is_ascii_digit())
            .parse::<u16>()
            .map(|status| status == 408 || status == 429 || (500..=599).contains(&status))
            .unwrap_or(false))
}

async fn run_provider(provider: SearchProvider, jpeg_base64: String) -> Result<Value, String> {
    match provider {
        SearchProvider::TraceMoe => providers::search_tracemoe_inner(jpeg_base64).await,
        SearchProvider::GoogleLens => providers::open_lens(jpeg_base64).await,
        SearchProvider::YandexImages => providers::search_yandex(jpeg_base64).await,
        SearchProvider::Pinterest => providers::search_pinterest(jpeg_base64).await,
    }
}

/// Routes one visual search through the selected provider and retries one transient failure.
#[tauri::command]
pub async fn search_visual(
    provider_id: String,
    jpeg_base64: String,
) -> Result<Value, String> {
    let provider = parse_provider_id(&provider_id)?;
    let first_error = match run_provider(provider, jpeg_base64.clone()).await {
        Ok(value) => return Ok(value),
        Err(error) => error,
    };

    if !is_transient_error(&first_error) {
        return Err(first_error);
    }

    run_provider(provider, jpeg_base64)
        .await
        .map_err(|second_error| format!("{first_error}; retry failed: {second_error}"))
}

#[cfg(test)]
mod tests {
    use super::{is_transient_error, parse_provider_id, SearchProvider};

    #[test]
    fn provider_ids_route_to_existing_flows() {
        assert_eq!(parse_provider_id("tracemoe"), Ok(SearchProvider::TraceMoe));
        assert_eq!(parse_provider_id("googlelens"), Ok(SearchProvider::GoogleLens));
        assert_eq!(parse_provider_id("yandeximages"), Ok(SearchProvider::YandexImages));
        assert_eq!(parse_provider_id("pinterest"), Ok(SearchProvider::Pinterest));
        assert!(parse_provider_id("not-a-provider").is_err());
    }

    #[test]
    fn only_transient_provider_errors_are_retryable() {
        assert!(is_transient_error("trace.moe HTTP 429 Too Many Requests"));
        assert!(is_transient_error("upload failed: connection reset"));
        assert!(is_transient_error("Lens request failed: timed out"));
        assert!(is_transient_error("provider returned HTTP 503"));
        assert!(!is_transient_error("invalid base64: invalid byte"));
        assert!(!is_transient_error("Google Lens did not return a results page"));
    }
}
