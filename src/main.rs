use reqwest::{Certificate, Client, Identity};

fn read_config() -> (String, String, String, String) {
   use serde_yaml::Value;
   use std::fs;

   let user = std::env::var("USER").unwrap();
   let path = format!("/home/{}/.kube/config", user);
   let s = fs::read_to_string(path).unwrap();
   let config: Value = serde_yaml::from_str(&s).unwrap();

   let first_cluster = config
      .get("clusters")
      .expect("no clusters")
      .as_sequence()
      .expect("not sequence")
      .get(0)
      .expect("empty sequence")
      .get("cluster")
      .expect("no cluster");

   let host = first_cluster
      .get("server")
      .expect("no server")
      .as_str()
      .expect("not str");

   let ca_cert = first_cluster
      .get("certificate-authority-data")
      .expect("no ca")
      .as_str()
      .expect("not str");

   let client_credentials = config
      .get("users")
      .expect("no users")
      .as_sequence()
      .expect("not sequences")
      .get(0)
      .expect("empty sequence")
      .get("user")
      .expect("no user");

   let client_cert = client_credentials
      .get("client-certificate-data")
      .expect("no client ca")
      .as_str()
      .expect("not str");

   let client_key = client_credentials
      .get("client-key-data")
      .expect("no client key")
      .as_str()
      .expect("not str");

   let host = host.into();
   let ca_cert = ca_cert.into();
   let client_cert = client_cert.into();
   let client_key = client_key.into();

   (host, ca_cert, client_cert, client_key)
}

fn generate_creds(
   ca_cert: String,
   client_cert: String,
   client_key: String,
) -> (Certificate, Identity) {
   use base64::{Engine, engine::general_purpose::STANDARD};

   let ca_cert = STANDARD.decode(ca_cert).unwrap();
   let client_cert = STANDARD.decode(client_cert).unwrap();
   let client_key = STANDARD.decode(client_key).unwrap();

   let ca_cert = Certificate::from_pem(&ca_cert).unwrap();

   let mut pem = vec![];
   pem.extend_from_slice(&client_cert);
   pem.extend_from_slice(&client_key);

   let identity = Identity::from_pem(&pem).unwrap();

   (ca_cert, identity)
}

#[tokio::main]
async fn main() {
   let (host, cert, ident) = {
      let (host, ca_cert, client_cert, client_key) = read_config();
      let (cert, ident) = generate_creds(ca_cert, client_cert, client_key);

      (host, cert, ident)
   };

   let client = Client::builder()
      .use_rustls_tls() // without this, it didnt use rustls and identity would fail
                                      // because identity provided here is not compatible with
                                      // native-tls
      .add_root_certificate(cert)
      .identity(ident)
      .build()
      .unwrap();


   let url = format!("{}/api/v1/namespaces/default/pods", host);

   let response = client
      .get(url)
      .send()
      .await
      .unwrap();

   let text = response.text().await.unwrap();
   println!("{}", text);
}
