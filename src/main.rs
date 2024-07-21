use marg::ArgConfig;

#[cfg(not(feature="key"))]
fn main() {
    let _test_run = ArgConfig::from_args();
}
#[cfg(feature="key")]
fn main() {
    let _test_run = ArgConfig::from_args(None);
}