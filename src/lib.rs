mod error;
mod parsers;

pub use crate::{error::Result, parsers::plugin::Plugin};

use crate::parsers::plugin::plugin;

use std::io::{BufReader, Read};

pub fn read_plugin<R>(readable: R) -> Result<Plugin>
where
    R: std::io::Read,
{
    let mut reader = BufReader::new(readable);
    let mut bytes = vec![];
    reader.read_to_end(&mut bytes)?;

    let (remaining, plugin) = plugin(&bytes).or(Err(error::Error::new(error::ErrorKind::NomError)))?;
    let bytes_remaining = remaining.iter().cloned().collect::<Vec<u8>>().len();

    if bytes_remaining > 0 {
        Err(error::Error::new(error::ErrorKind::UnconsumedBytes(bytes_remaining)))
    } else {
        Ok(plugin)
    }
}

#[cfg(test)]
mod tests {
    use super::{read_plugin, Plugin};

    use ctor::ctor;
    use env_logger;
    use lazy_static::lazy_static;
    use log::info;

    use std::fs::File;

    lazy_static! {
        static ref SKYRIM_PLUGIN: Plugin = {
            info!("Loading Skyrim.esm");

            let plugin = read_plugin(File::open("data/Skyrim.esm").unwrap()).unwrap();

            info!("Skyrim.esm loaded");
            plugin
        };
    }

    lazy_static! {
        static ref DAWNGUARD_PLUGIN: Plugin = {
            info!("Loading Dawnguard.esm");

            let plugin = read_plugin(File::open("data/Dawnguard.esm").unwrap()).unwrap();

            info!("Dawnguard.esm loaded");
            plugin
        };
    }

    #[ctor]
    fn init_logs() {
        env_logger::init();
    }

    #[test]
    fn test_header_magic() {
        assert_eq!(&SKYRIM_PLUGIN.tes4.header.code.to_string(), "TES4");
        assert_eq!(&DAWNGUARD_PLUGIN.tes4.header.code.to_string(), "TES4");
    }
}
