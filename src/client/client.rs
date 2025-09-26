use std::rc::Rc;
use super::{ClientError, ReplicaSets, Nodes, Pods};

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
   pub async fn nodes(&self) -> Result<Nodes, ClientError> {
      use super::get_nodes;
      let client = &self.client;
      get_nodes::get_nodes(client).await
   }
   
   pub async fn replica_sets(&self, targets: &Vec<Target>) -> Result<ReplicaSets, ClientError> {
      use super::get_replicasets::get_replica_sets;
      let client = &self.client;
      get_replica_sets(client, targets).await
   }


   pub async fn pods(&self, replica_sets: &ReplicaSets) -> Result<Pods, ClientError> {
      use super::get_pods::get_pods;

      let client: &Base = &self.client;
      get_pods(client, replica_sets).await
   }
}

#[derive(Debug, Clone)]
pub struct Watch {
   pub client: Rc<Base>,
}


use super::watchers::{Watcher, ReplicaSetEvent};
use k8s_openapi::api::apps::v1::ReplicaSet;

use std::time::Duration;

impl Watch {
   pub fn replica_sets(&self, targets: Vec<Target>, replica_sets: ReplicaSets, timeout: Duration) -> Watcher<ReplicaSet, ReplicaSetEvent> {
      let client = (*self.client).clone();
      Watcher::new(client, replica_sets, targets, timeout)
   }
}

#[derive(Debug, Clone)]
pub struct Validate {
   client: Rc<Base>,
}

use super::{TargetInput, Target};

impl Validate {
   pub async fn targets(
       &self,
      targets: Vec<TargetInput>,
   ) -> Result<Vec<Target>, ClientError> {

      let client = &self.client;

      use super::validate_targets::validate_targets;
      let result = validate_targets(client, targets).await?;

      match result {
         Ok(targets) => Ok(targets),
         Err(errors) => Err(ClientError::TargetValidationError(errors)),
      }
   }
}

#[derive(Debug, Clone)]
pub struct KubeClient {
   pub get: Get,
   pub watch: Watch,
   pub validate: Validate,
}

impl KubeClient {
   pub fn new() -> Result<Self, ClientError> {

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
      let validate = Validate { client: base };

      Ok(Self {
         get,
         watch,
         validate,
      })
   }
}
