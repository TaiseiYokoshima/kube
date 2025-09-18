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


#[derive(Debug, Clone)]
pub enum Kind {
   Namespace,
   Deployment,
   Containers(Vec<usize>)
}

#[derive(Debug, Clone)]
pub struct TargetError {
   pub kind: Kind,
   pub target: TargetInput,
}

