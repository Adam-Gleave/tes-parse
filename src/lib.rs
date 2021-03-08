mod error;
mod parsers;

use crate::error::Result;
use crate::parsers::plugin;
use std::io::{BufReader, Read};

pub fn read_plugin<R>(readable: R) -> Result<plugin::Plugin>
where
    R: std::io::Read,
{
    let mut reader = BufReader::new(readable);
    let mut bytes = vec![];
    reader.read_to_end(&mut bytes)?;

    let (remaining, plugin) = plugin::plugin(&bytes).or(Err(error::Error::new(error::ErrorKind::NomError)))?;
    let bytes_remaining = remaining.iter()
        .cloned()
        .collect::<Vec<u8>>()
        .len();

    if bytes_remaining > 0 {
        Err(error::Error::new(error::ErrorKind::UnconsumedBytes(bytes_remaining)))
    } else {
        Ok(plugin)
    }
}

#[cfg(test)]
mod tests {
    use crate::read_plugin;
    use std::fs::File;

    #[test]
    fn it_works() {
        let file = File::open("data/Skyrim.esm").unwrap();
        let plugin = read_plugin(file).unwrap();
        println!("{:#?}", plugin);

        let dawnguard_file = File::open("data/Dawnguard.esm").unwrap();
        let dawnguard_plugin = read_plugin(dawnguard_file).unwrap();
        println!("{:#?}", dawnguard_plugin);
    }
}
