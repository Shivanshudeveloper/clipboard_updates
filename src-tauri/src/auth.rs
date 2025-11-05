use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use reqwest;
use std::collections::HashMap;
use std::env;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
struct FirebaseClaims {
    pub aud: String,
    pub exp: usize,
    pub iat: usize,
    pub iss: String,
    pub sub: String,
    pub email: Option<String>,
     pub name: Option<String>,
}

pub async fn verify_firebase_token(id_token: &str) -> Result<(String, String, Option<String>), String> {
    // Load .env from project root (where Cargo.toml is)
    load_env_from_root();
    
    // let firebase_project_id = env::var("FIREBASE_PROJECT_ID")
        let firebase_project_id ="mealpro-development";

    println!("🔐 Verifying token for project: {}", firebase_project_id);

    // 1️⃣ Fetch Firebase public keys
    let jwks_url = "https://www.googleapis.com/robot/v1/metadata/x509/securetoken@system.gserviceaccount.com";
    let resp = reqwest::get(jwks_url)
        .await
        .map_err(|e| format!("Failed to fetch JWKS: {}", e))?
        .json::<HashMap<String, String>>()
        .await
        .map_err(|e| format!("Failed to parse JWKS JSON: {}", e))?;

    // 2️⃣ Decode header to get the key ID (kid)
    let header = decode_header(id_token)
        .map_err(|e| format!("Failed to decode token header: {}", e))?;
    let kid = header.kid.ok_or("Missing kid in token header")?;
    let cert_pem = resp.get(&kid).ok_or("No matching key found for token kid")?;

    // 3️⃣ Build decoding key
    let decoding_key = DecodingKey::from_rsa_pem(cert_pem.as_bytes())
        .map_err(|e| format!("Failed to create decoding key: {}", e))?;

    // 4️⃣ Verify the token
    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[firebase_project_id.clone()]);
    validation.set_issuer(&[format!(
        "https://securetoken.google.com/{}",
        firebase_project_id
    )]);

    let token_data = decode::<FirebaseClaims>(id_token, &decoding_key, &validation)
        .map_err(|e| format!("Token validation failed: {}", e))?;

    println!("✅ Token verified for user: {}", token_data.claims.sub);
      let email = token_data.claims.email.unwrap_or_else(|| {
        // Fallback: use UID-based email if no email in claims
        format!("{}@example.com", token_data.claims.sub)
    });
    
    // You might need to add name to your FirebaseClaims struct
   

    Ok((token_data.claims.sub, email,None))

    // Ok(token_data.claims.sub) // ✅ Firebase UID (unique user id)
}


/// Load environment variables from .env file in project root
fn load_env_from_root() {
    // Try multiple possible locations for the .env file
    let possible_paths = [
        ".env",
        "./.env",
        "../.env",
        "../../.env",
    ];
    
    for path in &possible_paths {
        if Path::new(path).exists() {
            println!("📁 Loading .env from: {}", path);
            if dotenv::from_filename(path).is_ok() {
                println!("✅ Successfully loaded .env from: {}", path);
                return;
            }
        }
    }
    
    println!("⚠️  No .env file found, using existing environment variables");
}

