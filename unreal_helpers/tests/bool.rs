#![cfg(feature = "read_write")]

use std::io::Cursor;

use unreal_helpers::{error::FStringError, UnrealReadExt, UnrealWriteExt};

#[test]
fn test_read_bool() -> Result<(), FStringError> {
    let mut cursor = Cursor::new(vec![0u8, 1u8]);
    let (first, second) = (cursor.read_bool()?, cursor.read_bool()?);

    assert!(!first);
    assert!(second);

    Ok(())
}

#[test]
fn test_write_bool() -> Result<(), FStringError> {
    let mut cursor = Cursor::new(Vec::new());
    cursor.write_bool(false)?;
    cursor.write_bool(true)?;

    assert_eq!(cursor.get_ref(), &[0u8, 1u8]);

    Ok(())
}
