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
use crate::client::configuration::configuration_error::ConfigurationError::NoPassword;
use crate::client::configuration::{
    get_mandatory_integer, get_mandatory_string, get_optional_boolean, get_optional_integer,
    get_optional_string,
};
use crate::transport::mqtt::configure_transport;
use rumqttc::MqttOptions;
use std::ops::Deref;

/// Represents the MQTT configuration.
#[derive(Clone, Debug)]
pub struct MqttConfiguration {
    pub mqtt_options: MqttOptions,
}

impl Default for MqttConfiguration {
    fn default() -> Self {
        Self {
            mqtt_options: MqttOptions::new("default_client", ("localhost", 1883)),
        }
    }
}

impl TryFrom<&toml::Table> for MqttConfiguration {
    type Error = ConfigurationError;

    /// Tries to create an `MqttConfiguration` from the given properties.
    ///
    /// # Arguments
    ///
    /// * `properties` - Properties to create the configuration from.
    ///
    /// # Returns
    ///
    /// A result containing the `MqttConfiguration` or an error.
    fn try_from(properties: &toml::Table) -> Result<Self, Self::Error> {
        let host = get_mandatory_string(None, "host", properties)?;
        let port = get_mandatory_integer::<u16>(None, "port", properties)?;
        let mut mqtt_options = MqttOptions::new(
            get_mandatory_string(None, "client_id", properties)?,
            (host, port),
        );
        let max_packet_size =
            get_optional_integer::<u32>(None, "max_packet_size", properties)?.unwrap_or(10_000);
        mqtt_options.set_max_packet_size(Some(max_packet_size));

        if let Ok(Some(username)) = get_optional_string(None, "username", properties) {
            if let Ok(Some(password)) = get_optional_string(None, "password", properties) {
                mqtt_options.set_credentials(username, password);
            } else {
                return Err(NoPassword);
            }
        }
        if let Ok(Some(connection_timeout)) =
            get_optional_integer::<u64>(None, "connection_timeout", properties)
        {
            mqtt_options.set_connect_timeout(std::time::Duration::from_secs(connection_timeout));
        }

        let use_tls = get_optional_boolean(None, "use_tls", properties)
            .unwrap_or_default()
            .unwrap_or_default();
        let use_websocket = get_optional_boolean(None, "use_websocket", properties)
            .unwrap_or_default()
            .unwrap_or_default();

        let ca_file = get_optional_string(None, "ca_file", properties).unwrap_or_default();
        let cert_file = get_optional_string(None, "cert_file", properties).unwrap_or_default();
        let key_file = get_optional_string(None, "key_file", properties).unwrap_or_default();

        configure_transport(
            use_tls,
            use_websocket,
            &mut mqtt_options,
            ca_file,
            cert_file,
            key_file,
        );
        Ok(Self { mqtt_options })
    }
}

impl Deref for MqttConfiguration {
    type Target = MqttOptions;

    /// Dereferences the `MqttConfiguration` to `MqttOptions`.
    ///
    /// # Returns
    ///
    /// A reference to the `MqttOptions`.
    fn deref(&self) -> &Self::Target {
        &self.mqtt_options
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    macro_rules! load_toml_table {
        ($str_conf: expr) => {
            toml::Table::from_str($str_conf).expect("Failed to load TOML configuration")
        };
    }

    #[test]
    fn mqtt_configuration_from_valid_properties() {
        let properties = load_toml_table!(
            r#"
    client_id = "test_client"
    host = "localhost"
    port = 1883
    "#
        );

        let config = MqttConfiguration::try_from(&properties).unwrap();

        assert_eq!(config.mqtt_options.client_id(), "test_client");
        assert_eq!(
            format!(
                "{}:{}",
                config.mqtt_options.broker().tcp_address().unwrap().0,
                config.mqtt_options.broker().tcp_address().unwrap().1
            ),
            "localhost:1883"
        );
    }

    #[test]
    fn mqtt_configuration_missing_mandatory_fields() {
        let properties = load_toml_table!(
            r#"
            [mqtt]
            host = "localhost"
            port = "1883"
        "#
        );

        let result = MqttConfiguration::try_from(&properties);

        assert!(result.is_err());
    }

    #[test]
    fn mqtt_configuration_with_credentials() {
        let properties = load_toml_table!(
            r#"
            client_id = "test_client"
            host = "localhost"
            port = 1883
            username = "user"
            password = "pass"
        "#
        );

        let config = MqttConfiguration::try_from(&properties).unwrap();

        assert_eq!(
            config.mqtt_options.auth(),
            &rumqttc::ConnectAuth::UsernamePassword {
                username: "user".to_string(),
                password: "pass".to_string().into(),
            }
        );
    }

    #[test]
    fn mqtt_configuration_missing_password() {
        let properties = load_toml_table!(
            r#"
            client_id = "test_client"
            host = "localhost"
            port = 1883
            username = "user"
        "#
        );

        let result = MqttConfiguration::try_from(&properties);

        assert!(matches!(result, Err(NoPassword)));
    }

    #[test]
    fn mqtt_configuration_with_tls_and_websocket() {
        let properties = load_toml_table!(
            r#"
            client_id = "test_client"
            host = "localhost"
            port = 1883
            use_tls = true
            use_websocket = true
        "#
        );

        let config = MqttConfiguration::try_from(&properties).unwrap();

        assert!(matches!(
            config.mqtt_options.transport(),
            rumqttc::Transport::Wss(_)
        ));
    }

    #[test]
    fn mqtt_configuration_with_tls_no_websocket() {
        let properties = load_toml_table!(
            r#"
            client_id = "test_client"
            host = "localhost"
            port = 1883
            use_tls = true
            use_websocket = false
        "#
        );

        let config = MqttConfiguration::try_from(&properties).unwrap();

        assert!(matches!(
            config.mqtt_options.transport(),
            rumqttc::Transport::Tls(_)
        ));
    }

    #[test]
    fn mqtt_configuration_with_websocket_no_tls() {
        let properties = load_toml_table!(
            r#"
            client_id = "test_client"
            host = "localhost"
            port = 1883
            use_tls = false
            use_websocket = true
        "#
        );

        let config = MqttConfiguration::try_from(&properties).unwrap();

        assert!(matches!(
            config.mqtt_options.transport(),
            rumqttc::Transport::Ws
        ));
    }

    #[test]
    fn mqtt_configuration_without_tls_and_websocket() {
        let properties = load_toml_table!(
            r#"
            client_id = "test_client"
            host = "localhost"
            port = 1883
            use_tls = false
            use_websocket = false
        "#
        );

        let config = MqttConfiguration::try_from(&properties).unwrap();

        assert!(matches!(
            config.mqtt_options.transport(),
            rumqttc::Transport::Tcp
        ));
    }
}
