use crate::crypto;
use crate::serialize::*;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type Id = String;
pub type IdTuple = (Id, Id);
pub type Base64 = Vec<u8>;
pub type Aes128Key = [u8; 16];

#[derive(Debug, Deserialize)]
pub struct Session {
    #[serde(with = "serde_format")]
    _format: (),
    #[serde(rename = "accessToken")]
    pub access_token: String,
    pub user: Id,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct User {
    #[serde(with = "serde_format")]
    _format: (),
    #[serde(rename = "_id")]
    pub id: Id,
    pub memberships: Vec<Membership>,
    #[serde(rename = "userGroup")]
    pub user_group: UserGroup,

    #[serde(skip)]
    group_keys: HashMap<Id, Aes128Key>,
}

impl User {
    pub fn has_group(&self, group_id: &Id) -> bool {
        return self.user_group.group == *group_id
            || self.memberships.iter().any(|m| m.group == *group_id);
    }

    pub fn unlock_group_keys(&mut self, user_passphrase_key: &Aes128Key) {
        let user_group_key =
            crypto::decrypt_key(&user_passphrase_key, &self.user_group.sym_enc_g_key);

        self.group_keys
            .insert(self.user_group.group.clone(), user_group_key.clone());

        for member in &self.memberships {
            if let Some(sym) = member.sym_enc_g_key {
                self.group_keys.insert(
                    member.group.clone(),
                    crypto::decrypt_key(&user_group_key, &sym),
                );
            }
        }
    }

    pub fn get_group_key(&self, group_id: &Id) -> Option<Aes128Key> {
        self.group_keys.get(group_id).copied()
    }

    pub fn get_user_group_key(&self) -> Aes128Key {
        self.group_keys
            .get(&self.user_group.group)
            .copied()
            .unwrap()
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Membership {
    pub group: Id,
    #[serde(with = "string_to_enum")]
    pub group_type: GroupType,
    #[serde(with = "serde_option_base64_16")]
    pub sym_enc_g_key: Option<Aes128Key>,
}

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct UserGroup {
    pub group: Id,
    pub group_info: IdTuple,
    #[serde(with = "serde_base64_16")]
    pub sym_enc_g_key: Aes128Key,
}

pub struct Credentials {
    pub login: String,
    pub access_token: String,
    pub user_id: Id,
}

#[derive(Debug, PartialEq, TryFromPrimitive, IntoPrimitive, Clone)]
#[repr(u8)]
pub enum GroupType {
    User,
    Admin,
    MailingList,
    Customer,
    External,
    Mail,
    Contact,
    File,
    LocalAdmin,
    Calendar,
    Template,
    ContactList,
}

#[derive(Debug, Deserialize)]
pub struct GroupInfo {
    #[serde(with = "serde_base64_16", rename = "_ownerEncSessionKey")]
    pub owner_enc_session_key: Aes128Key,
    #[serde(rename = "_ownerGroup")]
    pub owner_group: Id,
    #[serde(rename = "_permissions")]
    pub permissions: Id,
}

#[derive(Debug, Deserialize, PartialEq, TryFromPrimitive, IntoPrimitive, Clone)]
#[repr(u8)]
pub enum MailFolderType {
    Custom,
    Inbox,
    Sent,
    Trash,
    Archive,
    Spam,
    Draft,
}

#[derive(Debug, Deserialize)]
pub struct Folder {
    #[serde(with = "serde_format")]
    _format: (),
    #[serde(with = "string_to_enum", rename = "folderType")]
    pub folder_type: MailFolderType,
    #[serde(rename = "_id")]
    pub id: (String, String),
    pub mails: String,
    #[serde(with = "serde_base64")]
    pub name: Base64,
    #[serde(with = "serde_base64_16", rename = "_ownerEncSessionKey")]
    pub owner_enc_session_key: Aes128Key,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Mail {
    #[serde(with = "serde_format", rename = "_format")]
    format: (),
    pub auth_status: String,
    pub attachments: Vec<(String, String)>,
    // was not needed yet
    pub bucket_key: (),
    pub body: String,
    pub bcc_recipients: Vec<Sender>,
    pub cc_recipients: Vec<Sender>,
    #[serde(with = "serde_base64")]
    pub confidential: Base64,
    pub conversation_entry: (String, String),
    pub different_envelope_sender: Option<Sender>,
    pub first_recipient: Sender,
    pub headers: Option<String>,
    #[serde(rename = "_id")]
    pub id: (String, String),
    #[serde(with = "serde_base64")]
    pub list_unsubscribe: Base64,
    pub mail_details: (),
    pub mail_details_draft: (),
    #[serde(with = "serde_base64")]
    pub method: Base64,
    pub moved_time: String,
    #[serde(with = "serde_option_base64_16", rename = "_ownerEncSessionKey")]
    pub owner_enc_session_key: Option<Aes128Key>,
    #[serde(rename = "_ownerGroup")]
    pub owner_group: String,
    #[serde(rename = "_permissions")]
    pub permissions: Id,
    pub phishing_status: String,
    pub received_date: String,
    pub recipient_count: String,
    pub reply_tos: Vec<Sender>,
    pub reply_type: String,
    pub sent_date: String,
    pub sender: Sender,
    pub state: String,
    #[serde(with = "serde_base64")]
    pub subject: Base64,
    pub to_recipients: Vec<Sender>,
    pub unread: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Sender {
    pub address: String,
    pub contact: (),
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(with = "serde_base64")]
    pub name: Base64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Permission {
    #[serde(with = "serde_format", rename = "_format")]
    _format: (),
    #[serde(with = "string_to_enum", rename = "type")]
    pub permission_type: PermissionType,
    #[serde(with = "serde_option_base64_16")]
    pub bucket_enc_session_key: Option<Aes128Key>,
    #[serde(rename = "_ownerEncSessionKey")]
    pub owner_enc_session_key: Option<Aes128Key>,
    #[serde(rename = "_ownerGroup")]
    pub owner_group: Option<Id>,
    pub bucket: Option<Bucket>,
}

#[derive(Debug, PartialEq, TryFromPrimitive, IntoPrimitive, Clone)]
#[repr(u8)]
pub enum PermissionType {
    Public,
    Symmetric,
    PublicSymmetric,
    Unencrypted,
    External,
    OwnerList,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Bucket {
    pub bucket_permissions: Id,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BucketPermission {
    #[serde(with = "serde_format", rename = "_format")]
    _format: (),
    #[serde(with = "string_to_enum", rename = "type")]
    pub permission_type: BucketPermissionType,
    #[serde(rename = "_ownerGroup")]
    pub owner_group: Option<Id>,
    #[serde(with = "serde_option_base64_16")]
    pub owner_enc_bucket_key: Option<Aes128Key>,
    #[serde(with = "serde_option_base64")]
    pub pub_enc_bucket_key: Option<Base64>,
    #[serde(with = "serde_option_base64_16")]
    pub sym_enc_bucket_key: Option<Aes128Key>,
    pub group: Id,
}

#[derive(Debug, PartialEq, TryFromPrimitive, IntoPrimitive, Clone)]
#[repr(u8)]
pub enum BucketPermissionType {
    Public = 2,
    External = 3,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    #[serde(rename = "_id")]
    pub id: String,
    pub keys: Vec<KeyPair>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyPair {
    #[serde(with = "serde_base64")]
    pub sym_enc_priv_key: Base64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityUpdate {
    pub event_batch: Vec<Event>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub instance_id: String,
    pub instance_list_id: String,
    #[serde(with = "string_to_enum")]
    pub operation: OperationType,
    #[serde(rename = "type")]
    pub event_type: String, // yes this is really a string
}

#[derive(Debug, PartialEq, TryFromPrimitive, IntoPrimitive, Clone)]
#[repr(u8)]
pub enum OperationType {
    Create,
    Update,
    Delete,
}
