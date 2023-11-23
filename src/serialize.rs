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

    pub fn serialize<S: serde::Serializer>(
        value: &Vec<u8>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&engines::STANDARD.encode(value))
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

pub mod serde_group_type {
    use crate::user::GroupType;
    pub fn deserialize<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<GroupType, D::Error> {
        deserializer.deserialize_str(GroupTypeVisitor)
    }

    struct GroupTypeVisitor;

    impl<'de> serde::de::Visitor<'de> for GroupTypeVisitor {
        type Value = GroupType;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "an integer between 0 and 11 as a string")
        }

        fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
            match value {
                "0" => Ok(GroupType::User),
                "1" => Ok(GroupType::Admin),
                "2" => Ok(GroupType::MailingList),
                "3" => Ok(GroupType::Customer),
                "4" => Ok(GroupType::External),
                "5" => Ok(GroupType::Mail),
                "6" => Ok(GroupType::Contact),
                "7" => Ok(GroupType::File),
                "8" => Ok(GroupType::LocalAdmin),
                "9" => Ok(GroupType::Calendar),
                "10" => Ok(GroupType::Template),
                "11" => Ok(GroupType::ContactList),
                _ => {
                Err(serde::de::Error::invalid_value(
                    serde::de::Unexpected::Str(value),
                    &self,
                ))
                }
            }
        }
    }
}

pub mod serde_mail_folder_type {
    use crate::mailfolder::MailFolderType;
    pub fn deserialize<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<MailFolderType, D::Error> {
        deserializer.deserialize_str(MailFolderTypeVisitor)
    }

    struct MailFolderTypeVisitor;

    impl<'de> serde::de::Visitor<'de> for MailFolderTypeVisitor {
        type Value = MailFolderType;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "an integer between 0 and 11 as a string")
        }

        fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
            match value {
                "0" => Ok(MailFolderType::Custom),
                "1" => Ok(MailFolderType::Inbox),
                "2" => Ok(MailFolderType::Sent),
                "3" => Ok(MailFolderType::Trash),
                "4" => Ok(MailFolderType::Archive),
                "5" => Ok(MailFolderType::Spam),
                "6" => Ok(MailFolderType::Draft),
                _ => {
                Err(serde::de::Error::invalid_value(
                    serde::de::Unexpected::Str(value),
                    &self,
                ))
                }
            }
        }
    }
}