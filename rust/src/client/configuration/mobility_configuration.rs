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
use crate::client::configuration::{
    get_mandatory_boolean, get_mandatory_integer, get_mandatory_string,
};

pub(crate) const MOBILITY_SECTION: &str = "mobility";

const STATION_ID_FIELD: &str = "station_id";
const SOURCE_UUID_FIELD: &str = "source_uuid";
const USE_RESPONSIBILITY_FIELD: &str = "use_responsibility";

const THREAD_COUNT_FIELD: &str = "thread_count";

#[derive(Clone, Debug, Default)]
pub struct MobilityConfiguration {
    pub source_uuid: String,
    pub station_id: u32,
    pub use_responsibility: bool,
    pub thread_count: usize,
}

impl TryFrom<&toml::Table> for MobilityConfiguration {
    type Error = ConfigurationError;

    fn try_from(properties: &toml::Table) -> Result<Self, Self::Error> {
        let s = MobilityConfiguration {
            source_uuid: get_mandatory_string(None, SOURCE_UUID_FIELD, properties)?,
            station_id: get_mandatory_integer::<u32>(None, STATION_ID_FIELD, properties)?,
            use_responsibility: get_mandatory_boolean(None, USE_RESPONSIBILITY_FIELD, properties)?,
            thread_count: get_mandatory_integer::<usize>(None, THREAD_COUNT_FIELD, properties)?,
        };
        Ok(s)
    }
}
