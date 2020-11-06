use toml::Value;
use std::fs;
use std::path::PathBuf;

pub struct Definitions {}

impl Definitions {
    pub fn load(toml_path: PathBuf) {
        let toml_string = fs::read_to_string(toml_path).unwrap();
        let toml: Value = toml::from_str(&toml_string).unwrap();

        println!("Toml: {:#?}", toml);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_load() {
        let path = PathBuf::from(format!("{}{}", env!("CARGO_MANIFEST_DIR"), "/schema/tes5.toml"));
        Definitions::load(path);
        assert!(true);
    }
}