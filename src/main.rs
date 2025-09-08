use reqwest;

use std::env;

fn main() {
   let user = env::var("USER").expect("Could not find user environment variable");

   
   println!("{:?}", user);
}
