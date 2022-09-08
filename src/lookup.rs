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
    sync_hash_creator: CacheDb<Box<str>, Option<hash>>
}

impl LookupState
{
    pub fn new(db: Database, hasher: NickHasher) -> Self
    {
        let sync_hash_creator = CacheDb::new().with_constructor({
            let hasher = hasher.clone();   
            move |nick| {Some(hasher.create_nick_hash(nick)) }
        });
        Self { db, hasher, sync_hash_creator }
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
            let mut hash = self.sync_hash_creator.get_mut(Blocking, nick)?;
            
            if hash.is_some() {
                // the cachedb created a hash for us, lets store it
                self.db.add_hash(index, &hash).await?;
                // and invalidate it!
                hash = None;
                Ok(false)
            } else {
                // we waited, and got an ivalidated hash, what now? (this means it is inserted in the db unless an error happend there..)
                Ok(true)  // ??
            }
        }
    }
}
