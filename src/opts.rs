// @cladue
/// Option values to consume flags:
/// - -v or --verbose or --debug
/// - -q or --quiet or --batch
/// - -h or --help or --info
/// - -V or --version : print version and exit
///
/// The value could be pass or default as well,
/// e.g. --verbose=true or --verbose=false,
/// but the presence of the flag itself is enough to set it to true (--verbose), and absence means false.

#[derive(Debug, Clone)]
pub struct Opt {
    /// The name of the option
    pub name: String,
    /// The alternate long name of the option
    pub alt_name: String,
    pub short: String,
    /// The value of the option
    pub value: String,
    /// The default value of the option
    pub def_val: String,
    /// The value of the option was set
    pub set: bool,
}

impl Opt {
    pub fn new(name: &str, alt_name: &str, short: &str, def_val: &str) -> Self {
        Opt {
            name: name.to_string(),
            alt_name: alt_name.to_string(),
            short: short.to_string(),
            value: def_val.to_string(),
            def_val: def_val.to_string(),
            set: false,
        }
    }

    pub fn is_set(&self) -> bool {
        self.set
    }

    /// Match and apply `arg` to this option. Returns true if the arg matched.
    pub fn apply(&mut self, arg: &str) -> bool {
        let long = format!("--{}", self.name);
        let alt = if self.alt_name.is_empty() { String::new() } else { format!("--{}", self.alt_name) };
        let short = if self.short.is_empty() { String::new() } else { format!("-{}", self.short) };

        if arg == long
            || (!alt.is_empty() && arg == alt)
            || (!short.is_empty() && arg == short)
        {
            self.value = "true".to_string();
            self.set = true;
            return true;
        }

        let prefixes: Vec<String> = [format!("{}=", long)]
            .into_iter()
            .chain(if alt.is_empty() { None } else { Some(format!("{}=", alt)) })
            .collect();

        for prefix in &prefixes {
            if arg.starts_with(prefix.as_str()) {
                let v = &arg[prefix.len()..];
                self.set = v != "false" && !v.is_empty();
                self.value = v.to_string();
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opt_long_flag_test() {
        let mut opt = Opt::new("verbose", "debug", "v", "false");
        assert!(opt.apply("--verbose"));
        assert!(opt.is_set());
        assert_eq!(opt.value, "true");
    }

    #[test]
    fn opt_alt_flag_test() {
        let mut opt = Opt::new("verbose", "debug", "v", "false");
        assert!(opt.apply("--debug"));
        assert!(opt.is_set());
    }

    #[test]
    fn opt_short_flag_test() {
        let mut opt = Opt::new("verbose", "debug", "v", "false");
        assert!(opt.apply("-v"));
        assert!(opt.is_set());
    }

    #[test]
    fn opt_eq_true_test() {
        let mut opt = Opt::new("verbose", "debug", "v", "false");
        assert!(opt.apply("--verbose=true"));
        assert!(opt.is_set());
        assert_eq!(opt.value, "true");
    }

    #[test]
    fn opt_eq_false_test() {
        let mut opt = Opt::new("verbose", "debug", "v", "false");
        assert!(opt.apply("--verbose=false"));
        assert!(!opt.is_set());
        assert_eq!(opt.value, "false");
    }

    #[test]
    fn opt_no_match_test() {
        let mut opt = Opt::new("verbose", "debug", "v", "false");
        assert!(!opt.apply("--quiet"));
        assert!(!opt.is_set());
    }

    #[test]
    fn opt_default_unset_test() {
        let opt = Opt::new("quiet", "batch", "q", "false");
        assert!(!opt.is_set());
        assert_eq!(opt.value, "false");
        assert_eq!(opt.def_val, "false");
    }
}
