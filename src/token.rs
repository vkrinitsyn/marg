use std::time::{Duration, Instant};

/**
Token is actually a temporary password obtained from another app run
*/
#[derive(Debug, Clone)]
pub struct Token {
    /// time of taken
    pub refreshed: Instant,
    /// Time to live
    pub min: u16,
    /// Time to live
    pub ttl: Duration,
    /// shell command to run like this:
    /// az account get-access-token --resource-type oss-rdbms | jq -r .accessToken
    pub cmd: String,
    /// The password
    pub value: Option<String>,
}

impl Token {
    pub(crate) fn new(cmd: String, ttl: String, pwd: Option<String>) -> Result<Self, String> {
        let min = ttl.parse::<u16>().unwrap_or(60);
        let t = Token {
            refreshed: Instant::now(),
            ttl: Duration::from_secs(min as u64 * 60),
            min,
            cmd,
            value: pwd,
        };
        #[cfg(feature="token")]
        {
            let mut t = t;
            let _ = t.refresh()?;
            Ok(t)
        }
        #[cfg(not(feature="token"))]
        Ok(t)
    }

    /// load password from running specified command
    #[cfg(feature="token")]
    pub fn refresh(&mut self) -> Result<Duration, String> {

        unimplemented!()
    }
}