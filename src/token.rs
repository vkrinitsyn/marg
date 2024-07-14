use std::process::Command;
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
            if !t.cmd.is_empty() {
                let _ = t.refresh()?;
            }
            Ok(t)
        }
        #[cfg(not(feature="token"))]
        Ok(t)
    }

    /// load password from running specified command
    /// check before call:
    ///
    /// `self.refreshed.elapsed() > self.ttl`
    #[cfg(feature="token")]
    pub fn refresh(&mut self) -> Result<Duration, String> {
        if self.cmd.is_empty() {
            return Err("Not configured".to_string());
        } else {
            let cmd:Vec<&str> = self.cmd.split(" ").collect();
            let mut c = Command::new(cmd[0]);
            if cmd.len() > 1 {
                c.args(&cmd[1..]);
            }
            match c.output() {
                Ok(out) => {
                    if out.status.success() {
                        let value = String::from_utf8(out.stdout).map_err(|e| format!("parsing output of {} {}", self.cmd, e))?;
                        self.value = Some(value);
                        self.refreshed = Instant::now();
                        Ok(self.ttl.clone())
                    } else {
                        let value = String::from_utf8(out.stderr).map_err(|e| format!("parsing error of {} {}", self.cmd, e))?;
                        Err(value)
                    }
                }
                Err(e) => {
                    Err(format!("[{}] {}", self.cmd, e))
                }
            }
        }
    }
}