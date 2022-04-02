use std::collections::HashMap;

pub type Guid = [u8; 16];

pub fn new_guid(a: u32, b: u32, c: u32, d: u32) -> Guid {
    [
        (a & 0xff) as u8, ((a >> 8) & 0xff) as u8, ((a >> 16) & 0xff) as u8, ((a >> 24) & 0xff) as u8,
        (b & 0xff) as u8, ((b >> 8) & 0xff) as u8, ((b >> 16) & 0xff) as u8, ((b >> 24) & 0xff) as u8,
        (c & 0xff) as u8, ((c >> 8) & 0xff) as u8, ((c >> 16) & 0xff) as u8, ((c >> 24) & 0xff) as u8,
        (d & 0xff) as u8, ((d >> 8) & 0xff) as u8, ((d >> 16) & 0xff) as u8, ((d >> 24) & 0xff) as u8
    ]
}

#[derive(Debug)]
pub struct GenerationInfo {
    pub export_count: i32,
    pub name_count: i32,
}

#[derive(Debug, Default, Hash, PartialEq, Eq, Clone)]
pub struct FName {
    pub content: String,
    pub index: i32
}

impl FName {
    pub fn new(content: String, index: i32) -> Self {
        FName {
            content, index
        }
    }
}

#[derive(Debug, Default)]
pub struct NamespacedString {
    pub namespace: String,
    pub value: String
}

impl NamespacedString {
    pub fn new(namespace: String, value: String) -> Self {
        NamespacedString {
            namespace,
            value
        }
    }
}

#[derive(Debug)]
pub struct StringTable {
    pub namespace: String,
    pub value: HashMap<String, String>
}

impl StringTable {
    pub fn new(namespace: String) -> Self {
        StringTable {
            namespace,
            value: HashMap::new()
        }
    }
}