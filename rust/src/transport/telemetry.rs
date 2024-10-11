/*
 * Software Name : libits
 * SPDX-FileCopyrightText: Copyright (c) Orange SA
 * SPDX-License-Identifier: MIT
 *
 * This software is distributed under the MIT license,
 * see the "LICENSE.txt" file for more details or https://opensource.org/license/MIT/
 *
 * Authors: see CONTRIBUTORS.md
 * Software description: This Intelligent Transportation Systems (ITS) [MQTT](https://mqtt.org/) library based on the [JSon](https://www.json.org) [ETSI](https://www.etsi.org/committee/its) specification transcription provides a ready to connect project for the mobility (connected and autonomous vehicles, road side units, vulnerable road users,...).
 */

use std::fs::File;
use std::io::Read;
use std::time::Duration;

use opentelemetry::global::BoxedSpan;
use opentelemetry::propagation::{Extractor, TextMapPropagator};
use opentelemetry::trace::{Link, TraceContextExt, TraceError, Tracer};
use opentelemetry::{global, Context, KeyValue};
use opentelemetry_otlp::{HttpExporterBuilder, WithExportConfig};
use opentelemetry_sdk::export::trace::SpanExporter;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::runtime;
use opentelemetry_sdk::trace::{
    BatchConfigBuilder, BatchSpanProcessor, RandomIdGenerator, Sampler, TracerProvider,
};
use opentelemetry_sdk::Resource;
use reqwest::header;

use crate::client::configuration::telemetry_configuration::TelemetryConfiguration;

fn http_exporter_from_configuration(configuration: &TelemetryConfiguration) -> HttpExporterBuilder {
    let mut builder = reqwest::ClientBuilder::new();

    if let Some(basic_auth_header) = &configuration.basic_auth_header() {
        let mut headers = header::HeaderMap::new();
        let mut auth_value = header::HeaderValue::try_from(basic_auth_header)
            .expect("Failed to create header value");
        auth_value.set_sensitive(true);
        headers.insert(header::AUTHORIZATION, auth_value);
        builder = builder.default_headers(headers)
    }

    if let Some(ca_path) = &configuration.ca_path {
        let mut buf = Vec::new();
        File::open(ca_path)
            .expect("Failed to open certificate file")
            .read_to_end(&mut buf)
            .expect("Failed to read certificate");
        let cert =
            reqwest::Certificate::from_pem(&buf).expect("Failed to create Certificate from file");

        builder = builder
            .add_root_certificate(cert)
            .tls_sni(true)
            // .danger_accept_invalid_certs(true);
    }

    let http_client = builder
        .build()
        .expect("Failed to create telemetry HTTP client");

    opentelemetry_otlp::new_exporter()
        .http()
        .with_http_client(http_client)
        .with_endpoint(configuration.endpoint())
        .with_timeout(Duration::from_secs(3))
}

/// Registers a global TracerProvider with HTTP exporter
pub fn init_tracer(
    configuration: &TelemetryConfiguration,
    service_name: &'static str,
) -> Result<(), TraceError> {
    let http_exporter = http_exporter_from_configuration(configuration).build_span_exporter()?;

    let batch_processor = BatchSpanProcessor::builder(http_exporter, runtime::Tokio)
        .with_batch_config(
            BatchConfigBuilder::default()
                .with_max_export_batch_size(configuration.batch_size)
                .build(),
        )
        .build();

    let tracer_provider = TracerProvider::builder()
        .with_span_processor(batch_processor)
        .with_config(
            opentelemetry_sdk::trace::config()
                .with_sampler(Sampler::AlwaysOn)
                .with_id_generator(RandomIdGenerator::default())
                .with_max_events_per_span(64)
                .with_max_attributes_per_span(16)
                .with_max_events_per_span(16)
                .with_resource(Resource::new(vec![KeyValue::new(
                    "service.name",
                    service_name,
                )])),
        )
        .build();

    let _ = global::set_tracer_provider(tracer_provider);

    Ok(())
}

pub fn execute_in_span<F, E, R>(
    tracer_name: &'static str,
    span_name: &'static str,
    from: Option<&E>,
    block: F,
) -> R
where
    F: FnOnce() -> R,
    E: Extractor,
{
    let span = get_span(tracer_name, span_name, from);
    let cx = Context::current_with_span(span);
    let _guard = cx.attach();

    block()
}

pub fn get_span<E>(
    tracer_name: &'static str,
    span_name: &'static str,
    from: Option<&E>,
) -> BoxedSpan
where
    E: Extractor,
{
    let tracer = global::tracer(tracer_name);

    let span_builder = if let Some(packet) = from {
        let propagator = TraceContextPropagator::new();
        let trace_cx = propagator.extract(packet);
        let span_cx = trace_cx.span().span_context().clone();

        tracer
            .span_builder(span_name)
            .with_links(vec![Link::with_context(span_cx)])
    } else {
        tracer.span_builder(span_name)
    };

    span_builder.start(&tracer)
}
