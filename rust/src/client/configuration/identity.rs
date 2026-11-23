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
use crate::client::configuration::configuration_error::ConfigurationError;
use crate::client::configuration::configuration_error::ConfigurationError::{
    FileNotFound, InvalidFileType,
};
use crate::client::configuration::{Configuration, get_mandatory_string, pick_mandatory_section};
use log::{info, warn};
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct Identity {
    #[serde(alias = "psk-identity")]
    username: String,
    #[serde(alias = "psk-secret-key")]
    password: String,
    #[serde(alias = "iot3-id")]
    id: String,
}

/// Overrides the [Configuration] parts related to service access
///
/// This function will:
/// - override the credentials attributes
/// - suffix the id attributes (MQTT client_id for example)
/// in all relevant configuration structures.
///
/// The username, password and id suffix will be read from the file provided through the
/// `identity.file` field in the INI configuration file (see [example/config.ini]).
///
/// The file must content a single JSON object with these three fields.
/// Example:
///  ```json
///   {
///     "id": "192837465",
///     "username": "zqsdfguiqjoizhfze57451zg5f4",
///     "password": "86e16f35a59311780627630d9357fbabe11dbd71f7c9ce662402cb846c5c24f0"
///   }
///   ```
pub(crate) fn override_with_identity(
    toml_configuration: &mut toml::Table,
    configuration: &mut Configuration,
) -> Result<(), ConfigurationError> {
    match pick_mandatory_section("identity", toml_configuration) {
        Ok(identity) => {
            let file_name = get_mandatory_string(None, "file", &identity)?;

            match fs::read_to_string(&file_name) {
                Ok(json_str) => match serde_json::from_str::<Identity>(&json_str) {
                    Ok(identity) => {
                        let current_client_id =
                            configuration.mqtt.mqtt_options.client_id().to_string();
                        configuration
                            .mqtt
                            .mqtt_options
                            .set_client_id(format!("{}-{}", current_client_id, identity.id));
                        configuration.mqtt.mqtt_options.set_credentials(
                            identity.username.clone(),
                            identity.password.as_bytes().to_vec(),
                        );

                        #[cfg(feature = "telemetry")]
                        {
                            configuration.telemetry.username = Some(identity.username);
                            configuration.telemetry.password = Some(identity.password);
                        }

                        Ok(())
                    }
                    Err(e) => {
                        warn!("{}", e);
                        Err(InvalidFileType(file_name))
                    }
                },
                Err(err) => {
                    warn!("{}", err);
                    Err(FileNotFound(file_name))
                }
            }
        }
        Err(error) => {
            info!(
                "No identity section found, using credentials from dedicated sections: {}",
                error
            );
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn missing_section_returns_ok() {
        let ini_str = r#"
        [not_identity]
        file="/dev/null"
        "#;
        let mut toml_cfg =
            toml::Table::from_str(ini_str).expect("Failed to create TOM configuration");
        let mut configuration = Configuration::default();

        let result = override_with_identity(&mut toml_cfg, &mut configuration);

        assert!(result.is_ok());
    }
}
