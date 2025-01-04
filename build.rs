use directories::UserDirs;

fn main() {
    if let Some(user_dirs) = UserDirs::new() {
        let path = user_dirs.home_dir().join(".config/wot/config.json");
        if !path.exists() {
            let config = r#"{
  "testops_base_api_url": "",
  "testops_base_url": "",
  "testops_api_token": "",
}
"#;
            std::fs::write(path, config).expect("Не смогли создать конфиг")
        }
    }
}
