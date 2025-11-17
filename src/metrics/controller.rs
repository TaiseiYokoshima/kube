use std::collections::{HashMap, HashSet};

use crate::client::{CAdvisorDaemonSetMetadata, CAdvisorPods, KubeClient, Pod};


struct MetricCollector {
   map: HashMap<Box<str>, Vec<(f64, f64)>>,
   set: HashSet<Box<str>>,
   min: f64,
}






async fn query_task(pod: &Pod, client: KubeClient, ) {

   let query = || client.proxy.pod(pod, "metrics");

   
   loop {
      let x = query().await;

   }
}







pub async fn test(pods_meta: CAdvisorDaemonSetMetadata, pods_state: CAdvisorPods, client: KubeClient) {


   


   let watcher = client.watch.daemon_set_pods(pods_meta, pods_state, std::time::Duration::from_secs(60));






}

