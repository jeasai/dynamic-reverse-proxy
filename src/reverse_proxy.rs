use std::{net::SocketAddr, path::Path, sync::Arc};

use anyhow::bail;
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpSocket, TcpStream},
};
use tokio_rustls::{
    rustls::{
        pki_types::{pem::PemObject, CertificateDer, PrivateKeyDer},
        ServerConfig,
    },
    server::TlsStream,
    TlsAcceptor,
};

pub async fn start(
    listen_addr: &SocketAddr,
    cert_path: &Path,
    key_path: &Path,
) -> anyhow::Result<()> {
    // Loads certificates

    let certs = CertificateDer::pem_file_iter(cert_path)?.collect::<Result<Vec<_>, _>>()?;
    let key = PrivateKeyDer::from_pem_file(key_path)?;

    // Configures TLS acceptor, binds TCP listener

    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;

    let tls_acceptor = TlsAcceptor::from(Arc::new(config));

    let listener = TcpListener::bind(listen_addr).await?;

    loop {
        // Accepts a connection

        let (stream, peer_addr) = listener.accept().await?;
        let acceptor = tls_acceptor.clone();

        // Handles the connection

        tokio::spawn(async move {
            let _ = handle_connection(peer_addr, acceptor, stream).await;
        });
    }
}

pub async fn handle_connection(
    _peer_addr: SocketAddr,
    acceptor: TlsAcceptor,
    stream: TcpStream,
) -> anyhow::Result<()> {
    // Accepts the TLS connection

    let tls_stream = acceptor.accept(stream).await.unwrap();

    // Extract SNI

    let Some(sni) = tls_stream.get_ref().1.server_name() else {
        bail!("No SNI !");
    };

    // Gets the destination based on the SNI from the state of the app

    let Some(addr) = crate::state::get_sni_endpoint(sni).await else {
        bail!("No endpoint for {sni}");
    };

    // Opens a socket to the destination

    let sock: TcpSocket = TcpSocket::new_v4()?;

    sock.set_reuseaddr(true)?;
    sock.set_keepalive(true)?; // For NAT traversal

    let tcp_stream = sock.connect(addr).await?;

    // Writes the content of the TCP stream in the TLS one and vice versa

    pipe_stream(tls_stream, tcp_stream).await?;

    Ok(())
}

pub async fn pipe_stream(
    tls_stream: TlsStream<TcpStream>,
    tcp_stream: TcpStream,
) -> anyhow::Result<()> {
    let (mut tls_reader, mut tls_writer) = io::split(tls_stream);
    let (mut tcp_reader, mut tcp_writer) = io::split(tcp_stream);

    let tls_to_tcp = async {
        let mut buffer = [0; 1024];
        while let Ok(n) = tls_reader.read(&mut buffer).await {
            if n == 0 {
                break;
            }
            if tcp_writer.write_all(&buffer[..n]).await.is_err() {
                return Err(io::Error::new(io::ErrorKind::Other, "TCP write error"));
            }
        }
        Ok::<_, io::Error>(())
    };

    let tcp_to_tls = async {
        let mut buffer = [0; 1024];
        while let Ok(n) = tcp_reader.read(&mut buffer).await {
            if n == 0 {
                break;
            }
            if tls_writer.write_all(&buffer[..n]).await.is_err() {
                return Err(io::Error::new(io::ErrorKind::Other, "TLS write error"));
            }
        }
        Ok::<_, io::Error>(())
    };

    tokio::try_join!(tls_to_tcp, tcp_to_tls)?;

    Ok(())
}
