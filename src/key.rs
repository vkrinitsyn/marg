
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

#[cfg(feature="key")]
pub(crate) fn is_key_file(file: &String) -> bool {
    /*
    const PKCS1B: &str = "-----BEGIN RSA PRIVATE KEY-----";
    const PKCS8B: &str = "-----BEGIN PRIVATE KEY-----";

    const PKCS1E: &str = "-----END RSA PRIVATE KEY-----";
    const PKCS8E: &str = "-----END PRIVATE KEY-----";
     */
    const HEAD: &str = "-----BEGIN ";
    const ENDS: &str = " KEY-----";


    if let Ok(value) = std::fs::read_to_string(file.as_str()) {
        value.len() > u8::MAX as usize && value.starts_with(HEAD) && value.rfind(ENDS).is_some()
    } else {
        false
    }
}

#[cfg(not(feature="key"))]
pub(crate) fn is_key_file(_file: &String) -> bool {
    false
}

impl KeyFile {
    #[cfg(not(feature="key"))]
    pub(crate) fn new(_file: String, _passphrase: Option<String>, _file_def: Option<&str>) -> Result<Option<Self>, String> {
        return Ok(None)
    }

    /// filename must be readable and contains -----BEGIN
    ///
    /// pass if set and utf format
    #[cfg(feature="key")]
    pub(crate) fn new(key_file: String, passphrase: Option<String>, key_file_def: Option<&str>) -> Result<Option<Self>, String> {
        let file = if key_file.is_empty() {
            match key_file_def {
                None => "".to_string(),
                Some(file) => file.to_string()
            }
        } else {
            key_file
        };
        if file.is_empty() {
            return Ok(None);
        }

        let value = std::fs::read_to_string(file.as_str())
            .map_err(|e| format!("[{}] {}", file, e))?;

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
        let pk = KeyFile::new("".to_string(), None, Some(""));
        assert!(pk.unwrap().is_none())
    }
}