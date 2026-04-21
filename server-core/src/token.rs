use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub fn sign_payload(payload: &[u8], secret: &[u8]) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(secret)
        .expect("HMAC can take key of any size");

    mac.update(payload);

    let signature = mac.finalize().into_bytes();

    let mut token = Vec::with_capacity(payload.len() + signature.len());

    token.extend_from_slice(payload);
    token.extend_from_slice(&signature);

    token
}

pub fn verify_token(token: &[u8], secret: &[u8]) -> Option<Vec<u8>> {
    const SIG_LEN: usize = 32;

    if token.len() < SIG_LEN {
        return None;
    }

    let payload_len = token.len() - SIG_LEN;

    let payload = &token[..payload_len];
    let signature = &token[payload_len..];

    let mut mac = HmacSha256::new_from_slice(secret).ok()?;
    mac.update(payload);

    mac.verify_slice(signature).ok()?;

    Some(payload.to_vec())
}

fn load_secret(hex_key: &str) -> Vec<u8> {
    let key = hex::decode(hex_key).expect("invalid hex key");
    assert!(
        key.len() >= 32,
        "HMAC key should be at least 32 bytes"
    );
    key
}

pub fn expiration_time(validity_seconds: u64) -> u64 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();
    now.as_secs() + validity_seconds
}

pub fn is_token_expired(exp: u64) -> bool {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();
    now.as_secs() > exp
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Serialize, Deserialize};

    #[derive(Serialize, Deserialize, Debug)]
    struct TokenPayload {
        user_id: String,
        exp: u64,
    }

    #[test]
    fn test_token() {
        let hex_secret = "b7c6c9a50a6e5d3b8a16f7d92c0fd4a7d4c7ef24c9b3d7c87b13e6e8f3c2a6d1";
        let secret = load_secret(hex_secret);

        let payload = "MyPayload".as_bytes();
        let token = sign_payload(&payload, &secret);
        let payload2 = verify_token(&token, &secret);
        assert!(payload2.is_some());
        assert_eq!(payload, payload2.unwrap().as_slice());      
    }

     #[test]
    fn test_token2() {
        let hex_secret = "b7c6c9a50a6e5d3b8a16f7d92c0fd4a7d4c7ef24c9b3d7c87b13e6e8f3c2a6d1";
        let secret = load_secret(hex_secret);

        let token_payload = TokenPayload {
            user_id: "user123".to_string(),
            exp: expiration_time(300),
        };
        let payload = postcard::to_stdvec(&token_payload).unwrap();

        let token = sign_payload(&payload, &secret);
        let payload2 = verify_token(&token, &secret);
        assert!(payload2.is_some());
        let response: TokenPayload = postcard::from_bytes(payload2.unwrap().as_slice()).unwrap();
        assert_eq!(token_payload.user_id, response.user_id);
        assert_eq!(token_payload.exp, response.exp);
        assert!(!is_token_expired(response.exp));

        let secret2 = load_secret("b7c6c9a50a6e5d3b8a16f7d92c0fd4a7d4c7ef24c9b3d7c87b13e6e8f3c2a6d2");
        let token2  = sign_payload(&payload, &secret2);
        let payload3 = verify_token(&token2, &secret);
        assert!(payload3.is_none());

        let payload4 = verify_token(&token, &secret2);
        assert!(payload4.is_none());
    }
}


