use super::{APIError, Base, ResourceVersion, Uid, response_into_error, errors, parse_json_pod};

mod get;
mod watch;

pub use get::get_daemon_set_pods;
pub use watch::{Watcher, WatcherError, DaemonSetEvent};



#[derive(Debug, Clone)]
pub struct Pod {
   pub uid: Uid,
   pub namespace: Box<str>,
   pub name: Box<str>,
   pub status: bool,
}

impl Pod {
   pub fn new(uid: Uid, namespace: Box<str>, name: Box<str>, status: bool) -> Self {
      Self { uid, namespace, name, status }
   }
}


impl std::fmt::Display for Pod {
   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      write!(f, "Pod (name: {}, uid: {}, status: {})", self.name, self.uid, self.status)
   }
}


#[derive(Debug, Clone)]
pub struct CAdvisorPods
{
    pub pods:  Vec<Pod>,
    pub version: ResourceVersion,
}

impl std::fmt::Display for CAdvisorPods {
   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

      writeln!(f, "CAdvisorPods | version: {}: [", self.version)?;
      for pod in self.pods.iter() {
         writeln!(f, "\t{pod:#}",)?;
      };
      writeln!(f, "]")
   }

}


impl CAdvisorPods {
   fn get(&self, pod: &Pod) -> Option<&Pod> {
      self.pods.iter().find(|x| x.uid == pod.uid)
   }

   fn get_mut_by_uid(&mut self, uid: &Box<str>) -> Option<&mut Pod> {
      self.pods.iter_mut().find(|x| x.uid == *uid)
   }

   fn get_by_uid(&self, uid: &Box<str>) -> Option<&Pod> {
      self.pods.iter().find(|x| x.uid == *uid)
   }

   fn uids(&self) -> impl Iterator<Item = &Box<str>> {
      self.pods.iter().map(|x| &x.uid)
   }
   
   fn get_mut(&mut self, pod: &Pod) -> Option<&mut Pod> {
      self.pods.iter_mut().find(|x| x.uid == pod.uid)
   }

   fn remove(&mut self, pod: &Pod) -> Option<Pod> {
      let index = self.pods.iter().position(|x| x.uid == pod.uid)?;
      Some(self.pods.remove(index))
   }

   fn insert(&mut self, pod: Pod) -> Option<Pod> {
      let old = self.remove(&pod);
      self.pods.push(pod);
      old
   }
}


#[derive(Debug)]
pub struct CAdvisorDaemonSetMetadata
{
    pub key: Box<str>,
    pub value: Box<str>,
    pub namespace: Box<str>,
}

impl CAdvisorDaemonSetMetadata
{
    pub fn new(
        key: &str,
        value: &str,
        namespace: &str,
    ) -> Self
    {
        let key = key.into();
        let value = value.into();
        let namespace = namespace.into();

        Self {
            key,
            value,
            namespace,
        }
    }
}
