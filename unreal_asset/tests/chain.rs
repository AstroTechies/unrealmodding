use std::io::{Read, Seek, SeekFrom};
use unreal_asset_base::containers::Chain;

#[test]
fn read() {
    use std::io::Cursor;
    let mut v = Vec::with_capacity(12);
    Chain::new(
        Cursor::new(vec![0, 1, 2, 3, 4, 5, 6, 7]),
        Some(Cursor::new(vec![0, 1, 2, 3])),
    )
    .read_to_end(&mut v)
    .unwrap();
    assert_eq!(v, [0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3]);
}

#[test]
fn seek() {
    use std::io::Cursor;
    let mut chain = Chain::new(
        Cursor::new(vec![0, 1, 2, 3]),
        Some(Cursor::new(vec![4, 5, 6, 7])),
    );
    let mut read_at = |pos| {
        use byteorder::ReadBytesExt;
        use Seek;
        chain.seek(pos)?;
        chain.read_u8()
    };
    assert_eq!(read_at(SeekFrom::Start(0)).unwrap(), 0);
    assert!(read_at(SeekFrom::Start(8)).is_err());
    assert_eq!(read_at(SeekFrom::Current(-1)).unwrap(), 7);
    assert_eq!(read_at(SeekFrom::Current(-5)).unwrap(), 3);
    assert_eq!(read_at(SeekFrom::End(-4)).unwrap(), 4);
    assert!(read_at(SeekFrom::End(-12)).is_err());
}
