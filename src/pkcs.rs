
/**
RSA Private key loaded from text file
*/
#[derive(Debug, Clone)]
pub struct Pk {
    /// file name
    pub file: String,
    pub value: Option<String>,
    /// The password
    pub passphrase: Option<String>,
}

#[cfg(feature="rsa")]
pub(crate) fn is_pk_file(file: &String) -> bool {
    const PKCS1B: &str = "-----BEGIN RSA PRIVATE KEY-----";
    const PKCS8B: &str = "-----BEGIN PRIVATE KEY-----";

    const PKCS1E: &str = "-----END RSA PRIVATE KEY-----";
    const PKCS8E: &str = "-----END PRIVATE KEY-----";


    if let Ok(value) = std::fs::read_to_string(file.as_str()) {
        value.len() > u8::MAX as usize && (
            (value.starts_with(PKCS1B)  && value.rfind(PKCS1E).is_some())
            || (value.starts_with(PKCS8B) && value.rfind(PKCS8E).is_some())
        )
    } else {
        false
    }
}

#[cfg(not(feature="rsa"))]
pub(crate) fn is_pk_file(_file: &String) -> bool {
    false
}

impl Pk {
    #[cfg(not(feature="rsa"))]
    pub(crate) fn new(_file: String, _passphrase: Option<String>, _rsa_file_def: Option<&str>) -> Result<Option<Self>, String> {
        return Ok(None)
    }

    /// filename must be readable and contains -----BEGIN
    ///
    /// pass if set and utf format
    #[cfg(feature="rsa")]
    pub(crate) fn new(rsa_file: String, passphrase: Option<String>, rsa_file_def: Option<&str>) -> Result<Option<Self>, String> {
        let file = if rsa_file.is_empty() {
            match rsa_file_def {
                None => "".to_string(),
                Some(file) => file.to_string()
            }
        } else {
            rsa_file
        };
        if file.is_empty() {
            return Ok(None);
        }

        let value = std::fs::read_to_string(file.as_str())
            .map_err(|e| format!("[{}] {}", file, e))?;

        Ok(Some(Pk{
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
    #[cfg(feature = "rsa")]
    fn test() {
        let pk = Pk::new("".to_string(), None, Some(""));
        assert!(pk.unwrap().is_none())
    }
}