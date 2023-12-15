use super::config;
use super::*;
use crate::api::{
    bucket_permission, group, mail, mailbody, mailbox, mailboxgrouproot, mailfolder, permission,
    salt, session, user,
};
use crate::crypto;
use anyhow::{bail, Error, Result};
use lz4_flex::decompress_into;
use tracing::debug;
use types::{
    Aes128Key, Base64, BucketPermission, BucketPermissionType, GroupType, Id, Mail, MailFolderType,
    Permission, PermissionType, ReadStatus, User,
};
use websocket::WebSocketConnector;

use async_stream::{stream, try_stream};
use futures_core::stream::Stream;

pub struct Client {
    config: config::Account,
    access_token: String,
    inboxes: Vec<String>,
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
    pub async fn new(config: &config::Account) -> Result<Client> {
        let SessionData {
            user_passphrase_key,
            access_token,
            user_id,
        } = Self::create_session(config).await?;
        let mut user = user::fetch(&access_token, &user_id).await?;
        user.unlock_group_keys(&user_passphrase_key);
        // let user_group_info = group_info::fetch(&access_token, &user.user_group.group_info)?;
        let mail_member = user
            .memberships
            .iter()
            .find(|membership| membership.group_type == GroupType::Mail)
            .ok_or(Error::msg("Could not find group with type mail"))?;

        // i have never seen a GroupType::Mail with empty sym_enc_g_key
        // let mail_group_key =
        //     crypto::decrypt_key(&user_group_key, &mail_member.sym_enc_g_key.unwrap());
        let root = mailboxgrouproot::fetch(&access_token, &mail_member.group).await?;

        let mailbox = mailbox::fetch(&access_token, &root).await?;
        let folders = mailfolder::fetch(&access_token, &mailbox).await?;

        let inboxes: Vec<_> = folders
            .into_iter()
            .filter(|folder| folder.folder_type == MailFolderType::Inbox)
            .map(|folder| folder.mails)
            .collect();

        Ok(Client {
            config: config.clone(),
            access_token,
            inboxes,
            user,
        })
    }

    async fn create_session(config: &config::Account) -> Result<SessionData> {
        let salt = salt::fetch(&config.email_address).await?;
        let user_passphrase_key = crypto::create_user_passphrase_key(&config.password, &salt);
        let session = session::fetch(&config.email_address, &user_passphrase_key).await?;
        let access_token = session.access_token;
        Ok(SessionData {
            user_passphrase_key,
            access_token,
            user_id: session.user,
        })
    }

    pub fn get_mails(&self) -> impl Stream<Item = Result<Mail>> + '_ {
        try_stream! {
            for inbox in &self.inboxes {
                let mut start = None;
                let curr_mails = mail::fetch_from_inbox(&self.access_token, &inbox, start).await?;
                let mut n = curr_mails.len();
                let mut last = curr_mails.last().map_or("".into(), |m| m.id.1.clone());

                for mail in curr_mails {
                    yield mail
                }

                while n > 0 {
                    start = Some(last);
                    let curr_mails = mail::fetch_from_inbox(&self.access_token, &inbox, start).await?;
                    last = curr_mails.last().map_or("".into(), |m| m.id.1.clone());
                    n = curr_mails.len();
                    for mail in curr_mails {
                        yield mail
                    }
                }
            }
        }
    }

    fn resolve_session_key_owner(&self, mail: &Mail) -> Result<Aes128Key> {
        debug!("resolve session key with owner key");
        let gk = self
            .user
            .get_group_key(&mail.owner_group)
            .ok_or(Error::msg("No group key for mail"))?;

        let key = mail
            .owner_enc_session_key
            .ok_or(Error::msg("No owner enc session key for mail"))?;
        return Ok(crypto::decrypt_key(&gk, &key));
    }

    fn try_symmetric_permission(&self, perms: &Vec<Permission>) -> Option<Aes128Key> {
        debug!("try symmetric permission");
        let sym_perm = perms.iter().find(|p| {
            p.permission_type == PermissionType::PublicSymmetric
                || p.permission_type == PermissionType::Symmetric
                    && p.owner_group
                        .as_ref()
                        .is_some_and(|g| self.user.has_group(&g))
                    && p.owner_enc_session_key.is_some()
        });

        if let Some(sym) = sym_perm {
            let gk = self
                .user
                .get_group_key(&sym.owner_group.as_ref().unwrap())
                .unwrap();
            let sk = sym.owner_enc_session_key.unwrap();
            Some(crypto::decrypt_key(&gk, &sk))
        } else {
            None
        }
    }

    async fn resolve_session_key_public_external(
        &self,
        perms: &Vec<Permission>,
    ) -> Result<Aes128Key> {
        debug!("resolve session key from public or external bucket");
        let pub_or_external_perm = perms
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
            .bucket_permissions;
        let bucket_permissions =
            bucket_permission::fetch(&self.access_token, &bucket_perm_id).await?;
        let bucket_permission = bucket_permissions
            .iter()
            .find(|p| {
                p.permission_type == BucketPermissionType::Public
                    || p.permission_type == BucketPermissionType::External
            })
            .ok_or(Error::msg("could not find public or external permission"))?;

        match bucket_permission.permission_type {
            BucketPermissionType::External => {
                self.resolve_external_bucket(&bucket_permission, &pub_or_external_perm)
            }
            BucketPermissionType::Public => {
                self.resolve_public_bucket(&bucket_permission, &pub_or_external_perm)
                    .await
            }
        }
    }

    fn resolve_external_bucket(
        &self,
        bucket_perm: &BucketPermission,
        perm: &Permission,
    ) -> Result<Aes128Key> {
        debug!("decrypt with external bucket");
        let bucket_key;
        if let Some(bk) = bucket_perm.owner_enc_bucket_key {
            bucket_key = crypto::decrypt_key(
                &self
                    .user
                    .get_group_key(&bucket_perm.owner_group.as_ref().unwrap())
                    .unwrap(),
                &bk,
            );
        } else if let Some(sym) = bucket_perm.sym_enc_bucket_key {
            bucket_key = crypto::decrypt_key(&self.user.get_user_group_key(), &sym);
        } else {
            bail!("BucketEncSessionKey is not defined for Permission")
        }

        let msg = perm
            .bucket_enc_session_key
            .ok_or(Error::msg("bucket enc session key not defined"))?;
        Ok(crypto::decrypt_key(&bucket_key, &msg))
    }

    async fn resolve_public_bucket(
        &self,
        bucket_perm: &BucketPermission,
        perm: &Permission,
    ) -> Result<Aes128Key> {
        debug!("decrypt with public bucket");
        let pub_enc_bucket_key = bucket_perm
            .pub_enc_bucket_key
            .clone()
            .ok_or(Error::msg("PubEncBucketKey is not defined"))?;

        let bucket_enc_session_key = perm
            .bucket_enc_session_key
            .ok_or(Error::msg("BucktEncSessionKey is not defined"))?;

        let bucket_key = self
            .decrypt_bucket_key_key_pair_group(&bucket_perm.group, &pub_enc_bucket_key)
            .await?;
        let sk = crypto::decrypt_key(&bucket_key, &bucket_enc_session_key);

        // if let Some(og) = &bucket_perm.owner_group {
        //     // update sym perm
        //     let bucket_perm_ogk = self.user.get_group_key(&og).unwrap();
        //     let bucket_perm_gk = self.user.get_group_key(&bucket_perm.group).unwrap();
        //     self.update_sym_perm_key(mail, perm, &bucket_perm_ogk, &bucket_perm_gk)?;
        // }

        Ok(sk)
    }

    // broken and should never be used
    // fn update_sym_perm_key(
    //     &self,
    //     mail: &Mail,
    //     perm: &Permission,
    //     bucket_perm_ogk: &Aes128Key,
    //     bucket_perm_gk: &Aes128Key,
    // ) -> Result<()> {
    //     if !self.user.is_leader() {
    //         return Ok(());
    //     }

    //     debug!("update with symmetric permission key");
    //     if mail.owner_enc_session_key.is_none() && perm.owner_group == Some(mail.owner_group.clone()) {
    //         mail.owner_enc_session_key = Some(crypto::encrypt_key(bucket_perm_ogk, bucket_perm_gk));
    //         mail::update(&self.access_token, &mail, true)?;

    //     } else {
    //         warn!("shared permission not implemented");
    //     }

    //     Ok(())
    // }

    async fn decrypt_bucket_key_key_pair_group(
        &self,
        key_pair: &Id,
        pub_enc_bucket_key: &Base64,
    ) -> Result<Aes128Key> {
        debug!("decrypt bucket key with key pair of group");
        let group = group::fetch(&self.access_token, &key_pair).await?;
        let key_pair = &group.keys[0];
        let priv_key = crypto::decrypt_rsa_key(
            &self.user.get_group_key(&group.id).unwrap(),
            &key_pair.sym_enc_priv_key,
        )?;

        crypto::rsa_decrypt(&priv_key, pub_enc_bucket_key)?
            .try_into()
            .map_err(|_| Error::msg("Could not convert to [u8; 16]"))
    }

    async fn resolve_session_key(&self, mail: &Mail) -> Result<Aes128Key> {
        debug!("Resolve session key");
        if mail.owner_enc_session_key.is_some() && self.user.has_group(&mail.owner_group) {
            self.resolve_session_key_owner(mail)
        } else {
            let perms = permission::fetch(&self.access_token, &mail.permissions).await?;
            Ok(self
                .try_symmetric_permission(&perms)
                .unwrap_or(self.resolve_session_key_public_external(&perms).await?))
        }
    }

    pub async fn decrypt(&self, mail: &Mail) -> Result<MailContent> {
        let session_key = self.resolve_session_key(mail).await?;

        let subject = if self.config.show_subject {
            let tmp = crypto::aes_decrypt(&session_key, &mail.subject)?;
            Some(
                std::str::from_utf8(&tmp)
                    .expect("Subject could not converted to UTF-8")
                    .to_string(),
            )
        } else {
            None
        };

        let name = if self.config.show_name {
            let tmp = crypto::aes_decrypt(&session_key, &mail.sender.name)?;
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
            let mailbody = mailbody::fetch(&self.access_token, &mail.body).await?;
            let compressed_text = crypto::aes_decrypt(&session_key, &mailbody)?;
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

    pub async fn mark_read(&self, mail: &mut Mail) -> Result<()> {
        if mail.read_status == ReadStatus::Read {
            return Ok(());
        }

        mail.read_status = ReadStatus::Read;
        mail::update(&self.access_token, &mail, false).await?;
        Ok(())
    }

    pub fn get_websocket_connector(&self) -> Result<WebSocketConnector> {
        WebSocketConnector::from_url(&self.access_token, &self.user.id, &self.inboxes)
    }
}
