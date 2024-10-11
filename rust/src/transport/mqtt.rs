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

use std::io::{BufReader, Cursor};
use std::sync::Arc;
use rumqttc::v5::MqttOptions;
use rumqttc::{TlsConfiguration, Transport};
use rumqttc::tokio_rustls::rustls::client::danger::{DangerousClientConfig, DangerousClientConfigBuilder};
use rumqttc::tokio_rustls::rustls::client::WebPkiServerVerifier;
use rumqttc::tokio_rustls::rustls::{ClientConfig, RootCertStore};
use rumqttc::tokio_rustls::rustls::pki_types::CertificateDer;

use rumqttc::tokio_rustls::rustls::pki_types::pem::PemObject;
use rustls_pki_types::pem::SectionKind;


pub mod mqtt_client;
pub mod mqtt_router;
pub mod topic;

#[cfg(feature = "geo_routing")]
pub mod geo_topic;

pub(crate) fn configure_transport(
    tls_configuration: Option<TlsConfiguration>,
    use_websocket: bool,
    mqtt_options: &mut MqttOptions,
) {
    match (tls_configuration, use_websocket) {
        (Some(tls), true) => {
            println!("Transport: MQTT over WebSocket; TLS enabled");
            mqtt_options.set_transport(Transport::Wss(tls));
        }
        (Some(tls), false) => {
            println!("Transport: standard MQTT; TLS enabled");
            mqtt_options.set_transport(Transport::Tls(tls));
        }
        (None, true) => {
            println!("Transport: MQTT over WebSocket; TLS disabled");
            mqtt_options.set_transport(Transport::Ws);
        }
        (None, false) => println!("Transport: standard MQTT; TLS disabled"),
    }
}

/*

TlsConfiguration::SimpleNative { ca, der, password } => {
            // println!("rumqttc routing OK for native-tls");
            let cert = native_tls::Certificate::from_pem(ca)?;
            let identity = Identity::from_pkcs12(der, password)?;
            // let _identity = Identity::from_pkcs12(der, password)?;
            native_tls::TlsConnector::builder()
                .add_root_certificate(cert)
                .identity(identity)
                .danger_accept_invalid_certs(true)
                // .danger_accept_invalid_hostnames(true)
                // .identity(identity)
                .build()?
        }


 */

pub(crate) fn configure_tls(
    ca_path: Option<String>,
    alpn: Option<Vec<Vec<u8>>>,
    client_auth: Option<(Vec<u8>, Vec<u8>)>,
) -> TlsConfiguration {
    if let Some(ca_path) = ca_path {
        let ca: Vec<u8> = std::fs::read(ca_path).expect("Failed to read TLS certificate");

        // TlsConfiguration::Simple {
        //     ca,
        //     alpn,
        //     client_auth,
        // };

        let mut root_cert_store = RootCertStore::empty();
        let crt = CertificateDer::from_pem(SectionKind::Certificate, ca).expect("Ouille");
        // let crt = CertificateDer::from(ca);
        root_cert_store.add(crt).expect("Youyouille");
        let v = WebPkiServerVerifier::builder(Arc::new(root_cert_store)).build().unwrap();

        let tls_config = ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(v)
            .with_no_client_auth();

        TlsConfiguration::Rustls(Arc::new(tls_config))
    } else {
        TlsConfiguration::default()
    }
}

#[cfg(test)]
mod tests {
    use crate::transport::mqtt::configure_tls;
    use rumqttc::TlsConfiguration;

    #[test]
    #[should_panic]
    fn configure_tls_with_invalid_path_should_return_error() {
        let _ = configure_tls(Some("unextisting/path".to_string()), None, None);
    }

    #[test]
    fn configure_default_tls() {
        let tls = configure_tls(None, None, None);

        assert!(matches!(tls, TlsConfiguration::Rustls(_)));
    }
}
