#[macro_use] extern crate error_chain;
extern crate eui48;

mod utils;
mod errors;

use utils::*;
use errors::*;

use std::env;

quick_main!(run);

fn run() -> Result<()> {
   let id = env::args().nth(1).ok_or("No provided id")?;
   let mac = get_mac(&id)?;
   println!("{}", mac);

   Ok(())
}
