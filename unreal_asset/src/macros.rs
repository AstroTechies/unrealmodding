#[macro_export]
macro_rules! inner_trait {
    ($outer_name:ty, $($inner:ident),*) => {
        impl Hash for $outer_name {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                match self {
                    $(
                        Self::$inner(inner) => inner.hash(state),
                    )*
                }
            }
        }

        impl PartialEq for $outer_name {
            fn eq(&self, other: &Self) -> bool {
                match (self, other) {
                    $(
                        (Self::$inner(l0), Self::$inner(r0)) => l0 == r0,
                    )*
                    _ => false
                }
            }
        }

        impl Clone for $outer_name {
            fn clone(&self) -> Self {
                match self {
                    $(
                        Self::$inner(arg0) => Self::$inner(arg0.clone()),
                    )*
                }
            }
        }

        impl Debug for $outer_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        Self::$inner(arg0) => f.debug_tuple(stringify!($inner)).field(arg0).finish(),
                    )*
                }
            }
        }
    };
}
