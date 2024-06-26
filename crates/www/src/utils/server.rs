use std::{fs::File, io::BufReader, path::Path};
use web_core::prelude::*;

#[cfg(feature = "tls-rustls")]
fn __generate_tls_rustls_config<T>(key_file_path: T, cert_file_path: T) -> Result<rustls::ServerConfig>
where
    T: AsRef<Path>,
{
    // Load ssl keys.
    let mut key_file = BufReader::new(File::open(key_file_path.as_ref())?);
    let key_der = rustls_pemfile::private_key(&mut key_file)?.ok_or_else(|| anyhow!("Failed to load ssl key file."))?;

    let mut cert_file = BufReader::new(File::open(cert_file_path.as_ref())?);
    let cert_chain = rustls_pemfile::certs(&mut cert_file).map(|c| c.unwrap()).collect();

    Ok(rustls::ServerConfig::builder().with_no_client_auth().with_single_cert(cert_chain, key_der)?)
}

#[cfg(feature = "tls-rustls")]
pub fn generate_tls_rustls_config<T>(key_file_path: T, cert_file_path: T) -> Result<rustls::ServerConfig>
where
    T: AsRef<Path>,
{
    __generate_tls_rustls_config(key_file_path, cert_file_path).map_err(|e: anyhow::Error| {
        eprintln!("Tls Error:{}", Into::<rustls::Error>::into(rustls::CertificateError::BadSignature));

        e
    })
}
