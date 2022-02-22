
pub type Guid = [u8; 16];

pub fn new_guid(a: u32, b: u32, c: u32, d: u32) -> Guid {
    [
        a & 0xff, (a >> 8) & 0xff, (a >> 16) & 0xff, (a >> 24) & 0xff,
        b & 0xff, (b >> 8) & 0xff, (b >> 16) & 0xff, (b >> 24) & 0xff,
        c & 0xff, (c >> 8) & 0xff, (c >> 16) & 0xff, (c >> 24) & 0xff,
        d & 0xff, (d >> 8) & 0xff, (d >> 16) & 0xff, (d >> 24) & 0xff
    ]
}

#[derive(Debug)]
pub struct GenerationInfo {
    export_count: i32,
    name_count: i32,
}

#[derive(Debug, Default)]
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

#[derive(Debug)]
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