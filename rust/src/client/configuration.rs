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
    FileReadFailure, InvalidFileType, MissingMandatoryField, MissingMandatorySection,
    NoCustomSettings, TypeError,
};
use log::warn;
use std::any::type_name;
use std::fs::read_to_string;
use std::path::Path;
use std::str::FromStr;

#[cfg(feature = "geo_routing")]
use crate::client::configuration::geo_configuration::{GEO_SECTION, GeoConfiguration};
#[cfg(feature = "mobility")]
use crate::client::configuration::mobility_configuration::{
    MOBILITY_SECTION, MobilityConfiguration,
};
#[cfg(feature = "telemetry")]
use crate::client::configuration::telemetry_configuration::{
    TELEMETRY_SECTION, TelemetryConfiguration,
};

#[cfg(feature = "identity")]
use crate::client::configuration::identity::override_with_identity;

use crate::client::configuration::mqtt_configuration::MqttConfiguration;

pub mod configuration_error;
#[cfg(feature = "geo_routing")]
pub(crate) mod geo_configuration;
#[cfg(feature = "identity")]
mod identity;
#[cfg(feature = "mobility")]
pub(crate) mod mobility_configuration;
pub mod mqtt_configuration;
#[cfg(feature = "telemetry")]
pub(crate) mod telemetry_configuration;

const MQTT_SECTION: &str = "mqtt";

#[derive(Clone, Debug, Default)]
pub struct Configuration {
    pub mqtt: MqttConfiguration,
    #[cfg(feature = "geo_routing")]
    pub geo: GeoConfiguration,
    #[cfg(feature = "telemetry")]
    pub telemetry: TelemetryConfiguration,
    #[cfg(feature = "mobility")]
    pub mobility: MobilityConfiguration,
    pub custom_settings: Option<toml::Table>,
}

impl Configuration {
    pub fn get_integer<T: TryFrom<i64>>(
        &self,
        section: Option<&'static str>,
        key: &'static str,
    ) -> Result<T, ConfigurationError> {
        if let Some(custom_settings) = &self.custom_settings {
            get_mandatory_integer::<T>(section, key, custom_settings)
        } else {
            Err(NoCustomSettings)
        }
    }

    pub fn get_float<T: TryFrom<f64>>(
        &self,
        section: Option<&'static str>,
        key: &'static str,
    ) -> Result<T, ConfigurationError> {
        if let Some(custom_settings) = &self.custom_settings {
            get_mandatory_float::<T>(section, key, custom_settings)
        } else {
            Err(NoCustomSettings)
        }
    }

    pub fn get_boolean(
        &self,
        section: Option<&'static str>,
        key: &'static str,
    ) -> Result<bool, ConfigurationError> {
        if let Some(custom_settings) = &self.custom_settings {
            get_mandatory_boolean(section, key, custom_settings)
        } else {
            Err(NoCustomSettings)
        }
    }

    pub fn get_string(
        &self,
        section: Option<&'static str>,
        key: &'static str,
    ) -> Result<String, ConfigurationError> {
        if let Some(custom_settings) = &self.custom_settings {
            get_mandatory_string(section, key, custom_settings)
        } else {
            Err(NoCustomSettings)
        }
    }

    pub fn get_list<T: FromStr>(
        &self,
        section: Option<&'static str>,
        key: &'static str,
    ) -> Result<Vec<T>, ConfigurationError> {
        if let Some(custom_settings) = &self.custom_settings {
            let parsed = get_mandatory_array(section, key, custom_settings)?
                .iter()
                .filter_map(|value| value.as_str())
                .map(|value| value.trim())
                .map(|s| T::from_str(s))
                .collect();

            match parsed {
                Ok(values) => Ok(values),
                Err(_) => Err(TypeError(key, type_name::<T>())),
            }
        } else {
            Err(NoCustomSettings)
        }
    }

    pub fn set<T>(&mut self, section: Option<&str>, key: &str, value: T)
    where
        toml::Value: From<T>,
    {
        let custom_settings = self
            .custom_settings
            .get_or_insert_with(toml::Table::default);

        let table = if let Some(section) = section {
            if let Some(section) = custom_settings.get_mut(section) {
                match section {
                    toml::Value::Table(table) => table,
                    _ => custom_settings,
                }
            } else {
                custom_settings.insert(section.to_string(), toml::Value::Table(Default::default()));
                custom_settings
                    .get_mut(section)
                    .unwrap()
                    .as_table_mut()
                    .unwrap()
            }
        } else {
            custom_settings
        };

        table.insert(key.to_string(), toml::Value::from(value));
    }
}

fn get_optional<'t>(
    section: Option<&'static str>,
    field: &'static str,
    toml_config: &'t toml::Table,
) -> Result<Option<&'t toml::Value>, ConfigurationError> {
    let properties = if let Some(section) = section {
        match toml_config.get(section) {
            Some(toml::Value::Table(section)) => section,
            None => return Ok(None),
            _ => return Err(TypeError(section, "toml::Table")),
        }
    } else {
        toml_config
    };

    Ok(properties.get(field))
}

pub(crate) fn get_optional_boolean(
    section: Option<&'static str>,
    field: &'static str,
    properties: &toml::Table,
) -> Result<Option<bool>, ConfigurationError> {
    match get_optional(section, field, properties)? {
        Some(value) => {
            if let Some(boolean) = value.as_bool() {
                Ok(Some(boolean))
            } else {
                Err(TypeError(field, "boolean"))
            }
        }
        None => Ok(None),
    }
}

pub(crate) fn get_optional_integer<T: TryFrom<i64>>(
    section: Option<&'static str>,
    field: &'static str,
    properties: &toml::Table,
) -> Result<Option<T>, ConfigurationError> {
    match get_optional(section, field, properties)? {
        Some(value) => match value.as_integer() {
            Some(integer) => T::try_from(integer)
                .map(Some)
                .map_err(|_| TypeError(field, type_name::<T>())),
            None => Err(TypeError(field, "integer")),
        },
        None => Ok(None),
    }
}

pub(crate) fn get_optional_string(
    section: Option<&'static str>,
    field: &'static str,
    properties: &toml::Table,
) -> Result<Option<String>, ConfigurationError> {
    match get_optional(section, field, properties)? {
        Some(value) => match value.as_str() {
            Some(string) => Ok(Some(string.to_string())),
            None => Err(TypeError(field, "string")),
        },
        None => Ok(None),
    }
}

fn get_mandatory<'t>(
    section: Option<&'static str>,
    field: &'static str,
    toml_config: &'t toml::Table,
) -> Result<&'t toml::Value, ConfigurationError> {
    let properties = if let Some(section) = section {
        match toml_config.get(section) {
            Some(toml::Value::Table(section)) => section,
            None => return Err(MissingMandatorySection(section)),
            _ => return Err(TypeError(section, "toml::Table")),
        }
    } else {
        toml_config
    };

    match properties.get(field) {
        Some(value) => Ok(value),
        None => Err(MissingMandatoryField(field)),
    }
}

pub(crate) fn get_mandatory_string(
    section: Option<&'static str>,
    field: &'static str,
    toml_config: &toml::Table,
) -> Result<String, ConfigurationError> {
    get_mandatory(section, field, toml_config)?
        .as_str()
        .map(|v| v.to_string())
        .ok_or(TypeError(field, "String"))
}

pub(crate) fn get_mandatory_boolean(
    section: Option<&'static str>,
    field: &'static str,
    toml_config: &toml::Table,
) -> Result<bool, ConfigurationError> {
    get_mandatory(section, field, toml_config)?
        .as_bool()
        .ok_or(TypeError(field, "boolean"))
}

pub(crate) fn get_mandatory_integer<T: TryFrom<i64>>(
    section: Option<&'static str>,
    field: &'static str,
    toml_config: &toml::Table,
) -> Result<T, ConfigurationError> {
    let integer = get_mandatory(section, field, toml_config)?
        .as_integer()
        .ok_or(TypeError(field, type_name::<T>()))?;
    T::try_from(integer).map_err(|_| TypeError(field, type_name::<T>()))
}

pub(crate) fn get_mandatory_float<T: TryFrom<f64>>(
    section: Option<&'static str>,
    field: &'static str,
    toml_config: &toml::Table,
) -> Result<T, ConfigurationError> {
    let float = get_mandatory(section, field, toml_config)?
        .as_float()
        .ok_or(TypeError(field, "float"))?;
    T::try_from(float).map_err(|_| TypeError(field, type_name::<T>()))
}

pub(crate) fn get_mandatory_array(
    section: Option<&'static str>,
    field: &'static str,
    toml_config: &toml::Table,
) -> Result<Vec<toml::Value>, ConfigurationError> {
    let value = get_mandatory(section, field, toml_config)?;
    match value.as_array() {
        Some(array) => Ok(array.to_vec()),
        None => Err(TypeError(field, "toml::Array")),
    }
}

pub(crate) fn pick_mandatory_section(
    section: &'static str,
    ini_config: &mut toml::Table,
) -> Result<toml::Table, ConfigurationError> {
    match ini_config.remove(section) {
        Some(toml::Value::Table(properties)) => Ok(properties),
        _ => Err(MissingMandatorySection(section)),
    }
}

impl TryFrom<toml::Table> for Configuration {
    type Error = ConfigurationError;

    fn try_from(toml_config: toml::Table) -> Result<Self, Self::Error> {
        let mut ini_config = toml_config;

        #[allow(unused_mut)]
        let mut configuration = Configuration {
            mqtt: MqttConfiguration::try_from(&pick_mandatory_section(
                MQTT_SECTION,
                &mut ini_config,
            )?)?,
            #[cfg(feature = "geo_routing")]
            geo: GeoConfiguration::try_from(&pick_mandatory_section(
                GEO_SECTION,
                &mut ini_config,
            )?)?,
            #[cfg(feature = "telemetry")]
            telemetry: TelemetryConfiguration::try_from(&pick_mandatory_section(
                TELEMETRY_SECTION,
                &mut ini_config,
            )?)?,
            #[cfg(feature = "mobility")]
            mobility: MobilityConfiguration::try_from(&pick_mandatory_section(
                MOBILITY_SECTION,
                &mut ini_config,
            )?)?,
            custom_settings: None,
        };

        #[cfg(feature = "identity")]
        override_with_identity(&mut ini_config, &mut configuration)?;

        configuration.custom_settings = Some(ini_config);

        Ok(configuration)
    }
}

impl TryFrom<&Path> for Configuration {
    type Error = ConfigurationError;

    fn try_from(toml_path: &Path) -> Result<Self, Self::Error> {
        match read_to_string(toml_path) {
            Ok(toml_string) => match toml::from_str::<toml::Table>(&toml_string) {
                Ok(toml) => Self::try_from(toml),
                Err(e) => {
                    warn!("Failed to load config file as TOML: {:?}", e);
                    Err(InvalidFileType(toml_path.display().to_string()))
                }
            },
            Err(e) => {
                warn!("{:?}", e);
                Err(FileReadFailure(toml_path.display().to_string()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::configuration::{Configuration, get_optional, pick_mandatory_section};

    const EXHAUSTIVE_CUSTOM_CONFIG: &str = r#"
no_section = "noitceson"

[mqtt]
host = "localhost"
port = 1883
use_tls = false
use_websocket = false
client_id = "com_myapplication"
username = "username"
password = "password"

[geo]
prefix = "sandbox"
suffix = "v2x"

[mobility]
source_uuid = "com_myapplication-1"
station_id = 1
use_responsibility = true
thread_count = 4

[telemetry]
host = "otlp.domain.com"
port = 5418
use_tls = false
path = "/custom/v1/traces"
max_batch_size = 10
username = "username"
password = "password"

[custom]
test = "success"
"#;

    const MINIMAL_FEATURELESS_CONFIGURATION: &str = r#"
[mqtt]
host = "localhost"
port = 1883
use_tls = false
use_websocket = false
client_id = "com_myapplication"
"#;

    #[cfg(feature = "mobility")]
    const MINIMAL_MOBILITY_CONFIGURATION: &str = r#"
[mqtt]
host = "localhost"
port = 1883
use_tls = false
use_websocket = false
client_id = "com_myapplication"

[mobility]
source_uuid = "com_myapplication-1"
station_id = 1
use_responsibility = false
thread_count = 4
"#;

    #[cfg(feature = "geo_routing")]
    const MINIMAL_GEO_ROUTING_CONFIGURATION: &str = r#"
[mqtt]
host = "localhost"
port=1883
use_tls = false
use_websocket = false
client_id= "com_myapplication"

[mobility]
source_uuid = "com_myapplication-1"
station_id = 1
use_responsibility = false
thread_count = 4

[geo]
prefix = "sandbox"
suffix = "v2x"
"#;

    #[cfg(feature = "telemetry")]
    const MINIMAL_TELEMETRY_CONFIGURATION: &str = r#"
[mqtt]
host = "localhost"
port = 1883
use_tls = false
use_websocket = false
client_id = "com_myapplication"

[telemetry]
host = "otlp.domain.com"
port = 5418
use_tls = false
"#;

    macro_rules! load_toml_table {
        ($str_conf: expr) => {
            toml::Table::from_str($str_conf).expect("Failed to load TOML configuration")
        };
    }

    #[test]
    fn pick_section() {
        let mut toml = load_toml_table!(EXHAUSTIVE_CUSTOM_CONFIG);

        let s = pick_mandatory_section("mqtt", &mut toml);

        assert!(s.is_ok());
        assert!(toml.get("mqtt").is_none());
    }

    // --- Optional getters

    #[test]
    fn existing_optional_no_section_returns_some() {
        let toml = load_toml_table!(EXHAUSTIVE_CUSTOM_CONFIG);

        let value = get_optional(None, "no_section", &toml);

        assert!(value.is_ok());
        let value = value.unwrap();
        assert!(value.is_some());
        let value = value.unwrap();
        assert_eq!(value.as_str().unwrap(), "noitceson");
    }

    #[test]
    fn existing_optional_in_section_returns_some() {
        let toml = load_toml_table!(EXHAUSTIVE_CUSTOM_CONFIG);

        let value = get_optional(Some("custom"), "test", &toml);

        assert!(value.is_ok());
        let value = value.unwrap();
        assert!(value.is_some());
        let value = value.unwrap();
        assert_eq!(value.as_str().unwrap(), "success");
    }

    #[test]
    fn missing_optional_no_section_returns_none() {
        let toml = load_toml_table!(EXHAUSTIVE_CUSTOM_CONFIG);

        let result = get_optional(Some("custom"), "pmloikjuyh", &toml);

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn missing_optional_in_existing_section_returns_none() {
        let toml = load_toml_table!(EXHAUSTIVE_CUSTOM_CONFIG);

        let result = get_optional(None, "pmloikjuyh", &toml);

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn missing_optional_section_returns_none() {
        let toml = load_toml_table!(EXHAUSTIVE_CUSTOM_CONFIG);

        let result = get_optional(Some("non_existent_section"), "field", &toml);

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn existing_optional_with_wrong_type_is_err() {
        let toml = load_toml_table!(EXHAUSTIVE_CUSTOM_CONFIG);

        let err = get_optional_integer::<u16>(Some("custom"), "test", &toml);

        assert!(err.is_err());
    }

    // --- Mandatory getters

    #[test]
    fn mandatory_no_section_is_ok() {
        let toml = load_toml_table!(EXHAUSTIVE_CUSTOM_CONFIG);

        let result = get_mandatory(None, "no_section", &toml);

        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value.as_str().unwrap(), "noitceson");
    }

    #[test]
    fn mandatory_from_section_is_ok() {
        let toml = load_toml_table!(EXHAUSTIVE_CUSTOM_CONFIG);

        let result = get_mandatory(Some("custom"), "test", &toml);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().as_str().unwrap(), "success");
    }

    #[test]
    fn mandatory_missing_field_is_err() {
        let toml = load_toml_table!(EXHAUSTIVE_CUSTOM_CONFIG);

        let result = get_mandatory(None, "non_existent", &toml);

        assert!(result.is_err());
    }

    #[test]
    fn mandatory_missing_field_in_section_is_err() {
        let toml = load_toml_table!(EXHAUSTIVE_CUSTOM_CONFIG);

        let result = get_mandatory(Some("custom"), "missing", &toml);

        assert!(result.is_err());
    }

    #[test]
    fn mandatory_wrong_type_is_err() {
        let toml = load_toml_table!(EXHAUSTIVE_CUSTOM_CONFIG);

        let result = get_mandatory_integer::<u16>(Some("custom"), "test", &toml);

        assert!(result.is_err());
    }

    #[test]
    fn get_mandatory_missing_section() {
        let toml = load_toml_table!(EXHAUSTIVE_CUSTOM_CONFIG);

        let result = get_mandatory(Some("missing"), "test", &toml);

        assert!(result.is_err());
    }

    // --- Lists

    #[test]
    fn get_list_with_comma_separated_values() {
        let configuration = Configuration {
            custom_settings: Some(load_toml_table!(
                r#"
            [test]
            list_field = ["item1", "item2", "item3"]
            "#
            )),
            ..Default::default()
        };

        let result = configuration
            .get_list::<String>(Some("test"), "list_field")
            .unwrap();

        assert_eq!(result, vec!["item1", "item2", "item3"]);
    }

    #[test]
    fn get_list_no_custom_settings_error() {
        let configuration = Configuration {
            custom_settings: None,
            ..Default::default()
        };

        let result = configuration.get_list::<String>(Some("test"), "list_field");

        assert!(matches!(result, Err(NoCustomSettings)));
    }

    #[test]
    fn get_list_with_spaces_and_trimming() {
        let configuration = Configuration {
            custom_settings: Some(load_toml_table!(
                r#"
            [test]
            list_field = [" item1", " item2 ", "item3 "]
            "#
            )),
            ..Default::default()
        };

        let result = configuration
            .get_list::<String>(Some("test"), "list_field")
            .unwrap();

        assert_eq!(result, vec!["item1", "item2", "item3"]);
    }

    #[test]
    fn get_list_type_conversion_error() {
        let configuration = Configuration {
            custom_settings: Some(load_toml_table!(
                r#"
            [test]
            list_field = ["not_a_number", 123]
            "#
            )),
            ..Default::default()
        };

        let result = configuration.get_list::<u32>(Some("test"), "list_field");

        assert!(result.is_err());
    }

    // -- Configuration creation

    #[test]
    #[cfg_attr(any(feature = "telemetry", feature = "mobility"), should_panic)]
    fn minimal_featureless_configuration() {
        let toml = load_toml_table!(MINIMAL_FEATURELESS_CONFIGURATION);

        let _ = Configuration::try_from(toml)
            .expect("Failed to create Configuration with minimal mandatory sections and fields");
    }

    #[test]
    #[cfg(feature = "telemetry")]
    #[cfg_attr(feature = "mobility", should_panic)]
    fn minimal_telemetry_configuration() {
        let toml = load_toml_table!(MINIMAL_TELEMETRY_CONFIGURATION);

        Configuration::try_from(toml)
            .expect("Failed to create Configuration with minimal mandatory sections and fields");
    }

    #[test]
    #[cfg(feature = "mobility")]
    #[cfg_attr(any(feature = "telemetry", feature = "geo_routing"), should_panic)]
    fn minimal_mobility_configuration() {
        let toml = load_toml_table!(MINIMAL_MOBILITY_CONFIGURATION);

        Configuration::try_from(toml)
            .expect("Failed to create Configuration with minimal mandatory sections and fields");
    }

    #[test]
    #[cfg(feature = "geo_routing")]
    #[cfg_attr(feature = "telemetry", should_panic)]
    fn minimal_geo_routing_configuration() {
        let toml = load_toml_table!(MINIMAL_GEO_ROUTING_CONFIGURATION);

        Configuration::try_from(toml)
            .expect("Failed to create Configuration with minimal mandatory sections and fields");
    }

    // --- Custom settings

    #[test]
    fn custom_settings() {
        let toml = load_toml_table!(EXHAUSTIVE_CUSTOM_CONFIG);

        let configuration = Configuration::try_from(toml).expect("Minimal config should not fail");
        let no_section = configuration
            .get_string(None, "no_section")
            .expect("Failed to get field with no section");
        let custom_test = configuration
            .get_string(Some("custom"), "test")
            .expect("Failed to get field under custom section");

        assert_eq!(no_section, "noitceson");
        assert_eq!(custom_test, "success");
    }

    #[test]
    fn set_custom_setting_unexisting_field_no_section() {
        let mut configuration = Configuration::default();

        configuration.set(None, "no_section", "updated");
        let no_section = configuration
            .get_string(None, "no_section")
            .expect("Failed to get field with no section");

        assert_eq!(no_section, "updated");
    }

    #[test]
    fn set_custom_setting_existing_field_no_section() {
        let mut configuration = Configuration {
            custom_settings: Some(load_toml_table!(
                r#"
            no_section = "nottheexpectedvalue"
            "#
            )),
            ..Default::default()
        };

        configuration.set(None, "no_section", "updated");
        let no_section = configuration
            .get_string(None, "no_section")
            .expect("Failed to get field with no section");

        assert_eq!(no_section, "updated");
    }

    #[test]
    fn set_custom_setting_unexisting_field_unexisting_section() {
        let toml = load_toml_table!(EXHAUSTIVE_CUSTOM_CONFIG);
        let mut configuration =
            Configuration::try_from(toml).expect("Minimal config should not fail");

        configuration.set(Some("my_section"), "cool_key", "cool_value");
        let cool_value = configuration
            .get_string(Some("my_section"), "cool_key")
            .expect("Failed to get field under custom section");

        assert_eq!(cool_value, "cool_value");
    }

    #[test]
    fn set_custom_setting_unexisting_field_existing_section() {
        let toml = load_toml_table!(EXHAUSTIVE_CUSTOM_CONFIG);
        let mut configuration =
            Configuration::try_from(toml).expect("Exhaustive config should not fail");

        configuration.set(Some("custom"), "test", "cool_value");
        let cool_value = configuration
            .get_string(Some("custom"), "test")
            .expect("Failed to get field under custom section");

        assert_eq!(cool_value, "cool_value");
    }

    #[test]
    fn set_custom_setting_existing_field_existing_section() {
        let mut configuration = Configuration {
            custom_settings: Some(load_toml_table!(
                r#"
            [my_section]
            cool_key = "nottheexpectedvalue"
            "#
            )),
            ..Default::default()
        };

        configuration.set(Some("my_section"), "cool_key", "cool_value");
        let cool_value = configuration
            .get_string(Some("my_section"), "cool_key")
            .expect("Failed to get field under custom section");

        assert_eq!(cool_value, "cool_value");
    }

    macro_rules! set_into_value_test {
        ($test_name:ident, $value:expr, $typ3:ty) => {
            #[test]
            fn $test_name() {
                let mut configuration = Configuration::default();

                configuration.set::<$typ3>(None, "thekeydoesnotexist", $value);

                assert!(configuration.custom_settings.is_some());
                assert_eq!(
                    configuration
                        .custom_settings
                        .unwrap()
                        .get("thekeydoesnotexist"),
                    Some(&toml::Value::from($value))
                );
            }
        };
    }

    set_into_value_test!(set_u8, 123u8, u8);
    set_into_value_test!(set_u32, 1234567891u32, u32);
    set_into_value_test!(set_i8, -123i8, i8);
    set_into_value_test!(set_i32, -1234567891i32, i32);
    set_into_value_test!(set_i64, -123456789012345i64, i64);
    set_into_value_test!(set_f32, 1.5f32, f32);
    set_into_value_test!(set_f64, 1.2345f64, f64);
    set_into_value_test!(set_bool, true, bool);
    set_into_value_test!(set_str, "a_str", &str);
    set_into_value_test!(set_string, String::from("a_string"), String);
    set_into_value_test!(set_vec_i32, vec![1i32, 2, 3], Vec<i32>);
    set_into_value_test!(set_vec_str, vec!["a", "b"], Vec<&str>);
    set_into_value_test!(
        set_table,
        toml::Table::from_iter([("k".to_string(), toml::Value::from(1i32))]),
        toml::Table
    );

    // -------------------

    #[test]
    /// This tests checks that if a section is provided, that the key exists in the TOML table
    /// but that the value behind this key is not a Table, then an error will be raised
    fn get_optional_section_is_not_a_table_raises_an_error() {
        let toml = load_toml_table!(
            r#"
        test_section = "errortoberaised"
        "#
        );

        let result = get_optional(Some("test_section"), "key", &toml);

        assert!(result.is_err());
    }

    #[test]
    /// This tests checks that if a section is provided, that the key exists in the TOML table
    /// but that the value behind this key is not a Table, then an error will be raised
    fn get_mandatory_section_is_not_a_table_raises_an_error() {
        let toml = load_toml_table!(
            r#"
        test_section = "errortoberaised"
        "#
        );

        let result = get_mandatory(Some("test_section"), "key", &toml);

        assert!(result.is_err());
    }
}
