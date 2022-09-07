use super::database::*;
use super::hasher::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum LookupError
{
    #[error("Bcrypt error: {0}")]
    Bcrypt(#[from] bcrypt::BcryptError),
    #[error("Database error: {0}")]
    Db(#[from]tokio_postgres::Error),
}

pub struct LookupState
{
    db: Database,
    hasher: NickHasher,
}

impl LookupState
{
    pub fn new(db: Database, hasher: NickHasher) -> Self
    {
        Self { db, hasher }
    }

    pub async fn lookup(&self, nick: &str) -> Result<bool, LookupError>
    {
        let index = self.hasher.index_for(nick);

        let hashes = self.db.hashes_for_index(index).await?;

        for hash in hashes
        {
            if self.hasher.check_nick_hash(nick, &hash)?
            {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub async fn record(&self, nick: &str) -> Result<bool, LookupError>
    {
        let index = self.hasher.index_for(nick);

        if self.lookup(nick).await?
        {
            Ok(true)
        }
        else
        {
            let hash = self.hasher.create_nick_hash(nick)?;
            self.db.add_hash(index, &hash).await?;
            Ok(false)
        }
    }
}