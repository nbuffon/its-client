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
mod server;
#[path = "../database/stats.rs"]
mod stats;

use anyhow::{Result, ensure};
use clap::{Arg, Command};
use libits::client::logger::create_stdout_logger;
use serde::Deserialize;
use std::fs::read_to_string;
use std::path::Path;

const DISPLAY_SECTION: &str = "display";

#[derive(Deserialize)]
struct Display {
    pub db_path: String,
    pub _zoom: u8,
    pub port: u32,
}

//noinspection HttpUrlsUsage
#[tokio::main]
async fn main() -> Result<()> {
    let _logger = create_stdout_logger().expect("Logger initialization failed");

    let matches = Command::new("Display Server")
        .version("0.1.0")
        .about("Serves the display web UI from a SQLite database")
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
    let toml_config = match read_to_string(Path::new(config_path)) {
        Ok(toml_string) => match toml::from_str::<toml::Table>(&toml_string) {
            Ok(toml) => toml,
            Err(e) => {
                panic!("Failed to load config file as TOML: {:?}", e);
            }
        },
        Err(e) => {
            panic!("Failed to read config file: {:?}", e);
        }
    };
    let display_section = toml::Table::try_from(
        toml_config
            .get(DISPLAY_SECTION)
            .expect(format!("Missing mandatory section {}", DISPLAY_SECTION).as_str()),
    )
    .expect(format!("{} must be a section", DISPLAY_SECTION).as_str());

    let display_config = display_section
        .try_into::<Display>()
        .expect("Failed to parse display config");

    ensure!(
        Path::new(&display_config.db_path).try_exists()?,
        "Database not found: {}. Run 'display_log_reader' or 'display_mqtt_reader' first.",
        display_config.db_path
    );

    // Print database stats on startup
    stats::check_database(&display_config.db_path)?;

    let app = server::create_router(&display_config.db_path);
    let bind_address = format!("0.0.0.0:{}", display_config.port);
    let listener = tokio::net::TcpListener::bind(&bind_address).await?;
    println!(
        "🚀 Server running on http://{} (DB: {})",
        bind_address, display_config.db_path
    );
    println!("📍 Open your browser and navigate to the URL above");
    axum::serve(listener, app).await?;

    Ok(())
}
