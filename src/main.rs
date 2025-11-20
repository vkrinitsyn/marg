use marg::ArgConfig;

fn main() -> Result<(), String> {
    let cfg = ArgConfig::from_args()?;
    debug_assert!(cfg.secret.is_some());

    Ok(())
}