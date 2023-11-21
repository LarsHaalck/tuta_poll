pub mod serde_base64 {
    use base64::{engine::general_purpose as engines, Engine as _};

    pub fn deserialize<'de, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Vec<u8>, D::Error> {
        deserializer.deserialize_str(Base64Visitor)
    }

    struct Base64Visitor;

    impl<'de> serde::de::Visitor<'de> for Base64Visitor {
        type Value = Vec<u8>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "base64 string")
        }

        fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
            engines::STANDARD.decode(value).map_err(|_| {
                serde::de::Error::invalid_value(serde::de::Unexpected::Str(value), &self)
            })
        }
    }
}

pub mod serde_format {
    pub fn deserialize<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<(), D::Error> {
        deserializer.deserialize_str(FormatVisitor)
    }

    struct FormatVisitor;

    impl<'de> serde::de::Visitor<'de> for FormatVisitor {
        type Value = ();

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "string \"0\"")
        }

        fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
            if value == "0" {
                Ok(())
            } else {
                Err(serde::de::Error::invalid_value(
                    serde::de::Unexpected::Str(value),
                    &self,
                ))
            }
        }
    }

    pub fn serialize<S: serde::Serializer>(_value: &(), serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str("0")
    }
}
