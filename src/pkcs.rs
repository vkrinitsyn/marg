use std::fs;

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
    unimplemented!()
}

#[cfg(not(feature="rsa"))]
pub(crate) fn is_pk_file(_file: &String) -> bool {
    false
}

impl Pk {
    #[cfg(not(feature="rsa"))]
    pub(crate) fn new(_file: String, _passphrase: Option<String>) -> Result<Option<Self>, String> {
        return Ok(None)
    }

    /// filename must be readable and contains -----BEGIN
    ///
    /// pass if set and utf format
    #[cfg(feature="rsa")]
    pub(crate) fn new(file: String, passphrase: Option<String>) -> Result<Option<Self>, String> {

        let value = fs::read_to_string(file.as_str()).map_err(|e| e.to_string())?;

        Ok(Some(Pk{
            file,
            value: Some(value),
            passphrase,
        }))
    }
}