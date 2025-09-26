#[derive(Debug, Clone)]
pub struct Target {
   pub namespace: Box<str>,
   pub deployment_name: Box<str>,
   pub deployment_uid: Box<str>,
   pub containers: Option<Vec<Box<str>>>,
}

#[derive(Debug, Clone)]
pub struct TargetInput {
   pub namespace: Box<str>,
   pub deployment_name: Box<str>,
   pub containers: Vec<Box<str>>,
}


impl TargetInput {
   pub fn new<N, D, C, I>(namespace: N, deployment_name: D, containers: C) -> Self
   where 
      N: AsRef<str>,
      D: AsRef<str>,
      I: AsRef<str>,
      C: IntoIterator<Item = I>
   {
      
      let namespace = namespace.as_ref().into();
      let deployment_name = deployment_name.as_ref().into();
      let containers = containers.into_iter().map(|value| value.as_ref().into()).collect();

      Self {
         namespace,
         deployment_name,
         containers
      }
   }
}
