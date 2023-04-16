#![cfg(feature = "read_write")]

use std::io::Cursor;

use unreal_helpers::{error::FStringError, UnrealReadExt, UnrealWriteExt};

#[test]
fn test_read_fstring() -> Result<(), FStringError> {
    // ASCII
    let mut cursor = Cursor::new(vec![5u8, 0u8, 0u8, 0u8, b't', b'e', b's', b't', 0u8]);
    let maybe_string = cursor.read_fstring()?;
    assert_eq!(maybe_string, Some("test".to_string()));

    // Non-ASCII
    let mut cursor = Cursor::new(vec![0xfeu8, 0xffu8, 0xffu8, 0xffu8, 0xa7u8, 0u8, 0u8, 0u8]);
    let maybe_string = cursor.read_fstring()?;
    assert_eq!(maybe_string, Some("\u{A7}".to_string()));

    // Null
    let mut cursor = Cursor::new(vec![0u8; 4]);
    let maybe_string = cursor.read_fstring()?;
    assert_eq!(maybe_string, None);

    // Missing null terminator
    let mut cursor = Cursor::new(vec![1u8, 0u8, 0u8, 0u8, b't']);
    let err = cursor.read_fstring().expect_err("Expected err");
    assert!(matches!(err, FStringError::InvalidStringTerminator(116, 5)));

    // Missing null terminator, UTF-16
    let mut cursor = Cursor::new(vec![0xffu8, 0xffu8, 0xffu8, 0xffu8, b't', b'e']);
    let err = cursor.read_fstring().expect_err("Expected err");
    assert!(matches!(
        err,
        FStringError::InvalidStringTerminator(25972, 6)
    ));

    Ok(())
}

#[test]
fn test_write_fstring() -> Result<(), FStringError> {
    // ASCII
    let mut cursor = Cursor::new(Vec::new());
    cursor.write_fstring(Some("test"))?;
    assert_eq!(
        cursor.get_ref(),
        &[5u8, 0u8, 0u8, 0u8, b't', b'e', b's', b't', 0u8],
    );

    // Non-ASCII
    let mut cursor = Cursor::new(Vec::new());
    cursor.write_fstring(Some("\u{A7}"))?;
    assert_eq!(
        cursor.get_ref(),
        &[0xfeu8, 0xffu8, 0xffu8, 0xffu8, 0xa7u8, 0u8, 0u8, 0u8],
    );

    // Null
    let mut cursor = Cursor::new(Vec::new());
    cursor.write_fstring(None)?;
    assert_eq!(cursor.get_ref(), &[0u8; 4]);

    Ok(())
}
