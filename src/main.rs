use marg::ArgConfig;

#[cfg(not(feature="rsa"))]
fn main() {
    let _test_run = ArgConfig::from_args();
}
#[cfg(feature="rsa")]
fn main() {
    let _test_run = ArgConfig::from_args(None);
}