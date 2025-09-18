use std::mem::MaybeUninit;

#[derive(Debug)]
pub struct Target {
   pub namespace: Box<str>,
   pub deployment_name: Box<str>,
   pub deployment_uid: MaybeUninit<Box<str>>,
   pub containers: Option<Vec<Box<str>>>,
}


#[derive(Debug)]
pub struct TargetInput {
   pub namespace: Box<str>,
   pub deployment_name: Box<str>,
   pub containers: Vec<Box<str>>,
}
