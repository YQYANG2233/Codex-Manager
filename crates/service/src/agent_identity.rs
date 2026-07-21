use std::collections::BTreeMap;

use base64::engine::general_purpose::{STANDARD as BASE64_STANDARD, URL_SAFE_NO_PAD};
use base64::Engine as _;
use chrono::{SecondsFormat, Utc};
use codexmanager_core::storage::AccountAgentIdentity;
use ed25519_dalek::pkcs8::DecodePrivateKey;
use ed25519_dalek::{Signer as _, SigningKey};

const AGENT_ASSERTION_SCHEME: &str = "AgentAssertion";

pub(crate) fn authorization_header_for_agent_identity(
    identity: &AccountAgentIdentity,
) -> Result<String, String> {
    let timestamp = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    authorization_header_for_agent_identity_at(identity, &timestamp)
}

pub(crate) fn validate_agent_identity(identity: &AccountAgentIdentity) -> Result<(), String> {
    required_value(&identity.agent_runtime_id, "agent_runtime_id")?;
    required_value(&identity.task_id, "task_id")?;
    signing_key_from_pkcs8_base64(&identity.agent_private_key)?;
    Ok(())
}

fn authorization_header_for_agent_identity_at(
    identity: &AccountAgentIdentity,
    timestamp: &str,
) -> Result<String, String> {
    let runtime_id = required_value(&identity.agent_runtime_id, "agent_runtime_id")?;
    let task_id = required_value(&identity.task_id, "task_id")?;
    let signing_key = signing_key_from_pkcs8_base64(&identity.agent_private_key)?;
    let signed_payload = format!("{runtime_id}:{task_id}:{timestamp}");
    let signature = BASE64_STANDARD.encode(signing_key.sign(signed_payload.as_bytes()).to_bytes());
    let envelope = BTreeMap::from([
        ("agent_runtime_id", runtime_id),
        ("signature", signature.as_str()),
        ("task_id", task_id),
        ("timestamp", timestamp),
    ]);
    let serialized = serde_json::to_vec(&envelope)
        .map_err(|err| format!("failed to serialize agent assertion: {err}"))?;
    Ok(format!(
        "{AGENT_ASSERTION_SCHEME} {}",
        URL_SAFE_NO_PAD.encode(serialized)
    ))
}

pub(crate) fn format_upstream_authorization(auth_token: &str) -> String {
    let trimmed = auth_token.trim();
    if trimmed.starts_with(&format!("{AGENT_ASSERTION_SCHEME} ")) {
        trimmed.to_string()
    } else {
        format!("Bearer {trimmed}")
    }
}

fn required_value<'a>(value: &'a str, field: &str) -> Result<&'a str, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        Err(format!("agent identity {field} is empty"))
    } else {
        Ok(trimmed)
    }
}

fn signing_key_from_pkcs8_base64(private_key: &str) -> Result<SigningKey, String> {
    let private_key = BASE64_STANDARD
        .decode(private_key.trim())
        .map_err(|err| format!("agent identity private key is not valid base64: {err}"))?;
    SigningKey::from_pkcs8_der(&private_key)
        .map_err(|err| format!("agent identity private key is not valid PKCS#8: {err}"))
}

#[cfg(test)]
mod tests {
    use base64::engine::general_purpose::{STANDARD as BASE64_STANDARD, URL_SAFE_NO_PAD};
    use base64::Engine as _;
    use codexmanager_core::storage::now_ts;
    use ed25519_dalek::pkcs8::EncodePrivateKey;
    use ed25519_dalek::{Signature, SigningKey, Verifier as _};

    use super::*;

    fn identity() -> (AccountAgentIdentity, SigningKey) {
        let signing_key = SigningKey::from_bytes(&[7_u8; 32]);
        let private_key = signing_key.to_pkcs8_der().expect("encode private key");
        let now = now_ts();
        (
            AccountAgentIdentity {
                account_id: "account-1".to_string(),
                agent_runtime_id: "agent-runtime-1".to_string(),
                agent_private_key: BASE64_STANDARD.encode(private_key.as_bytes()),
                task_id: "task-1".to_string(),
                chatgpt_user_id: "user-1".to_string(),
                chatgpt_account_is_fedramp: false,
                auth_mode: "agentIdentity".to_string(),
                workspace_id: Some("workspace-1".to_string()),
                created_at: now,
                updated_at: now,
            },
            signing_key,
        )
    }

    #[test]
    fn agent_assertion_matches_codex_agent_identity_wire_format() {
        let (identity, signing_key) = identity();
        let timestamp = "2026-07-21T12:00:00Z";
        let header = authorization_header_for_agent_identity_at(&identity, timestamp)
            .expect("build agent assertion");
        let encoded = header
            .strip_prefix("AgentAssertion ")
            .expect("agent assertion scheme");
        let envelope: serde_json::Value =
            serde_json::from_slice(&URL_SAFE_NO_PAD.decode(encoded).expect("decode assertion"))
                .expect("parse assertion");

        assert_eq!(envelope["agent_runtime_id"], "agent-runtime-1");
        assert_eq!(envelope["task_id"], "task-1");
        assert_eq!(envelope["timestamp"], timestamp);
        let signature_bytes = BASE64_STANDARD
            .decode(envelope["signature"].as_str().expect("signature"))
            .expect("decode signature");
        let signature = Signature::from_slice(&signature_bytes).expect("parse signature");
        signing_key
            .verifying_key()
            .verify(
                format!("agent-runtime-1:task-1:{timestamp}").as_bytes(),
                &signature,
            )
            .expect("verify signature");
    }

    #[test]
    fn upstream_authorization_preserves_agent_assertion_and_wraps_bearer() {
        assert_eq!(
            format_upstream_authorization("AgentAssertion encoded"),
            "AgentAssertion encoded"
        );
        assert_eq!(
            format_upstream_authorization("access-token"),
            "Bearer access-token"
        );
    }
}
