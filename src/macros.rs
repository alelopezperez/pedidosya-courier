//! Internal macros for repetitive type definitions.

/// Defines a string-backed enum that never fails to deserialize.
///
/// The PedidosYa API documents closed sets of values, but its own examples
/// already contain values missing from the spec (for example the
/// `CROSS_DOCKING` delivery mode). Unrecognized values are kept in an
/// `Unknown(String)` variant instead of failing the whole response.
macro_rules! string_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $(#[$vmeta:meta])*
                $variant:ident => $value:literal,
            )+
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        #[non_exhaustive]
        $vis enum $name {
            $(
                $(#[$vmeta])*
                $variant,
            )+
            /// A value not known to this version of the crate. The raw string
            /// is kept so it can still be inspected, logged or sent back.
            Unknown(String),
        }

        impl $name {
            /// Returns the value as sent over the wire, e.g. `"CONFIRMED"`.
            pub fn as_str(&self) -> &str {
                match self {
                    $(Self::$variant => $value,)+
                    Self::Unknown(other) => other,
                }
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl ::std::convert::From<&str> for $name {
            fn from(value: &str) -> Self {
                match value {
                    $($value => Self::$variant,)+
                    other => Self::Unknown(other.to_owned()),
                }
            }
        }

        impl ::std::convert::From<String> for $name {
            fn from(value: String) -> Self {
                match value.as_str() {
                    $($value => Self::$variant,)+
                    _ => Self::Unknown(value),
                }
            }
        }

        impl ::serde::Serialize for $name {
            fn serialize<S: ::serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(self.as_str())
            }
        }

        impl<'de> ::serde::Deserialize<'de> for $name {
            fn deserialize<D: ::serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                String::deserialize(deserializer).map(Self::from)
            }
        }
    };
}

/// Defines a string identifier newtype, so different kinds of IDs can't be
/// mixed up (C-NEWTYPE).
macro_rules! id_type {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, ::serde::Serialize, ::serde::Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            /// Wraps a raw identifier.
            pub fn new(id: impl Into<String>) -> Self {
                Self(id.into())
            }

            /// Returns the identifier as a string slice.
            pub fn as_str(&self) -> &str {
                &self.0
            }

            /// Consumes the wrapper and returns the raw identifier.
            pub fn into_string(self) -> String {
                self.0
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl ::std::convert::AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }

        impl ::std::convert::From<String> for $name {
            fn from(id: String) -> Self {
                Self(id)
            }
        }

        impl ::std::convert::From<&str> for $name {
            fn from(id: &str) -> Self {
                Self(id.to_owned())
            }
        }

        impl ::std::convert::From<$name> for String {
            fn from(id: $name) -> Self {
                id.0
            }
        }
    };
}

pub(crate) use id_type;
pub(crate) use string_enum;
