use anyhow::Error;
use dirs;
use lazy_static::lazy_static;
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};
use toml;


lazy_static! {
    #[derive(Debug)]
    pub static ref DATA_LOCAL: PathBuf = Path::new(&dirs::data_local_dir().unwrap()).to_path_buf();
    pub static ref CONFIG_FILEPATH: PathBuf = Path::new(&DATA_LOCAL.join("afkttv/config.toml")).to_path_buf();
    pub static ref CONFIG_READER: Config = Config::read(&CONFIG_FILEPATH);
}

#[derive(Deserialize, Clone, Debug)]
pub struct Auth {
    pub auth: String,
    pub user: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    pub authorization: Auth,
}

impl Config {

    // writes the user's auth configuration to the (platform-agnostic) `$LOCALDATA/afkttv/config.toml`
    pub fn write(filepath: &PathBuf) -> Result<(), Error> {

        // do some term prompt stuff maybe - this is like a first-run type thing to provide the
        // user's details to the API
        //
        // for now we will just indicate intention and kill ourselves
        //
        eprintln!("[ERR] Unimplemented 'write()' function in the config reader - dump the `config.toml` into the filepath by hand for now.");
        eprintln!("[ERR] The filepath you want is: '{}'", &CONFIG_FILEPATH.to_string_lossy().to_string());
        unimplemented!();
    }

    // reads the auth config from the configuration filepath
    pub fn read(filepath: &PathBuf) -> Config {
        // do pattern matching and call `write()` so that we get user input and write the auth info to that
        // localdata toml file if it doesn't exist
        let content = fs::read_to_string(filepath).unwrap_or_else(|_| {
            eprintln!("[ERR] Unable to read contents of file: '{}'", filepath.to_string_lossy().to_string());
            panic!();
        });

        let parsed = toml::from_str(&content).unwrap_or_else(|_| {
            eprintln!(
                "[ERR] Unable to parse file content into TOML structure: '{}'",
                filepath.to_string_lossy().to_string()
            );
            panic!();
        });

        return parsed;
    }
}
