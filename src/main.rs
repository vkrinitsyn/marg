use base64::Engine;
use marg::ArgConfig;

#[cfg(feature="gen_secret")]
fn random_32_bytes_vec() -> Vec<u8> {
    let mut bytes = vec![0u8; 32];
    let mut rng = rand::RngCore::rng();
    rng.fill_bytes(&mut bytes);
    bytes
}
#[cfg(not(feature="gen_secret"))]
fn random_32_bytes_vec() -> Vec<u8> {
    println!("use gen_secret feature");
    vec![0u8; 32]
}

fn main() -> Result<(), String> {
    let cfg = ArgConfig::from_args()?;

    if cfg.secret.is_none() {
        let bytes = random_32_bytes_vec();
        let secret = base64::prelude::BASE64_STANDARD.encode(&bytes);

        println!("Generated: {secret} [{}]", bytes.len());
    } else {
        let bytes = base64::prelude::BASE64_STANDARD.decode(cfg.secret.as_ref().unwrap()).unwrap();

        println!("Parsed:    {} [{}]", cfg.secret.unwrap(), bytes.len());
    }
    Ok(())
}