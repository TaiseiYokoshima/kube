use std::collections::HashSet;
use std::collections::HashMap;

pub type PodSet = HashSet<Box<str>>;


#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct NamespaceMap(HashMap<Box<str>, PodSet>);

use std::ops::Deref;
use std::ops::DerefMut;


impl Deref for NamespaceMap {
    type Target = HashMap<Box<str>, HashSet<Box<str>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for NamespaceMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}


impl NamespaceMap {
   pub fn new() -> Self {
      Self(HashMap::new())
   }
}


#[derive(Debug, Clone)]
pub struct Pods {
   // for each replica set we have to grab resource versoin
   // for some reason the api did not allow to resue it
   pub versions: HashMap<Box<str>, Box<str>>,
   pub map: NamespaceMap,
}


impl std::fmt::Display for NamespaceMap {
   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
   
      for (namespace, pods) in self.0.iter() {
         writeln!(f, "for namespace \"{}\":", namespace)?;
         pods.iter().enumerate().try_for_each(|(i, pod)| writeln!(f, "{}: \"{pod}\"", i + 1))?;
      };

      Ok(())
   }
}
