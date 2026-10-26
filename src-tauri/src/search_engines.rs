// Search engine URL builders. Pure functions; no network here.
// Port of reference/CTS/Search/TextSearchEngines.cs + TextSearchUrlBuilder.cs.

use crate::search_coordinator::parse_provider_id;

const QUERY_PLACEHOLDER: &str = "%s";

/// "Match image search" uses the selected visual provider's own search page
/// instead of a dedicated text engine.
pub const MATCH_IMAGE_SEARCH: &str = "matchimagesearch";

fn engine_template(engine: &str) -> Option<&'static str> {
    match engine {
        "google" => Some("https://www.google.com/search?q=%s"),
        "bing" => Some("https://www.bing.com/search?q=%s"),
        "duckduckgo" => Some("https://duckduckgo.com/?q=%s"),
        "kagi" => Some("https://kagi.com/search?q=%s"),
        "qwant" => Some("https://www.qwant.com/?q=%s"),
        "startpage" => Some("https://www.startpage.com/sp/search?q=%s"),
        _ => None,
    }
}

fn search_url(query: &str, provider: &str, engine: &str) -> Result<String, String> {
    let template = if engine == MATCH_IMAGE_SEARCH {
        parse_provider_id(provider)?.text_search_template()
    } else {
        engine_template(engine).ok_or_else(|| format!("unknown text search engine: {engine}"))?
    };
    Ok(template.replace(QUERY_PLACEHOLDER, &urlencoding::encode(query)))
}

#[tauri::command]
pub fn open_search(query: String, provider: String, engine: String) -> Result<(), String> {
    let url = search_url(&query, &provider, &engine)?;
    std::process::Command::new("xdg-open")
        .arg(&url)
        .spawn()
        .map_err(|e| format!("xdg-open failed: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{search_url, MATCH_IMAGE_SEARCH};

    #[test]
    fn unknown_engine_is_not_silently_routed() {
        assert!(super::search_url("test", "googlelens", "unknown").is_err());
    }

    #[test]
    fn match_image_search_uses_the_selected_provider() {
        assert_eq!(
            search_url("hello world", "googlelens", MATCH_IMAGE_SEARCH).unwrap(),
            "https://www.google.com/search?q=hello%20world"
        );
        assert_eq!(
            search_url("hello world", "tracemoe", MATCH_IMAGE_SEARCH).unwrap(),
            "https://anilist.co/search/anime?search=hello%20world"
        );
        assert_eq!(
            search_url("hello world", "yandeximages", MATCH_IMAGE_SEARCH).unwrap(),
            "https://yandex.com/search/?text=hello%20world"
        );
        assert!(search_url("hello", "not-a-provider", MATCH_IMAGE_SEARCH).is_err());
    }

    #[test]
    fn dedicated_engines_build_their_own_url() {
        assert_eq!(
            search_url("a b", "googlelens", "bing").unwrap(),
            "https://www.bing.com/search?q=a%20b"
        );
    }
}
