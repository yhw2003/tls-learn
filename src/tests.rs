mod tests {
    use std::{fs::File, io::{self, BufReader}, net::{Ipv4Addr, SocketAddrV4}, path::Path, sync::Arc, time::Duration};

    use rustls_pemfile::{certs, read_one, Item};
    use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{TcpListener, TcpStream}, time::sleep};
    use tokio_rustls::{rustls::{pki_types::{CertificateDer, PrivatePkcs8KeyDer, ServerName}, ClientConfig, RootCertStore, ServerConfig}, TlsAcceptor, TlsConnector};


    #[tokio::test]
    async fn try_tls() {
        let (hdl1, hdl2)
            = (
                tokio::spawn(run_server()),
                tokio::spawn(run_client("Hello, world!".to_string()))
            );
        assert_eq!("Hello, world!".to_string(), hdl1.await.unwrap());
        assert_eq!("Hello, world!Hello, world!".to_string(), hdl2.await.unwrap());
    }

    async fn run_server() -> String {
        let certs = {
            let path = Path::new("certs/server.crt");
            load_certs(path).unwrap()
        };
        let key = {
            let path = Path::new("certs/server.key");
            load_keys(path).unwrap()
        };
        let tls_server_cfg = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key.into())
            .unwrap();
        let socket_addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 12323);

        let tls_server_cfg = Arc::new(tls_server_cfg);
        let acceptor = TlsAcceptor::from(tls_server_cfg);
        let listener = TcpListener::bind(socket_addr).await.unwrap();
        let (tcp_stream, _) = listener.accept().await.unwrap();
        let mut tls_stream = acceptor.accept(tcp_stream).await.unwrap();
        let mut buffer = vec![];
        // let mut buffer = String::new();
        tls_stream.read_buf(&mut buffer).await.unwrap();
        unsafe {
            tls_stream.write_all(format!("{}{}", String::from_utf8_unchecked(buffer.clone()), String::from_utf8_unchecked(buffer.clone())).as_bytes()).await.unwrap();
            tls_stream.shutdown().await.unwrap();
            return  String::from_utf8_unchecked(buffer);
        }
    }

    async fn run_client(content: String) -> String {
        // wait for server to start
        sleep(Duration::from_secs(1)).await;
        let mut root_cert_store =RootCertStore::empty();
        root_cert_store.add({
            let path = Path::new("certs/ca.crt");
            load_certs(path).unwrap()[0].clone()
        }).unwrap();
        let client_config = ClientConfig::builder()
            .with_root_certificates(root_cert_store)
            .with_no_client_auth();
        let client_config = Arc::new(client_config);
        let connector = TlsConnector::from(client_config);
        let tcp_stream = TcpStream::connect("127.0.0.1:12323").await.unwrap();
        let mut tls_stream = connector.connect(ServerName::DnsName("server".try_into().unwrap()), tcp_stream).await.unwrap();
        let mut buffer = vec![];
        tls_stream.write_all(content.clone().as_bytes()).await.unwrap();
        tls_stream.read_buf(&mut buffer).await.unwrap();
        tls_stream.shutdown().await.unwrap();
        return String::from_utf8(buffer).unwrap();
    }

    fn load_certs(path: &Path) -> io::Result<Vec<CertificateDer<'static>>> {
        let file = File::open(path).unwrap();
        let mut reader = BufReader::new(file);
        certs(&mut reader).collect()
    }

    fn load_keys(path: &Path) -> io::Result<PrivatePkcs8KeyDer<'static>> {
        let mut binding = BufReader::new(File::open(path).unwrap());
        let key = read_one(&mut binding).unwrap().unwrap();
        return match key {
            Item::Pkcs8Key(key) => Ok(key),
            _=> {
                panic!("key not found, found format {:?}", key);
            }
        };
    }


}