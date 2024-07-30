use crate::websocket::util;
use dirs;
use lazy_static::lazy_static;
use serde::Deserialize;
use std::error::Error;
use std::io::{stdin, stdout};
use std::{
    fs,
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
};
use toml;

lazy_static! {
    #[derive(Debug)]
    pub static ref DATA_LOCAL: PathBuf = Path::new(&dirs::data_local_dir().unwrap()).to_path_buf();
    pub static ref CONFIG_FILEPATH: PathBuf = Path::new(&DATA_LOCAL.join("afkttv/config.toml")).to_path_buf();

    #[derive(Debug)]
    pub static ref CONFIG_READER: Config = Config::read(&CONFIG_FILEPATH).unwrap();
}

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    pub authorization: Auth,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Auth {
    pub auth: String,
    pub user: String,
}

// instructions on the command line aren't AT ALL clear but this probably requires screenshots to
// make any sense at all
impl Config {
    pub fn write(filepath: &PathBuf) -> Result<String, Box<dyn Error>> {
        // this surely could be more concise i would imagine
        println!(
            "[{}] [-]: Unable to find TTV auth information at '{}'.",
            util::log_time(),
            // if we throw on any of these unwraps then its joever anyway
            &CONFIG_FILEPATH.to_str().unwrap()
        );

        let mut input_buff = String::new();

        // build the oauth config
        input_buff += "[authorization]\nauth = \"";
        print!(
            "[{}] [+] TTV oauth ('PASS oauth:[enter_this_string]): ",
            util::log_time()
        );
        stdout().flush().unwrap();

        // username section
        stdin().read_line(&mut input_buff)?;
        input_buff = format!("{}\"\nuser = \"", input_buff.trim_end());
        print!(
            "[{}] [+] TTV username ('NICK [enter_this_string]'): ",
            util::log_time()
        );
        stdout().flush().unwrap();

        stdin().read_line(&mut input_buff)?;
        input_buff = format!("{}\"\n", input_buff.trim_end());

        // write the input buffer content to a file for future use and return the entered string
        let _ = fs::write(filepath, &input_buff);
        return Ok(input_buff);
    }

    // reads the auth config from the configuration filepath, panicking if we can't parse the file
    // as a config
    pub fn read(filepath: &PathBuf) -> Result<Config, Box<dyn Error>> {
        // println!("FILEPATH => {}", filepath.to_string_lossy().to_string());
        let file = fs::read_to_string(filepath);
        let content = match file {
            Ok(content) => content,
            Err(err) => match err.kind() {
                ErrorKind::NotFound => match Config::write(filepath) {
                    Ok(fc) => fc,
                    Err(e) => {
                        // no file @ filepath, error returned from write fn
                        panic!("[-] Unable to create auth file: '{}'", e);
                    }
                },
                _ => {
                    panic!(
                        // found config @ filepath but couldnt read its content
                        "[-] Unable to read contents of file: '{}'",
                        filepath.to_str().unwrap()
                    );
                }
            },
        };

        let parsed: Config = toml::from_str(&content).map_err(|err| {
            eprintln!(
                "[-] Unable to parse file content into TOML structure: '{}'",
                filepath.to_str().unwrap()
            );

            Box::new(err) as Box<dyn Error>
        })?;

        return Ok(parsed);
    }
}
