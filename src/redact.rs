// Keeps provider API keys out of logs, stored errors and caller-facing errors.
// Exports url_host, host_label and redact_url to the pool, tiered and WebSocket paths.
// Deps: url for parsing; everything else is std.

/// Secrets at least this long are removed wherever they occur, even inside other text.
/// Path segments and fragments shorter than this are route names, not keys.
const MIN_EMBEDDED_LEN: usize = 16;
/// Shortest credential (userinfo, query value) removed, and then only as a whole token.
const MIN_TOKEN_LEN: usize = 4;
const REDACTED: &str = "<redacted>";

/// Host of an endpoint URL, the only part safe to log or hand back to callers.
///
/// Provider URLs carry the API key in userinfo, path or query; the host never does.
pub(crate) fn url_host(url: &str) -> Option<String> {
    url::Url::parse(url).ok()?.host_str().map(str::to_string)
}

/// Host of `url` for a message, or `"endpoint"` when it has none.
#[cfg(feature = "ws")]
pub(crate) fn host_label(url: &str) -> String {
    url_host(url).unwrap_or_else(|| "endpoint".to_string())
}

/// `text` with every rendering of `endpoint_url` reduced to its host.
///
/// Each `scheme://…` token becomes `scheme://host`. The endpoint's credentials, raw and
/// percent-decoded, are then removed wherever they still appear, which covers scheme-less
/// renderings and a provider quoting the key back.
pub(crate) fn redact_url(text: &str, endpoint_url: &str) -> String {
    let mut out = reduce_url_tokens(text);
    for secret in secrets(endpoint_url) {
        out = if secret.len() >= MIN_EMBEDDED_LEN {
            out.replace(&secret, REDACTED)
        } else {
            replace_token(&out, &secret)
        };
    }
    out
}

fn reduce_url_tokens(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(sep) = rest.find("://") {
        let start = rest[..sep]
            .char_indices()
            .rev()
            .find(|&(_, c)| !(c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.')))
            .map_or(0, |(i, c)| i + c.len_utf8());
        let end = rest[sep..]
            .find(|c: char| {
                c.is_whitespace() || matches!(c, '(' | ')' | '"' | '\'' | '<' | '>' | ',')
            })
            .map_or(rest.len(), |i| sep + i);
        out.push_str(&rest[..start]);
        let reduced = url::Url::parse(&rest[start..end])
            .ok()
            .and_then(|u| u.host_str().map(|host| format!("{}://{host}", u.scheme())));
        out.push_str(reduced.as_deref().unwrap_or(REDACTED));
        rest = &rest[end..];
    }
    out.push_str(rest);
    out
}

/// Credentials of `endpoint_url`, raw and percent-decoded, longest first: userinfo and
/// query values (parameter names are not secrets), plus path segments and the fragment
/// when they are long enough to be a key. A query value cannot be told apart from a key,
/// so a routing value such as `network=ethereum` is removed from messages as well.
fn secrets(endpoint_url: &str) -> Vec<String> {
    let Ok(url) = url::Url::parse(endpoint_url) else {
        return Vec::new();
    };
    let query = url.query().unwrap_or("");
    let credentials = [url.username(), url.password().unwrap_or("")]
        .into_iter()
        .chain(
            query
                .split('&')
                .filter_map(|pair| pair.split_once('=').map(|(_, v)| v)),
        )
        .map(|part| (part, MIN_TOKEN_LEN));
    let key_like = url
        .path_segments()
        .into_iter()
        .flatten()
        .chain(url.fragment())
        .map(|part| (part, MIN_EMBEDDED_LEN));

    let mut out = Vec::new();
    for (part, min_len) in credentials.chain(key_like) {
        for variant in [part.to_string(), percent_decode(part)] {
            if variant.chars().count() >= min_len {
                out.push(variant);
            }
        }
    }
    out.sort_by(|a, b| b.len().cmp(&a.len()).then_with(|| a.cmp(b)));
    out.dedup();
    out
}

/// `text` with each occurrence of `token` that stands as a whole word replaced.
///
/// Boundaries are judged on the original text, so a run of repeated content such as
/// zero-padded revert data never exposes an inner match as a word.
fn replace_token(text: &str, token: &str) -> String {
    let is_word = |c: char| c.is_ascii_alphanumeric() || matches!(c, '_' | '-');
    let mut out = String::with_capacity(text.len());
    let mut copied = 0;
    for (i, _) in text.match_indices(token) {
        let end = i + token.len();
        let bounded = !text[..i].chars().next_back().is_some_and(is_word)
            && !text[end..].chars().next().is_some_and(is_word);
        if bounded {
            out.push_str(&text[copied..i]);
            out.push_str(REDACTED);
            copied = end;
        }
    }
    out.push_str(&text[copied..]);
    out
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let hex = bytes
            .get(i + 1..i + 3)
            .and_then(|h| std::str::from_utf8(h).ok());
        match (bytes[i], hex.and_then(|h| u8::from_str_radix(h, 16).ok())) {
            (b'%', Some(byte)) => {
                out.push(byte);
                i += 3;
            }
            (byte, _) => {
                out.push(byte);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: &str = "Zx9SYNTHETICkey0123456789abcdef";

    #[test]
    fn url_host_drops_userinfo_path_and_query() {
        for url in [
            format!("https://eth-mainnet.g.alchemy.com/v2/{KEY}"),
            format!("https://user:{KEY}@eth-mainnet.g.alchemy.com/"),
            format!("https://eth-mainnet.g.alchemy.com/?apikey={KEY}"),
            format!("wss://eth-mainnet.g.alchemy.com/v2/{KEY}"),
        ] {
            let host = url_host(&url);
            assert_eq!(host.as_deref(), Some("eth-mainnet.g.alchemy.com"), "{url}");
        }
        assert_eq!(url_host("not a url"), None);
    }

    #[test]
    fn redact_url_removes_every_rendering_of_the_key() {
        let cases = [
            (
                format!("https://eth-mainnet.g.alchemy.com/v2/{KEY}"),
                format!(
                    "error sending request for url (https://eth-mainnet.g.alchemy.com/v2/{KEY})"
                ),
            ),
            (
                format!("https://user:{KEY}@rpc.example.com/"),
                format!("connect to https://user:{KEY}@rpc.example.com/ refused"),
            ),
            (
                format!("https://rpc.example.com/?apikey={KEY}&x=1"),
                format!("401 for \"https://rpc.example.com/?apikey={KEY}&x=1\""),
            ),
            (
                format!("https://rpc.example.com/v2/{KEY}"),
                format!("scheme-less rpc.example.com/v2/{KEY} failed"),
            ),
            (
                format!("https://rpc.example.com/v2/{KEY}"),
                format!("provider said: invalid api key {KEY}"),
            ),
            (
                "https://rpc.example.com/v2/key%20with%20spaces%20and%20more".to_string(),
                "decoded /v2/key with spaces and more leaked".to_string(),
            ),
        ];
        for (endpoint, message) in cases {
            let redacted = redact_url(&message, &endpoint);
            assert!(!redacted.contains(KEY), "{redacted}");
            assert!(!redacted.contains("spaces and more"), "{redacted}");
        }
    }

    #[test]
    fn redact_url_keeps_host_and_unrelated_text() {
        let endpoint = format!("https://eth-mainnet.g.alchemy.com/v2/{KEY}");
        let message = format!(
            "error sending request for url (https://eth-mainnet.g.alchemy.com/v2/{KEY}): connection refused"
        );
        assert_eq!(
            redact_url(&message, &endpoint),
            "error sending request for url (https://eth-mainnet.g.alchemy.com): connection refused"
        );
        let revert = "server returned an error response: error code 3: execution reverted, data: \"0x08c379a0\"";
        assert_eq!(redact_url(revert, &endpoint), revert);
    }

    #[test]
    fn redact_url_handles_unparseable_tokens_and_endpoints() {
        assert_eq!(
            redact_url("bad ://secret-thing here", "not a url"),
            "bad <redacted> here"
        );
        assert_eq!(redact_url("plain text", "not a url"), "plain text");
        assert_eq!(
            redact_url("é://x é https://h.example/p?q=1 é", "not a url"),
            "é<redacted> é https://h.example é"
        );
    }

    #[test]
    fn redact_url_removes_short_credentials_as_whole_tokens() {
        let password = redact_url(
            "authentication failed for password p4ss",
            "https://u:p4ss@rpc.example.com/",
        );
        assert_eq!(password, "authentication failed for password <redacted>");
        let api_key = redact_url(
            "invalid api key SYNTH-short",
            "https://rpc.example.com/?apikey=SYNTH-short",
        );
        assert_eq!(api_key, "invalid api key <redacted>");
        let embedded = redact_url("request passed", "https://u:pass@rpc.example.com/");
        assert_eq!(embedded, "request passed");
    }

    #[test]
    fn redact_url_leaves_parameter_names_and_route_segments() {
        let revert = "server returned an error response: error code 3: execution reverted, data: \"0xdeadbeefdeadbeef\"";
        assert_eq!(
            redact_url(revert, "https://rpc.example.com/?deadbeefdeadbeef=1"),
            revert
        );
        let endpoint = format!("https://rpc.ankr.com/arbitrum/{KEY}");
        assert_eq!(
            redact_url("arbitrum node unavailable", &endpoint),
            "arbitrum node unavailable"
        );
    }

    #[test]
    fn redact_url_keeps_repeated_content_around_a_short_credential() {
        let zeros = format!("execution reverted, data: \"0x{}\"", "0".repeat(64));
        assert_eq!(
            redact_url(&zeros, "https://user:0000@rpc.example.com/"),
            zeros
        );
        let repeated = "execution reverted, data: \"0xdeadbeefdeadbeef\"";
        assert_eq!(
            redact_url(repeated, "https://user:deadbeef@rpc.example.com/"),
            repeated
        );
        assert_eq!(
            redact_url(
                "bad: deadbeef deadbeef",
                "https://user:deadbeef@rpc.example.com/"
            ),
            "bad: <redacted> <redacted>"
        );
    }

    #[test]
    fn redact_url_keeps_a_route_only_path() {
        assert_eq!(
            redact_url(
                "route /arbitrum unavailable",
                "https://rpc.ankr.com/arbitrum"
            ),
            "route /arbitrum unavailable"
        );
    }

    #[test]
    fn percent_decode_tolerates_malformed_escapes() {
        assert_eq!(percent_decode("a%20b"), "a b");
        assert_eq!(percent_decode("100%"), "100%");
        assert_eq!(percent_decode("%zz%4"), "%zz%4");
        assert_eq!(percent_decode("é%C3%A9"), "éé");
    }
}
