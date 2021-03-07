mod components;
mod error;
mod parsers;

use crate::components::Plugin;
use crate::error::{Error, ErrorKind, Result};
use std::io::{BufReader, Read};

pub fn read_plugin<R>(readable: R) -> Result<Plugin>
where
    R: std::io::Read,
{
    let mut reader = BufReader::new(readable);
    let mut bytes = vec![];
    reader.read_to_end(&mut bytes)?;

    let (remaining, plugin) = parsers::plugin(&bytes).or(Err(error::Error::new(error::ErrorKind::NomError)))?;
    let bytes_remaining = remaining.iter()
        .cloned()
        .collect::<Vec<u8>>()
        .len();

    Ok(plugin)
}

#[cfg(test)]
mod tests {
    use crate::read_plugin;
    use std::fs::File;

    #[test]
    fn it_works() {
        let file = File::open("data/Skyrim.esm").unwrap();
        let plugin = read_plugin(file).unwrap();
        println!("{:?}", plugin);
    }
}
