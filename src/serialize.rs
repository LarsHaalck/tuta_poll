pub mod serde_base64 {
    use base64::{engine::general_purpose as engines, Engine as _};

    pub fn deserialize<'de, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Vec<u8>, D::Error> {
        deserializer.deserialize_str(Base64Visitor)
    }

    pub struct Base64Visitor;

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

    pub fn serialize<S: serde::Serializer>(
        value: &Vec<u8>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&engines::STANDARD.encode(value))
    }
}

pub mod serde_base64_16 {
    use base64::{engine::general_purpose as engines, Engine as _};

    pub fn deserialize<'de, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<[u8; 16], D::Error> {
        deserializer.deserialize_str(Base64_16Visitor)
    }

    pub struct Base64_16Visitor;

    impl<'de> serde::de::Visitor<'de> for Base64_16Visitor {
        type Value = [u8; 16];

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "base64 string with a decoded length of 16")
        }

        fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
            let decode = engines::STANDARD.decode(value).map_err(|_| {
                serde::de::Error::invalid_value(serde::de::Unexpected::Str(value), &self)
            })?;

            decode.try_into().map_err(|_| {
                serde::de::Error::invalid_value(serde::de::Unexpected::Str(value), &self)
            })
        }
    }

    // pub fn serialize<S: serde::Serializer>(
    //     value: &[u8; 16],
    //     serializer: S,
    // ) -> Result<S::Ok, S::Error> {
    //     serializer.serialize_str(&engines::STANDARD.encode(value))
    // }
}

pub mod serde_option_base64_16 {
    use crate::serialize::serde_base64;
    use base64::{engine::general_purpose as engines, Engine as _};

    pub fn deserialize<'de, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<[u8; 16]>, D::Error> {
        deserializer.deserialize_option(OptionBase64_16Visitor)
    }

    struct OptionBase64_16Visitor;

    impl<'de> serde::de::Visitor<'de> for OptionBase64_16Visitor {
        type Value = Option<[u8; 16]>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(
                formatter,
                "optional base64 string with a decoded length of 16"
            )
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::de::Deserializer<'de>,
        {
            let decode = deserializer.deserialize_str(serde_base64::Base64Visitor)?;
            if decode.is_empty() {
                return Ok(None);
            }
            let value = decode.clone().try_into().map_err(|_| {
                serde::de::Error::custom(format!("Wrong length {} != 16", decode.len()))
            })?;
            Ok(Some(value))
        }

        fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
    }

    pub fn serialize<S: serde::Serializer>(
        value: &Option<[u8; 16]>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match value {
            Some(s) => serializer.serialize_str(&engines::STANDARD.encode(s)),
            None => serializer.serialize_none(),
        }
    }
}

pub mod serde_option_base64 {
    use crate::serialize::serde_base64;
    use base64::{engine::general_purpose as engines, Engine as _};

    pub fn deserialize<'de, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<Vec<u8>>, D::Error> {
        deserializer.deserialize_option(OptionBase64Visitor)
    }

    struct OptionBase64Visitor;

    impl<'de> serde::de::Visitor<'de> for OptionBase64Visitor {
        type Value = Option<Vec<u8>>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "optional base64 string")
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::de::Deserializer<'de>,
        {
            let decode = deserializer.deserialize_str(serde_base64::Base64Visitor)?;
            if decode.is_empty() {
                return Ok(None);
            }

            Ok(Some(decode))
        }

        fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
    }

    pub fn serialize<S: serde::Serializer>(
        value: &Option<Vec<u8>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        match value {
            Some(s) => serializer.serialize_str(&engines::STANDARD.encode(s)),
            None => serializer.serialize_none(),
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

pub mod string_to_enum {
    use num_enum::TryFromPrimitive;

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: TryFromPrimitive<Primitive = u8>,
        D: serde::Deserializer<'de>,
    {
        use serde::Deserialize;
        let s = String::deserialize(deserializer)?;
        let num = s
            .parse::<u8>()
            .map_err(|_| serde::de::Error::custom("Expected u8 wrapped in a string"))?;
        T::try_from_primitive(num)
            .map_err(|_| serde::de::Error::custom("Expected a variant of the enum"))
    }

    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Into<u8> + Clone,
        S: serde::Serializer,
    {
        serializer.serialize_str(&value.clone().into().to_string())
    }
}
