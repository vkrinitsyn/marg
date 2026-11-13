//! Runs the built `marg` binary (src/main.rs) as a child process.
//! Each test uses its own temp directory and passes env vars only to its child,
//! so tests never race on the parent's environment.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_marg");
/// base64 of 16 bytes of 0x01
const SECRET16: &str = "AQEBAQEBAQEBAQEBAQEBAQ==";

/// removes the temp directory when the test ends, also on failure
struct Dir(PathBuf);

impl Dir {
    fn new(name: &str) -> Dir {
        let d = std::env::temp_dir().join(format!("marg-cli-{}-{}", std::process::id(), name));
        std::fs::create_dir_all(&d).unwrap();
        Dir(d)
    }
    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Dir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn run(dir: &Path, args: &[&str], envs: &[(&str, &str)]) -> Output {
    let mut c = Command::new(BIN);
    c.current_dir(dir).args(args).env_remove("SECRET").env_remove("USER").env_remove("PGPASSWORD");
    for (k, v) in envs {
        c.env(k, v);
    }
    c.output().expect("run marg binary")
}

#[test]
fn version_flag_test() {
    let d = Dir::new("version");
    for flag in ["--version", "-V"] {
        let out = run(d.path(), &[flag], &[]);
        assert!(out.status.success());
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(stdout.contains(&format!("marg {}", env!("CARGO_PKG_VERSION"))));
        assert!(stdout.contains("DBinvent"));
    }
}

#[test]
#[cfg(feature = "gen_secret")]
fn generated_secret_test() {
    let d = Dir::new("generated");
    let out = run(d.path(), &[], &[]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.starts_with("Generated: "));
    assert!(stdout.trim_end().ends_with("[32]"));
}

#[test]
fn parsed_secret_flag_test() {
    let d = Dir::new("secret-flag");
    let out = run(d.path(), &["--secret", SECRET16], &[]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(stdout.trim_end(), format!("Parsed:    {} [16]", SECRET16));
}

#[test]
fn parsed_secret_env_test() {
    let d = Dir::new("secret-env");
    let out = run(d.path(), &[], &[("SECRET", SECRET16)]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(stdout.trim_end(), format!("Parsed:    {} [16]", SECRET16));
}

#[test]
fn file_config_flag_test() {
    let d = Dir::new("file-flag");
    // written under both names: before 0.4.6 the --file branch loaded a file named after the
    // flag ("file"), not after its value; file_config_flag_value_only_test has "x" alone
    for name in ["x", "file"] {
        std::fs::write(d.path().join(name), "ttl=5\nconfig=public.cfg\n").unwrap();
    }
    let out = run(d.path(), &["--file", "x"], &[]);
    assert!(out.status.success());
}

#[test]
fn file_config_missing_test() {
    let d = Dir::new("file-missing");
    let out = run(d.path(), &["--file", "x"], &[]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("No such file"));
}

#[test]
#[cfg(feature = "key")]
fn key_env_missing_file_test() {
    let d = Dir::new("key-missing");
    let out = run(d.path(), &[], &[("key", "/nonexistent-marg-cli/key.pem")]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("nonexistent-marg-cli"));
}

#[cfg(unix)]
#[test]
fn non_utf8_env_falls_back_test() {
    use std::os::unix::ffi::OsStrExt;
    let d = Dir::new("non-utf8");
    let out = Command::new(BIN)
        .current_dir(d.path())
        .args(["--secret", SECRET16])
        .env_remove("SECRET")
        .env("db", std::ffi::OsStr::from_bytes(b"\xff\xfe"))
        .output()
        .expect("run marg binary");
    assert!(out.status.success());
}

#[test]
fn pgpassword_env_test() {
    let d = Dir::new("pgpassword");
    let out = run(d.path(), &["--secret", SECRET16], &[("PGPASSWORD", "not-a-real-password")]);
    assert!(out.status.success());
}

// without gen_secret the binary says so and generates 32 zero bytes
#[test]
#[cfg(not(feature = "gen_secret"))]
fn generated_secret_without_feature_test() {
    let d = Dir::new("generated-off");
    let out = run(d.path(), &[], &[]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.starts_with("use gen_secret feature\n"));
    assert!(stdout.contains(&format!("Generated: {}= [32]", "A".repeat(43))));
}

// without the key feature a key file named in the env is never read: no error
#[test]
#[cfg(not(feature = "key"))]
fn key_env_ignored_without_feature_test() {
    let d = Dir::new("key-off");
    let out = run(d.path(), &[], &[("key", "/nonexistent-marg-cli/key.pem")]);
    assert!(out.status.success());
}

// --file reads the file named after it: only "x" exists, no file named "file"
#[test]
fn file_config_flag_value_only_test() {
    let d = Dir::new("file-value-only");
    std::fs::write(d.path().join("x"), "ttl=5\nconfig=public.cfg\n").unwrap();
    let out = run(d.path(), &["--file", "x"], &[]);
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
}

// the command line wins over the env: --secret, not SECRET
#[test]
fn secret_flag_beats_env_test() {
    let d = Dir::new("secret-both");
    let out = run(d.path(), &["--secret", SECRET16], &[("SECRET", "AgICAgICAgICAgICAgICAg==")]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(stdout.trim_end(), format!("Parsed:    {} [16]", SECRET16));
}

// a secret that is not base64 is an error, not a panic
#[test]
fn secret_not_base64_test() {
    let d = Dir::new("secret-bad");
    let out = run(d.path(), &["--secret", "abc!"], &[]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("the secret is not base64"), "{stderr}");
    assert!(!stderr.contains("panicked"), "{stderr}");
}

// a value flag at the end is an error, not ignored
#[test]
fn flag_without_value_test() {
    let d = Dir::new("no-value");
    let out = run(d.path(), &["--secret"], &[]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("--secret needs a value"), "{stderr}");
}

// with the key feature a key file named in the env is checked as one: not a key is an error
#[test]
#[cfg(feature = "key")]
fn key_env_not_a_key_test() {
    let d = Dir::new("key-env-notkey");
    let f = d.path().join("notkey.pem");
    std::fs::write(&f, "plain text, not a key").unwrap();
    let out = run(d.path(), &[], &[("key", f.to_str().unwrap())]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("not a key file"), "{stderr}");
}

// an empty SECRET is an error when --secret is not given; with --secret the flag wins
#[test]
fn empty_secret_env_test() {
    let d = Dir::new("secret-empty");
    let out = run(d.path(), &[], &[("SECRET", "")]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("SECRET is empty"), "{stderr}");
    let out = run(d.path(), &["--secret", SECRET16], &[("SECRET", "")]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(stdout.trim_end(), format!("Parsed:    {} [16]", SECRET16));
}

// --version after a value flag is not a version request: the flag has no value
#[test]
fn version_as_flag_value_test() {
    let d = Dir::new("version-value");
    for flag in ["--version", "-V"] {
        let out = run(d.path(), &["--db", flag], &[]);
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(!stdout.contains("DBinvent"), "{flag}: {stdout}");
    }
    let out = run(d.path(), &["--db", "--version"], &[]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("--db needs a value"), "{stderr}");
    // after a value flag's value, it is one
    let out = run(d.path(), &["--db=postgresql://h/d", "--version"], &[]);
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("DBinvent"));
}
