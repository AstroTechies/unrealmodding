#[macro_export]
macro_rules! bake_instructions {
    ($($name:literal : $instructions:expr),*) => {
        {
            let mut instructions = serde_json::Map::new();
            $(
                instructions.insert($name.to_string(), serde_json::json!($instructions));
            )*
            instructions
        }
    };
}
