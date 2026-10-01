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

use crate::client::configuration::Configuration;
use crate::exchange::Exchange;
use crate::transport::mqtt::topic::Topic;
use crate::transport::mqtt::geo_topic::GeoTopic;
use crate::transport::packet::Packet;

use crate::exchange::sequence_number::SequenceNumber;
use std::sync::{Arc, RwLock};
use serde_json::Value;

pub trait AgnosticAnalyzer<T: Topic, C> {
    fn new(
        configuration: Arc<Configuration>,
        context: Arc<RwLock<C>>,
        sequence_number: Arc<RwLock<SequenceNumber>>,
    ) -> Self
    where
        Self: Sized;

    fn analyze(&mut self, packet: Packet<T, Value>) -> Vec<Packet<GeoTopic, Exchange>>;
}
