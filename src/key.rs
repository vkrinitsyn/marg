
/**
Private key loaded from text file
*/
#[derive(Debug, Clone)]
pub struct KeyFile {
    /// file name
    pub file: String,
    pub value: Option<String>,
    /// The password
    pub passphrase: Option<String>,
}

// KeyFile::new checks the text it read; the tests check files with this
#[cfg(feature="key")]
#[allow(dead_code)]
pub(crate) fn is_key_file(file: &str) -> bool {
    /*
    const PKCS1B: &str = "-----BEGIN RSA PRIVATE KEY-----";
    const PKCS8B: &str = "-----BEGIN PRIVATE KEY-----";

    const PKCS1E: &str = "-----END RSA PRIVATE KEY-----";
    const PKCS8E: &str = "-----END PRIVATE KEY-----";
     */
    if let Ok(value) = std::fs::read_to_string(file) {
        is_key_text(&value)
    } else {
        false
    }
}

/// a PEM key: "-----BEGIN ...", then "-----END ... KEY-----" after it. No length floor
/// (an Ed25519 key is ~120 bytes) and text before it is fine (openssl "Bag Attributes")
#[cfg(feature="key")]
fn is_key_text(value: &str) -> bool {
    const HEAD: &str = "-----BEGIN ";
    const END: &str = "-----END ";
    const ENDS: &str = " KEY-----";
    value.find(HEAD)
        .and_then(|b| value[b..].find(END).map(|e| b + e))
        .is_some_and(|e| value[e..].contains(ENDS))
}

#[cfg(not(feature="key"))]
#[allow(dead_code)]
pub(crate) fn is_key_file(_file: &str) -> bool {
    false
}

impl KeyFile {
    #[cfg(not(feature="key"))]
    pub(crate) fn new(_file: String, _passphrase: Option<String>) -> Result<Option<Self>, String> {
        Ok(None)
    }

    /// filename must be readable and contains -----BEGIN
    ///
    /// pass if set and utf format
    #[cfg(feature="key")]
    pub(crate) fn new(file: String, passphrase: Option<String>) -> Result<Option<Self>, String> {
        if file.is_empty() {
            return Ok(None);
        }

        let value = std::fs::read_to_string(&file)
            .map_err(|e| format!("[{}] {}", file, e))?;
        // --key, env and the config file alike (before 0.4.6 only --key was checked)
        if !is_key_text(&value) {
            return Err(format!("[{}] not a key file", file));
        }

        Ok(Some(KeyFile {
            file,
            value: Some(value),
            passphrase,
        }))
    }
}

#[allow(warnings)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "key")]
    fn test() {
        let pk = KeyFile::new("".to_string(), None);
        assert!(pk.unwrap().is_none())
    }
}