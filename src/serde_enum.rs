
macro_rules! enum_number {
    ($name:ident { $($variant:ident = $value:expr, )* }) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub enum $name {
            $($variant = $value,)*
        }

        impl ::serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
                where S: ::serde::Serializer
            {
                // Serialize the enum as a u64.
                serializer.serialize_u64(*self as u64)
            }
        }

        impl ::serde::Deserialize for $name {
            fn deserialize<D>(deserializer: D) -> ::std::result::Result<Self, D::Error>
                where D: ::serde::Deserializer
            {
                use std::fmt;

                struct Visitor;

                impl ::serde::de::Visitor for Visitor {
                    type Value = $name;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("positive integer")
                    }

                    fn visit_u64<E>(self, value: u64) -> ::std::result::Result<$name, E>
                        where E: ::serde::de::Error
                    {
                        // Rust does not come with a simple way of converting a
                        // number to an enum, so use a big `match`.
                        match value {
                            $( $value => Ok($name::$variant), )*
                            _ => Err(E::custom(
                                format!("unknown {} value: {}",
                                stringify!($name), value))),
                        }
                    }
                }

                // Deserialize the enum from a u64.
                deserializer.deserialize_u64(Visitor)
            }
        }
    }
}