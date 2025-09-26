use std::collections::HashMap;

pub type PodsCount = u32;
pub type TemplateHash = Box<str>;

#[derive(Debug, Clone)]
pub struct ReplicaSets {
   pub version: Box<str>,
   pub hashes: HashMap<TemplateHash, PodsCount>,
}
