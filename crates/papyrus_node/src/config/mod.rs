#[cfg(test)]
mod config_test;

mod file_config;


use std::collections::HashMap;
use std::mem::discriminant;
use std::path::PathBuf;
use std::time::Duration;
use std::{env, fs, io};
use serde_json::value::Value;

use clap::{arg, value_parser, Arg, ArgMatches, Command, ArgAction};
use file_config::FileConfigFormat;
use papyrus_gateway::GatewayConfig;
use papyrus_monitoring_gateway::MonitoringGatewayConfig;
use papyrus_storage::db::DbConfig;
use papyrus_storage::StorageConfig;
use papyrus_sync::{CentralSourceConfig, SyncConfig};
use serde::{Deserialize, Serialize};
use starknet_api::core::ChainId;
use starknet_client::RetryConfig;

use crate::version::VERSION_FULL;

lazy_static! { static ref CONFIG: Config = ConfigBuilder::build(env::args().collect()).unwrap(); 
    static ref CONFIGL: HashMap<String, Option<Value>> = BuilderConfig::build().unwrap();
            }

// The path of the default configuration file, provided as part of the crate.
const CONFIG_FILE: &str = "config/default.yaml";

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ConfigAttr {
    pub default: Option<Value>,
    pub short: Option<char>,
    pub long: Option<String>,
    pub env: Option<String>,
    pub description: Option<String>
}

#[derive(Default)]
pub struct BuilderConfig {
    config_norm: HashMap<String, ConfigAttr>,
    cla_config: ArgMatches,
    env_config: HashMap<String, Value>,
    file_config: HashMap<String, Option<Value>>
}

#[test]
fn this_this(){
    let yaml_path = "config/default.yaml";
    //let args = self.args.clone().expect("Config builder should have args.");
    //if let Some(config_file) = args.try_get_one::<PathBuf>("config_file")? {
    //    yaml_path =
    //        config_file.to_str().ok_or(ConfigError::BadPath { path: config_file.clone() })?    ;
    //}

    let config = BuilderConfig::build();

    match config {
        Ok(content)=> {println!("{content:?}");}
        Err(err) => {println!("{err:?}");}
    }

    let configl = ConfigL::new();
    println!("{configl:?}");
    // let yaml_contents = fs::read_to_string(yaml_path);
    // match yaml_contents{
    //     Ok(contents) => {
    //         let from_yaml: Result<HashMap<String, ConfigAttr>, _> = serde_yaml::from_str(&contents);
    //         println!("{from_yaml:?}");
    //     }
    //     Err(err) => {
    //         println!("err");
    //     }
    // }
}

impl BuilderConfig {
    fn load_norm(mut self) -> Result<Self, ConfigError> {
        let yaml_contents = fs::read_to_string("/Users/miller/papyrus/config/default.yaml")?;
        self.config_norm = serde_yaml::from_str(&yaml_contents)?;
        Ok(self)
    }

    fn load_env(mut self) -> Result<Self, ConfigError> {
        self.env_config = HashMap::new();
        for (k, v) in self.config_norm.iter() {
            if let Some(env) = &v.env {
                let env_value = env::var(env);
                if let Ok(value) = env_value {
                    self.env_config.insert(k.clone(), serde_json::json!(value));
                }  
            }
        }
        Ok(self)
    }

    fn load_cla(mut self) -> Result<Self, ConfigError> {
        let mut _args = Command::new("Papyrus",)
               .version(VERSION_FULL)
               .about("Papyrus is a StarkNet full node written in Rust.");
            //    .args(&[arg!(--cconfig_file [cconfig_file] "cconfig file")]).try_get_matches_from(env::args().into_iter())?;
        for (k, v) in self.config_norm.iter() {
            let mut _arg = Arg::new(k.as_str())
                                    .action(ArgAction::Set);
            if let Some(short) = &v.short {
                _arg = _arg.short(*short);
            }
            if let Some(long) = &v.long {
                _arg = _arg.long(long.as_str());
            } 
            if let Some(env) = &v.env {
                _arg = _arg.env(env);
            }
            _args = _args.arg(_arg);
        }
        // _args = _args.arg(arg!(--cconfig_file [cconfig_file] "cconfig file"));
        self.cla_config = _args.try_get_matches_from(env::args().into_iter())?;
        Ok(self)
    }

    fn load_file(mut self) -> Result<Self, ConfigError> {
        self.file_config = HashMap::new();
        let config_file: Option<&Value> = self.env_config.get("config_file")
                                                          .or(self.cla_config.try_get_one::<Value>("config_file").ok().flatten())
                                                          .or(self.config_norm.get("config_file").map(|v|v.default.as_ref()).flatten());
        if let Some(file_path) = config_file {
            let yaml_contents = fs::read_to_string(file_path.as_str().unwrap())?;
            self.file_config = serde_yaml::from_str(&yaml_contents)?; 
        }                                                                                        
        Ok(self)
    }

    fn build() -> Result<HashMap<String, Option<Value>>, ConfigError> {
        let builder = Self::default().load_norm()?.load_env()?.load_cla()?.load_file()?;
        let mut config: HashMap<String, Option<Value>> = HashMap::new();
        for (k, v) in builder.config_norm.iter() {
            println!("{k:?}");
            let value: Option<&Value> = builder.env_config.get(k)
                                             .or(builder.cla_config.try_get_one::<Value>(k).ok().flatten())
                                             .or(builder.file_config.get(k).map(|v|v.as_ref().unwrap()))
                                             .or(v.default.as_ref());
            config.insert(k.clone(),value.map(|v|v.clone()));
        }
        Ok(config)
    }
}

/// The configurations of the various components of the node.
#[derive(Deserialize, Serialize, Clone)]
pub struct Config {
    pub gateway: GatewayConfig,
    pub central: CentralSourceConfig,
    pub monitoring_gateway: MonitoringGatewayConfig,
    pub storage: StorageConfig,
    /// None if the syncing should be disabled.
    pub sync: Option<SyncConfig>,
}

#[derive(Debug)]
#[derive(ConfigDerive)]
pub struct RetryConfigL {
    pub chain_id: Value,
    pub server_address: Value,
    pub max_events_chunk_size: Value,
    pub max_events_keys: Value,
}

#[derive(Debug)]
#[derive(ConfigDerive)]
pub struct CentralSourceConfigL {
    pub concurrent_requests: Value,
    pub url: Value,
    pub retry_config: RetryConfigL,
}

#[derive(Debug, ConfigDerive)]
pub struct GatewayConfigL {
    pub chain_id: Value,
    pub server_address: Value,
    pub max_events_chunk_size: Value,
    pub max_events_keys: Value,
}

#[derive(Debug, ConfigDerive)]
pub struct StorageConfigL {
    pub db_config: DbConfigL,
}

#[derive(Debug, ConfigDerive)]
pub struct DbConfigL {
    pub path: Value,
    pub min_size: Value,
    pub max_size: Value,
    pub growth_step: Value,
}

#[derive(Debug, ConfigDerive)]
pub struct ConfigL {
    pub path: Value,
    pub gateway: GatewayConfigL,
    pub central: CentralSourceConfigL,
    pub storage: StorageConfigL,
}

impl Config {
    pub fn load(args: Vec<String>) -> Result<Self, ConfigError> {
        ConfigBuilder::build(args)
    }

    pub fn get_config_representation(&self) -> Result<serde_json::Value, ConfigError> {
        Ok(serde_json::to_value(FileConfigFormat::from(self.clone()))?)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("Unable to parse path: {path}")]
    BadPath { path: PathBuf },
    #[error(transparent)]
    Clap(#[from] clap::Error),
    #[error(transparent)]
    Matches(#[from] clap::parser::MatchesError),
    #[error(transparent)]
    Read(#[from] io::Error),
    #[error(transparent)]
    Serde(#[from] serde_yaml::Error),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(
        "CLA http_header \"{illegal_header}\" is not valid. The Expected format is name:value"
    )]
    CLAHttpHeader { illegal_header: String },
}

// Builds the configuration for the node based on default values, yaml configuration file and
// command-line arguments.
// TODO: add configuration from env variables.
pub(crate) struct ConfigBuilder {
    args: Option<ArgMatches>,
    chain_id: ChainId,
    config: Config,
}

//pub(crate) struct ConfigBuilderLL {
//    config: ConfigLL
//}
//
//impl ConfigBuilderLL {
//    pub fn apply default(mut self) -> Self {
//        self.config.chain_id = CONFIG.chain_id;
//        
//    }
//}

// Default configuration values.
// TODO: Consider implementing Default for each component individually.
impl Default for ConfigBuilder {
    fn default() -> Self {
        let chain_id = ChainId(String::from("SN_MAIN"));
        ConfigBuilder {
            args: None,
            chain_id: chain_id.clone(),
            config: Config {
                central: CentralSourceConfig {
                    concurrent_requests: 300,
                    url: String::from("https://alpha-mainnet.starknet.io/"),
                    http_headers: None,
                    retry_config: RetryConfig {
                        retry_base_millis: 30,
                        retry_max_delay_millis: 30000,
                        max_retries: 10,
                    },
                },
                gateway: GatewayConfig {
                    chain_id,
                    server_address: String::from("0.0.0.0:8080"),
                    max_events_chunk_size: 1000,
                    max_events_keys: 100,
                },
                monitoring_gateway: MonitoringGatewayConfig {
                    server_address: String::from("0.0.0.0:8081"),
                },
                storage: StorageConfig {
                    db_config: DbConfig {
                        path: PathBuf::from("./data"),
                        min_size: 1 << 20,    // 1MB
                        max_size: 1 << 40,    // 1TB
                        growth_step: 1 << 26, // 64MB
                    },
                },
                sync: Some(SyncConfig {
                    block_propagation_sleep_duration: Duration::from_secs(10),
                    recoverable_error_sleep_duration: Duration::from_secs(10),
                    blocks_max_stream_size: 1000,
                    state_updates_max_stream_size: 1000,
                }),
            },
        }
    }
}

impl ConfigBuilder {
    // Creates the configuration struct.
    fn build(args: Vec<String>) -> Result<Config, ConfigError> {
        Ok(Self::default().prepare_command(args)?.yaml()?.args()?.propagate_chain_id().config)
    }

    // Builds the applications command-line interface.
    fn prepare_command(mut self, args: Vec<String>) -> Result<Self, ConfigError> {
        self.args = Some(
            Command::new("Papyrus",)
            .version(VERSION_FULL)
            .about("Papyrus is a StarkNet full node written in Rust.")
            .args(&[
                arg!(-f --config_file [path] "Optionally sets a config file to use").value_parser(value_parser!(PathBuf)),
                arg!(-c --chain_id [name] "Optionally sets chain id to use"),
                arg!(--server_address ["IP:PORT"] "Optionally sets the RPC listening address"),
                arg!(--http_headers ["NAME:VALUE"] ... "Optionally adds headers to the http requests"),
                arg!(-s --storage [path] "Optionally sets storage path to use (automatically extended with chain ID)").value_parser(value_parser!(PathBuf)),
                arg!(-n --no_sync [bool] "Optionally run without sync").value_parser(value_parser!(bool)).default_missing_value("true"),
                arg!(--central_url ["URL"] "Central URL. It should match chain_id."),
            ])
            .try_get_matches_from(args).unwrap_or_else(|e| e.exit()),
        );
        Ok(self)
    }

    // Parses a yaml configuration file given by the command-line args (or default), and applies it
    // on the configuration.
    fn yaml(mut self) -> Result<Self, ConfigError> {
        let mut yaml_path = CONFIG_FILE;

        let args = self.args.clone().expect("Config builder should have args.");
        if let Some(config_file) = args.try_get_one::<PathBuf>("config_file")? {
            yaml_path =
                config_file.to_str().ok_or(ConfigError::BadPath { path: config_file.clone() })?;
        }

        let yaml_contents = fs::read_to_string(yaml_path)?;
        let from_yaml: FileConfigFormat = serde_yaml::from_str(&yaml_contents)?;
        from_yaml.update_config(&mut self);

        Ok(self)
    }

    // Reads the command-line args and updates the relevant configurations.
    fn args(mut self) -> Result<Self, ConfigError> {
        match self.args {
            None => unreachable!(),
            Some(ref args) => {
                if let Some(chain_id) = args.try_get_one::<String>("chain_id")? {
                    self.chain_id = ChainId(chain_id.clone());
                }

                if let Some(server_address) = args.try_get_one::<String>("server_address")? {
                    self.config.gateway.server_address = server_address.to_string()
                }

                if let Some(storage_path) = args.try_get_one::<PathBuf>("storage")? {
                    self.config.storage.db_config.path = storage_path.to_owned();
                }

                if let Some(http_headers) = args.try_get_one::<String>("http_headers")? {
                    let mut headers_map = match self.config.central.http_headers {
                        Some(map) => map,
                        None => HashMap::new(),
                    };
                    for header in http_headers.split(' ') {
                        let split: Vec<&str> = header.split(':').collect();
                        if split.len() != 2 {
                            return Err(ConfigError::CLAHttpHeader {
                                illegal_header: header.to_string(),
                            });
                        }
                        headers_map.insert(split[0].to_string(), split[1].to_string());
                    }
                    self.config.central.http_headers = Some(headers_map);
                }

                if let Some(no_sync) = args.try_get_one::<bool>("no_sync")? {
                    if *no_sync {
                        self.config.sync = None;
                    }
                }
                if let Some(central_url) = args.try_get_one::<String>("central_url")? {
                    self.config.central.url = central_url.to_string()
                }

                Ok(self)
            }
        }
    }

    // Propagates the chain id into all the of configurations that use it.
    fn propagate_chain_id(mut self) -> Self {
        self.config.gateway.chain_id = self.chain_id.clone();
        // Assuming a valid path.
        self.config.storage.db_config.path.push(self.chain_id.0.as_str());
        self
    }
}
