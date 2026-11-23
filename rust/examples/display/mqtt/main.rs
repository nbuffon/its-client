/*
 * Software Name : libits-client
 * SPDX-FileCopyrightText: Copyright (c) Orange SA
 * SPDX-License-Identifier: MIT
 *
 * This software is distributed under the MIT license,
 * see the "LICENSE.txt" file for more details or https://opensource.org/license/MIT/
 *
 * Authors: see CONTRIBUTORS.md
 */

#[path = "../database.rs"]
mod database;
#[path = "../helpers.rs"]
mod helpers;
#[path = "../database/ingest.rs"]
mod ingest;
mod mqtt_reader;

use anyhow::Result;
use clap::{Arg, Command};
use libits::client::configuration::mqtt_configuration::MqttConfiguration;
use libits::client::logger::create_stdout_logger;
use serde::Deserialize;
use std::fs::read_to_string;
use std::path::Path;

const DISPLAY_SECTION: &str = "display";
const RECEIVER_SECTION: &str = "receiver";
const MQTT_SECTION: &str = "mqtt";

#[derive(Deserialize)]
struct Display {
    pub db_path: String,
    pub zoom: u8,
    pub _port: u32,
}

#[tokio::main]
async fn main() -> Result<()> {
    let _logger = create_stdout_logger().expect("Logger initialization failed");

    let matches = Command::new("Display Ingestor")
        .version("0.1.0")
        .about("Subscribes to MQTT and ingests messages into a SQLite database")
        .arg(
            Arg::new("config-file-path")
                .short('c')
                .long("config")
                .value_name("CONFIG_FILE_PATH")
                .default_value("examples/config.toml")
                .help("Path to the configuration file"),
        )
        .get_matches();

    let config_path = matches.get_one::<String>("config-file-path").unwrap();

    let toml = match read_to_string(Path::new(config_path)) {
        Ok(toml_string) => match toml::from_str::<toml::Table>(&toml_string) {
            Ok(toml) => toml,
            Err(e) => {
                panic!("Failed to load config file as TOML: {:?}", e);
            }
        },
        Err(e) => {
            panic!("Failed to load config file {}: {}", config_path, e);
        }
    };

    // Build the MQTT options straight from the [mqtt] section instead of the full
    // library configuration, which would also require unrelated mandatory sections
    // (e.g. [mobility]) that a display-only config does not provide.
    let mqtt_properties = toml
        .get(MQTT_SECTION)
        .expect("Missing [mqtt] section in config file")
        .as_table()
        .expect("Found `mqtt` key but it does not seem to be a section");
    let mqtt_configuration = MqttConfiguration::try_from(mqtt_properties).unwrap_or_else(|error| {
        panic!(
            "Invalid [{}] configuration in {}: {}",
            MQTT_SECTION, config_path, error
        )
    });

    let display_section = toml::Table::try_from(
        toml.get(DISPLAY_SECTION)
            .expect(format!("Missing mandatory section {}", DISPLAY_SECTION).as_str()),
    )
    .expect(format!("{} must be a section", DISPLAY_SECTION).as_str());

    let display_config = display_section
        .try_into::<Display>()
        .expect("Failed to parse display config");

    let topics: Vec<String> = toml
        .get(RECEIVER_SECTION)
        .and_then(|section| {
            let vec = if let Some(table) = section.as_table() {
                if let Some(topic_list) = table.get("topic_list") {
                    if let Some(topic_array) = topic_list.as_array() {
                        topic_array.clone()
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            };

            Some(vec)
        })
        .map(|value| {
            value
                .iter()
                .map(|value| value.as_str().unwrap_or_default().to_string())
                .collect::<Vec<String>>()
        })
        .filter(|topics| !topics.is_empty())
        .unwrap_or_else(|| vec!["#".to_string()]);

    let config = mqtt_reader::MqttIngestorConfig {
        mqtt_options: mqtt_configuration.mqtt_options,
        topics,
        zoom: display_config.zoom,
        db_path: display_config.db_path,
    };

    mqtt_reader::run_mqtt_ingestor(config).await?;

    Ok(())
}
