/*#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}*/

pub mod uasset {
    use std::io::{Cursor, Error, ErrorKind, Read};

    const UE4_ASSET_MAGIC: u32 = u32::from_be_bytes([0xc1, 0x83, 0x2a, 0x9e]);

    #[derive(Debug)]
    pub struct Asset {
        cursor: Cursor<Vec<u8>>,
    }

    impl Asset {
        pub fn new(raw_data: Vec<u8>) -> Self {
            Asset {
                cursor: Cursor::new(raw_data),
            }
        }

        pub fn parse_data(&mut self) -> Result<(), Error> {
            println!("Parsing data...");

            // reuseable buffers for reading
            let mut buf4 = [0u8; 4];
            //let mut buf8 = [0u8; 8];

            // read and check magic
            self.cursor.read_exact(&mut buf4)?;
            if u32::from_be_bytes(buf4) != UE4_ASSET_MAGIC {
                return Err(Error::new(
                    ErrorKind::Other,
                    "File is not a valid uasset file",
                ));
            }

            println!(
                "data (len: {:?}): {:1X?}",
                self.cursor.get_ref().len(),
                self.cursor.get_ref()
            );

            Ok(())
        }
    }
}
