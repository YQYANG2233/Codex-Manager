use super::should_skip_codex_v1_alt_for_responses;

#[test]
fn api_client_responses_request_skips_codex_v1_alt_retry() {
    assert!(should_skip_codex_v1_alt_for_responses(
        "https://chatgpt.com/backend-api/codex/v1/responses"
    ));
}

#[test]
fn native_codex_responses_request_skips_codex_v1_alt_retry() {
    assert!(should_skip_codex_v1_alt_for_responses(
        "https://chatgpt.com/backend-api/codex/v1/responses"
    ));
}

#[test]
fn non_responses_request_keeps_alternate_path_available() {
    assert!(!should_skip_codex_v1_alt_for_responses(
        "https://chatgpt.com/backend-api/codex/v1/chat/completions"
    ));
}

#[test]
fn compact_responses_mapped_to_chat_completions_keeps_alternate_path_available() {
    assert!(!should_skip_codex_v1_alt_for_responses(
        "https://chatgpt.com/backend-api/codex/v1/chat/completions"
    ));
}
