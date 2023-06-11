pub(crate) struct Config {
  pub(crate) config_path: String,
}

impl Config {
  pub(crate) fn new() -> Self {
    Self {
      config_path: get_config_path(),
    }
  }
}

// Get Config File path from XDG_CONFIG_HOME
fn get_config_path() -> String {
  let mut config_path = String::new();
  match std::env::var("XDG_CONFIG_HOME") {
    Ok(path) => config_path.push_str(&path),
    Err(_) => match std::env::var("HOME") {
      Ok(path) => {
        config_path.push_str(&path);
        config_path.push_str("/.config");
      }
      Err(_) => {
        panic!("Could not find XDG_CONFIG_HOME or HOME");
      }
    },
  }
  config_path.push_str("/popi");
  config_path
}
