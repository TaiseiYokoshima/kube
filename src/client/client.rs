use std::rc::Rc;
use std::time::Duration;

use super::{
   Watcher,
   DaemonSetEvent,
   CAdvisorDaemonSetMetadata,
   APIError, 
   CAdvisorPods
};


#[derive(Debug, Clone)]
pub struct Base {
   pub host: Box<str>,
   client: reqwest::Client,
}

impl Base {
   pub fn get(&self, endpoint: impl AsRef<str>) -> reqwest::RequestBuilder {
      let host = &self.host;
      let endpoint = endpoint.as_ref();
      let url = format!("{host}{endpoint}");
      self.client.get(url)
   }
}

#[derive(Debug, Clone)]
pub struct Get {
   client: Rc<Base>,
}

impl Get {
   pub async fn daemon_set_pods(&self, daemon_set: &CAdvisorDaemonSetMetadata) -> Result<CAdvisorPods, APIError> {
      use super::daemon_set::get_daemon_set_pods;
      let client = &self.client;
      let result = get_daemon_set_pods(client, daemon_set).await?;
      Ok(result)
   }
}

#[derive(Debug, Clone)]
pub struct Watch {
   pub client: Rc<Base>,
}


impl Watch {
   pub fn daemon_set_pods(&self, daemon_set: CAdvisorDaemonSetMetadata, state: CAdvisorPods, duration: Duration) -> Watcher<CAdvisorDaemonSetMetadata, DaemonSetEvent> {
      let client = (*self.client).clone();
      Watcher::new(client, daemon_set, state, duration)
   }
}

#[derive(Debug, Clone)]
pub struct KubeClient {
   pub get: Get,
   pub watch: Watch,
}

impl KubeClient {
   pub fn new() -> Result<Self, reqwest::Error> {

      use super::config::{read_config, generate_creds};

      let (host, ca_cert, client_cert, client_key) = read_config();
      let (certificate, identity) = generate_creds(ca_cert, client_cert, client_key);


      let client = reqwest::Client::builder()
         .use_rustls_tls()
         // without this, it didnt use rustls and identity would fail
         // because identity provided here is not compatible with
         // native-tls
         .add_root_certificate(certificate)
         .identity(identity)
         .build()?;

      let base = Base {
         client,
         host: host.into(),
      };

      let base = Rc::new(base);

      let get = Get {
         client: base.clone(),
      };
      let watch = Watch {
         client: base.clone(),
      };

      Ok(Self {
         get,
         watch,
      })
   }
}
