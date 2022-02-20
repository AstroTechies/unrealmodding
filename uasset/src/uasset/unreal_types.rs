
pub type Guid = [u8; 16];
#[derive(Debug)]
pub struct CustomVersion {
    guid: Guid,
    version: i32,
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