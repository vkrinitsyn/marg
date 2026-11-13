use base64::Engine;
use marg::ArgConfig;

#[cfg(feature="gen_secret")]
fn random_32_bytes_vec() -> Vec<u8> {
    let mut bytes = vec![0u8; 32];
    use rand::Rng; // rand 0.10: the old RngCore, fill_bytes with it
    let mut rng = rand::rng();
    rng.fill_bytes(&mut bytes);
    bytes
}
#[cfg(not(feature="gen_secret"))]
fn random_32_bytes_vec() -> Vec<u8> {
    println!("use gen_secret feature");
    vec![0u8; 32]
}

fn main() -> Result<(), String> {
    let cfg = ArgConfig::from_args2(env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))?;

    if let Some(secret) = &cfg.secret {
        let bytes = base64::prelude::BASE64_STANDARD.decode(secret)
            .map_err(|e| format!("the secret is not base64: {e}"))?;

        println!("Parsed:    {} [{}]", secret, bytes.len());
    } else {
        let bytes = random_32_bytes_vec();
        let secret = base64::prelude::BASE64_STANDARD.encode(&bytes);

        println!("Generated: {secret} [{}]", bytes.len());
    }
    Ok(())
}