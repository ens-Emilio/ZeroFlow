use crate::providers;
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SearchProvider {
    TraceMoe,
    GoogleLens,
    YandexImages,
    Pinterest,
}

impl SearchProvider {
    /// Search page used by "match image search" text queries.
    /// Port of reference/CTS/Search/TextSearchUrlBuilder.cs Template().
    pub(crate) fn text_search_template(self) -> &'static str {
        match self {
            SearchProvider::TraceMoe => "https://anilist.co/search/anime?search=%s",
            SearchProvider::GoogleLens => "https://www.google.com/search?q=%s",
            SearchProvider::YandexImages => "https://yandex.com/search/?text=%s",
            // Pinterest search sits behind a login wall, so it falls back to Google.
            SearchProvider::Pinterest => "https://www.google.com/search?q=%s",
        }
    }
}

pub(crate) fn parse_provider_id(id: &str) -> Result<SearchProvider, String> {
    match id.trim().to_ascii_lowercase().as_str() {
        "tracemoe" | "trace.moe" | "trace_moe" => Ok(SearchProvider::TraceMoe),
        "googlelens" | "google-lens" | "google_lens" | "lens" => Ok(SearchProvider::GoogleLens),
        "yandeximages" | "yandex-images" | "yandex_images" | "yandex" => Ok(SearchProvider::YandexImages),
        "pinterest" => Ok(SearchProvider::Pinterest),
        other => Err(format!("unknown visual search provider: {other}")),
    }
}

fn retryable(provider: SearchProvider, error: &str) -> bool {
    // Only trace.moe performs a repeatable search request; other providers upload
    // the image or launch a browser flow, so a retry can duplicate side effects.
    if provider != SearchProvider::TraceMoe {
        return false;
    }
    let lower = error.to_ascii_lowercase();
    lower.contains("timeout")
        || lower.contains("timed out")
        || lower.contains("request failed")
        || lower.contains("request:")
        || lower.contains("upload failed")
        || lower.split_whitespace().any(|word| {
            word.trim_matches(|c: char| !c.is_ascii_digit())
                .parse::<u16>()
                .map(|status| status == 408 || status == 429 || (500..=599).contains(&status))
                .unwrap_or(false)
        })
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

    if !retryable(provider, &first_error) {
        return Err(first_error);
    }

    run_provider(provider, jpeg_base64)
        .await
        .map_err(|second_error| format!("{first_error}; retry failed: {second_error}"))
}

#[cfg(test)]
mod tests {
    use super::{parse_provider_id, retryable, SearchProvider};

    #[test]
    fn provider_ids_route_to_existing_flows() {
        assert_eq!(parse_provider_id("tracemoe"), Ok(SearchProvider::TraceMoe));
        assert_eq!(parse_provider_id("googlelens"), Ok(SearchProvider::GoogleLens));
        assert_eq!(parse_provider_id("yandeximages"), Ok(SearchProvider::YandexImages));
        assert_eq!(parse_provider_id("pinterest"), Ok(SearchProvider::Pinterest));
        assert!(parse_provider_id("not-a-provider").is_err());
    }

    #[test]
    fn retry_policy_avoids_repeating_upload_and_browser_flows() {
        assert!(retryable(SearchProvider::TraceMoe, "HTTP 429 Too Many Requests"));
        assert!(retryable(SearchProvider::TraceMoe, "request failed: timed out"));
        assert!(!retryable(SearchProvider::GoogleLens, "request failed: timed out"));
        assert!(!retryable(SearchProvider::YandexImages, "upload failed: connection reset"));
        assert!(!retryable(SearchProvider::Pinterest, "HTTP 503"));
        assert!(!retryable(SearchProvider::TraceMoe, "invalid base64: invalid byte"));
    }
}
