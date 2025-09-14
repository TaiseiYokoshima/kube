use std::fmt::Display;
use reqwest::{Client, RequestBuilder};


pub struct K8SClient {
   base: String,
   client: Client,
}



impl K8SClient {
   pub fn new(client: Client, base: String) -> Self {
      Self {
         client,
         base
      }
   }

   pub fn get(&self, endpoint: impl AsRef<str> + Display) -> RequestBuilder {
      let url = format!("{}{}", self.base, endpoint);
      self.client.get(url)
   }

   pub fn post(&self, endpoint: impl AsRef<str> + Display) -> RequestBuilder {
      let url = format!("{}{}", self.base, endpoint);
      self.client.post(url)
   }
}
