use clap::Parser;
use jsonwebtoken::{encode, EncodingKey, Header};
use nusavela_backend::routes::admin::Claims;
#[derive(Parser)]
struct Args {
    #[arg(long)]
    subject: String,
    #[arg(long, default_value_t = 900)]
    seconds: u64,
}
fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let a = Args::parse();
    anyhow::ensure!(
        !a.subject.trim().is_empty() && (1..=3600).contains(&a.seconds),
        "Subject and lifetime 1–3600 seconds required"
    );
    let secret = std::env::var("ADMIN_JWT_SECRET")?;
    anyhow::ensure!(secret.len() >= 32, "Secret too short");
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();
    let c = Claims {
        sub: a.subject,
        role: "admin".into(),
        exp: now + a.seconds,
        nbf: now,
        iss: std::env::var("ADMIN_JWT_ISSUER").unwrap_or_else(|_| "nusavela-admin".into()),
        aud: std::env::var("ADMIN_JWT_AUDIENCE").unwrap_or_else(|_| "nusavela-api".into()),
    };
    println!(
        "{}",
        encode(
            &Header::default(),
            &c,
            &EncodingKey::from_secret(secret.as_bytes())
        )?
    );
    Ok(())
}
