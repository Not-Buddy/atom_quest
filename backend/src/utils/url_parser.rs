use regex::Regex;

/// Extracts GitHub username from various GitHub URL formats
/// Supports:
/// - https://github.com/username
/// - http://github.com/username
/// - github.com/username
/// - https://www.github.com/username
pub fn parse_github_url(url: &str) -> Option<String> {
    let re = Regex::new(r"(?:https?://)?(?:www\.)?github\.com/([a-zA-Z0-9_-]+)/?").ok()?;
    re.captures(url)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
}

/// Extracts LeetCode username from various LeetCode URL formats
/// Supports:
/// - https://leetcode.com/u/username
/// - https://leetcode.com/username
/// - http://leetcode.com/u/username
/// - leetcode.com/u/username
pub fn parse_leetcode_url(url: &str) -> Option<String> {
    let re = Regex::new(r"(?:https?://)?(?:www\.)?leetcode\.com/(?:u/)?([a-zA-Z0-9_-]+)/?").ok()?;
    re.captures(url)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
}

/// Validates and normalizes LinkedIn URL
/// Keeps the full URL but validates it's a proper LinkedIn profile
pub fn validate_linkedin_url(url: &str) -> Option<String> {
    let re = Regex::new(r"(?:https?://)?(?:www\.)?linkedin\.com/in/[a-zA-Z0-9_-]+/?").ok()?;
    if re.is_match(url) {
        // Normalize URL to include https://
        if url.starts_with("http://") || url.starts_with("https://") {
            Some(url.to_string())
        } else {
            Some(format!("https://{}", url))
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_url() {
        assert_eq!(
            parse_github_url("https://github.com/Not-Buddy"),
            Some("Not-Buddy".to_string())
        );
        assert_eq!(
            parse_github_url("github.com/Not-Buddy"),
            Some("Not-Buddy".to_string())
        );
        assert_eq!(
            parse_github_url("https://www.github.com/Not-Buddy/"),
            Some("Not-Buddy".to_string())
        );
    }

    #[test]
    fn test_parse_leetcode_url() {
        assert_eq!(
            parse_leetcode_url("https://leetcode.com/u/not_buddy/"),
            Some("not_buddy".to_string())
        );
        assert_eq!(
            parse_leetcode_url("leetcode.com/not_buddy"),
            Some("not_buddy".to_string())
        );
    }

    #[test]
    fn test_validate_linkedin_url() {
        assert_eq!(
            validate_linkedin_url("https://www.linkedin.com/in/aary-k-a77499240/"),
            Some("https://www.linkedin.com/in/aary-k-a77499240/".to_string())
        );
        assert_eq!(
            validate_linkedin_url("linkedin.com/in/aary-k-a77499240"),
            Some("https://linkedin.com/in/aary-k-a77499240".to_string())
        );
    }
}


/// Parse CodeChef profile URL and extract username
/// Handles formats:
/// - https://www.codechef.com/users/username
/// - https://codechef.com/users/username
pub fn parse_codechef_url(url: &str) -> Option<String> {
    let url = url.trim();
    
    // Try to parse as URL
    if let Ok(parsed_url) = url::Url::parse(url)
        && let Some(host) = parsed_url.host_str()
            && host.contains("codechef.com") {
                let path = parsed_url.path();
                // Extract username from /users/username
                if let Some(username) = path.strip_prefix("/users/") {
                    let username = username.trim_end_matches('/');
                    if !username.is_empty() {
                        return Some(username.to_string());
                    }
                }
            }
    
    None
}

/// Parse Codeforces profile URL and extract username
/// Handles formats:
/// - https://codeforces.com/profile/username
/// - https://www.codeforces.com/profile/username
pub fn parse_codeforces_url(url: &str) -> Option<String> {
    let url = url.trim();
    
    // Try to parse as URL
    if let Ok(parsed_url) = url::Url::parse(url)
        && let Some(host) = parsed_url.host_str()
            && host.contains("codeforces.com") {
                let path = parsed_url.path();
                // Extract username from /profile/username
                if let Some(username) = path.strip_prefix("/profile/") {
                    let username = username.trim_end_matches('/');
                    if !username.is_empty() {
                        return Some(username.to_string());
                    }
                }
            }
    
    None
}
