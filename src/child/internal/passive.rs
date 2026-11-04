//! Parser-safe helpers: no host effects beyond diagnostic stderr.

pub(super) fn log(plugin_name: &str, level_str: &str, message: &str) {
    eprintln!("[child:{}] {}: {}", plugin_name, level_str, message);
}

pub(crate) fn validate_http_url(url: &str) -> Result<String, String> {
    let parsed = reqwest::Url::parse(url).map_err(|e| format!("invalid URL: {}", e))?;

    if parsed.scheme() != "https" {
        return Err(format!("only HTTPS allowed, got '{}'", parsed.scheme()));
    }

    let host = parsed
        .host_str()
        .ok_or_else(|| "no host in URL".to_string())?;

    if host == "localhost" {
        return Err("localhost not allowed".to_string());
    }

    let bare_host = host
        .strip_prefix('[')
        .and_then(|h| h.strip_suffix(']'))
        .unwrap_or(host);
    if bare_host.parse::<std::net::IpAddr>().is_ok() {
        return Err("IP addresses not allowed".to_string());
    }

    Ok(bare_host.to_string())
}

#[cfg(test)]
mod tests {
    use super::validate_http_url;

    #[test]
    fn parser_validation_preserves_domain_boundary_without_host_access() {
        assert_eq!(
            validate_http_url("https://example.org/path").unwrap(),
            "example.org"
        );
        for invalid in [
            "http://example.org",
            "https://localhost",
            "https://127.0.0.1",
            "https://[::1]",
            "not a URL",
        ] {
            assert!(validate_http_url(invalid).is_err(), "{invalid}");
        }
    }
}
