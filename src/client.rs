use super::config;
use super::*;
use crate::api::{
    bucket_permission, group_info, mail, mailbody, mailbox, mailboxgrouproot, mailfolder,
    permission, salt, session, user,
};
use crate::crypto;
use anyhow::{Error, Result};
use lz4_flex::decompress_into;
use types::{
    Aes128Key, BucketPermissionType, Folder, GroupType, Id, Mail, MailFolderType, PermissionType,
    User,
};

pub struct Client {
    config: config::Account,
    access_token: String,
    mail_group_key: Aes128Key,
    inboxes: Vec<Folder>,
    user: User,
}

#[derive(Debug)]
pub struct MailContent {
    pub subject: Option<String>,
    pub name: Option<String>,
    pub address: String,
    pub body: Option<String>,
}

struct SessionData {
    user_passphrase_key: Aes128Key,
    access_token: String,
    user_id: Id,
}

impl Client {
    pub fn new(config: &config::Account) -> Result<Client> {
        let SessionData {
            user_passphrase_key,
            access_token,
            user_id,
        } = Self::create_session(config)?;
        let user = user::fetch(&access_token, &user_id)?;
        let user_group_info = group_info::fetch(&access_token, &user.user_group.group_info)?;
        let user_group_key =
            crypto::decrypt_key(&user_passphrase_key, &user.user_group.sym_enc_g_key);
        let mail_member = user
            .memberships
            .iter()
            .find(|membership| membership.group_type == GroupType::Mail)
            .ok_or(Error::msg("Could not find group with type mail"))?;

        // i have never seen a GroupType::Mail with empty sym_enc_g_key
        let mail_group_key =
            crypto::decrypt_key(&user_group_key, &mail_member.sym_enc_g_key.unwrap());
        let root = mailboxgrouproot::fetch(&access_token, &mail_member.group)?;

        let mailbox = mailbox::fetch(&access_token, &root)?;
        let folders = mailfolder::fetch(&access_token, &mailbox)?;

        let inboxes: Vec<_> = folders
            .into_iter()
            .filter(|folder| folder.folder_type == MailFolderType::Inbox)
            .collect();

        Ok(Client {
            config: config.clone(),
            access_token,
            mail_group_key,
            inboxes,
            user,
        })
    }

    fn create_session(config: &config::Account) -> Result<SessionData> {
        let salt = salt::fetch(&config.email_address)?;
        let user_passphrase_key = crypto::create_user_passphrase_key(&config.password, &salt);
        let session = session::fetch(&config.email_address, &user_passphrase_key)?;
        let access_token = session.access_token;
        Ok(SessionData {
            user_passphrase_key,
            access_token,
            user_id: session.user,
        })
    }

    pub fn get_mails(&self) -> Result<Vec<Mail>> {
        let mut mails = Vec::new();
        for inbox in &self.inboxes {
            mails.extend(mail::fetch_from_inbox(&self.access_token, &inbox.mails)?);
        }
        Ok(mails)
    }

    fn resolve_session_key_owner(&self, mail: &Mail) -> Aes128Key {
        return crypto::decrypt_key(
            &self.mail_group_key,
            &mail.owner_enc_session_key.as_ref().unwrap(),
        );
    }

    fn resolve_session_key_public_external(&self, mail: &Mail) -> Result<Aes128Key> {
        let permissions = permission::fetch(&self.access_token, &mail.permissions)?;
        let pub_or_external_perm = permissions
            .iter()
            .find(|p| {
                p.permission_type == PermissionType::Public
                    || p.permission_type == PermissionType::External
            })
            .ok_or(Error::msg("could not find public or external permission"))?;

        let bucket_perm_id = &pub_or_external_perm
            .bucket
            .clone()
            .ok_or(Error::msg("Bucket is null"))?
            .bucket_permission;
        let bucket_permissions = bucket_permission::fetch(&self.access_token, &bucket_perm_id)?;
        let bucket_permission = bucket_permissions
            .iter()
            .find(|p| {
                p.permission_type == BucketPermissionType::Public
                    || p.permission_type == BucketPermissionType::External
            })
            .ok_or(Error::msg("could not find public or external permission"))?;

        match bucket_permission.permission_type {
            BucketPermissionType::Public => {
                self.resolve_public_bucket(mail, &bucket_permission, &pub_or_external_perm)
            }
            BucketPermissionType::External => {
                self.resolve_external_bucket(mail, &bucket_permission, &pub_or_external_perm)
            }
        }
    }

    fn resolve_public_bucket(&self, mail: &Mail) -> Result<Aes128Key> {
        return Ok(crypto::decrypt_key(
            &self.mail_group_key,
            &mail.owner_enc_session_key.as_ref().unwrap(),
        ));
    }

    fn resolve_external_bucket(&self, mail: &Mail) -> Result<Aes128Key> {
        return Ok(crypto::decrypt_key(
            &self.mail_group_key,
            &mail.owner_enc_session_key.as_ref().unwrap(),
        ));
    }

    fn resolve_session_key(&self, mail: &Mail) -> Result<Aes128Key> {
        if mail.owner_enc_session_key.is_some() && self.user.has_group(&mail.owner_group) {
            Ok(self.resolve_session_key_owner(mail))
        } else {
            // resolve public or external
            self.resolve_session_key_public_external(mail)
        }
    }

    pub fn decrypt(&self, mail: &Mail) -> Result<MailContent> {
        let session_key = self.resolve_session_key(mail)?;
        let session_sub_keys = crypto::SubKeys::new(session_key);

        let subject = if self.config.show_subject {
            let tmp = crypto::decrypt_with_mac(&session_sub_keys, &mail.subject)?;
            Some(
                std::str::from_utf8(&tmp)
                    .expect("Subject could not converted to UTF-8")
                    .to_string(),
            )
        } else {
            None
        };

        let name = if self.config.show_name {
            let tmp = crypto::decrypt_with_mac(&session_sub_keys, &mail.sender.name)?;
            Some(
                std::str::from_utf8(&tmp)
                    .expect("Name could not converted to UTF-8")
                    .to_string(),
            )
        } else {
            None
        };

        let address = mail.sender.address.to_string();

        let body = if self.config.show_body {
            let mailbody = mailbody::fetch(&self.access_token, &mail.body)?;
            let compressed_text = crypto::decrypt_with_mac(&session_sub_keys, &mailbody)?;
            let mut buf: Vec<u8> = vec![0; mailbody.len() * 6];
            let size = decompress_into(&compressed_text, &mut buf)?;
            buf.resize(size, 0);
            Some(
                std::str::from_utf8(&buf)
                    .expect("Body could not be converted to UTF-8")
                    .to_string(),
            )
        } else {
            None
        };

        Ok(MailContent {
            subject,
            name,
            address,
            body,
        })
    }

    pub fn mark_read(&self, mail: &mut Mail) -> Result<()> {
        if mail.unread == "0" {
            return Ok(());
        }

        mail.unread = "0".to_string();
        mail::update(&self.access_token, &mail)?;
        Ok(())
    }
}
