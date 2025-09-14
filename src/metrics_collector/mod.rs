use k8s_openapi::serde_json;
use tokio::time::{
   sleep,

};

use tokio::sync::mpsc;

static PODNAME: &'static str = "stress-test";



pub async fn extract_cpu_metrics(data: String) {
   use serde_json::Value;

   let mut json: Value = serde_json::from_str(&data).unwrap();
   let pods = json.get_mut("pods").unwrap().as_array().unwrap().iter().filter(|pod| {
      let pod_ref = pod.get("podRef").unwrap();
      let pod_name = pod_ref.get("name").unwrap().as_str().unwrap();

      pod_name.starts_with(PODNAME)
   });
   

   



}

pub async fn collector() {
   let (receiver, sender) = mpsc::channel::<String>(10000);

   loop {





   }
}


pub struct MetricsCollector {
   nodes: Vec<String>,
   status: Vec<bool>,
   handles: Vec<Option<Box<dyn Future<Output = ()>>>>,
}

impl MetricsCollector {
   pub fn new() -> Self {
      Self {
         nodes: Vec::new(),
         status: Vec::new(),
         handles: Vec::new(),
      }
   }




   pub async fn new_collector_task(&mut self) {


   }

   
   pub async fn modified(&mut self, node: impl AsRef<str>, status: bool) {
      let i = self.nodes.iter().position(|node_name| node_name.as_str() == node.as_ref()).unwrap();
      let prev_status = self.status.get(i).unwrap();

      if *prev_status == status {
         return;
      };

      if !status {
         self.handles[i] = None;
         return

      }


      

   }



}
