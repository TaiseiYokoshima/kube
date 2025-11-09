#[derive(Debug, Clone)]
pub struct Nodes {
   pub names: Vec<Box<str>>,
   pub statuses: Vec<bool>,
   pub version: Box<str>,
}
