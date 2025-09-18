use std::{rc::Rc, sync::Arc};

use super::ClientError;


#[derive(Debug, Clone)]
pub struct Base {
   host: Box<str>,
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
   // pub async fn get_nodes(&self) -> Result<(Vec<Arc<str>>, Vec<bool>, Box<str>), ClientError> {
   //    use super::get_nodes;
   //    get_nodes::get_nodes(&self.client).await
   // }
}

#[derive(Debug, Clone)]
pub struct Watch {
   client: Rc<Base>,
}

impl Watch {}

#[derive(Debug, Clone)]
pub struct Validate {
   client: Rc<Base>,
}

use super::{TargetInput, Target};

impl Validate {
   // pub async fn namespaces(
   //    client: &Rc<Base>,
   //    namespaces_to_check: impl Iterator<Item = &str>,
   // ) -> Result<Vec<bool>, ClientError> {
   //    use super::validate_namespace::validate_namespaces;
   //    validate_namespaces(client, namespaces_to_check).await
   // }
   //
   // pub async fn deployments(
   //    client: &Rc<Base>,
   //    deployments_to_check: impl Iterator<Item = (&str, &str)>,
   // ) -> Result<Vec<bool>, ClientError> {
   //    todo!()
   // }

   pub async fn targets(
       &self,
      targets: Vec<TargetInput>,
   ) -> Result<Vec<Target>, ClientError> {

      let client = &self.client;

      use super::validate_target::validate_targets;
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
