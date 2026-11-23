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
use crate::client::configuration::get_mandatory_string;

pub(crate) const GEO_SECTION: &str = "geo";

/// Configuration of the geo_routing feature
///
/// Contains the information to build [GeoTopic][1]s
///
/// Example
/// ```ini
/// [geo]
/// prefix=myProject
/// suffix=my_domain
/// ```
///
/// [1]: crate::transport::mqtt::geo_topic::GeoTopic
#[derive(Clone, Debug, Default)]
pub struct GeoConfiguration {
    pub prefix: String,
    pub suffix: String,
}

impl TryFrom<&toml::Table> for GeoConfiguration {
    type Error = ConfigurationError;

    fn try_from(properties: &toml::Table) -> Result<Self, Self::Error> {
        Ok(Self {
            prefix: get_mandatory_string(None, "prefix", properties)?,
            suffix: get_mandatory_string(None, "suffix", properties)?,
        })
    }
}
