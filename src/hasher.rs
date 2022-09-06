use std::hash::Hasher;

use siphasher::sip::SipHasher;

const INDEX_BITMASK: u64 = 0xffff;
const BCRYPT_COST: u32 = bcrypt::DEFAULT_COST;

pub struct NickHasher
{
    sip_hasher: SipHasher,
}

impl NickHasher
{
    pub fn new(key: &[u8; 16]) -> Self
    {
        Self {
            sip_hasher: SipHasher::new_with_key(key)
        }
    }

    pub fn index_for(&self, nick: &str) -> i32
    {
        let mut hasher = self.sip_hasher.clone();

        // This is unstable, so do it by hand
        //hasher.write_str(nick);
        hasher.write(nick.as_bytes());

        // try_unwrap can't fail here because INDEX_BITMASK guarantees it'll fit into i32
        (hasher.finish() & INDEX_BITMASK).try_into().unwrap()
    }

    pub fn check_nick_hash(&self, nick: &str, hash: &str) -> Result<bool, bcrypt::BcryptError>
    {
        bcrypt::verify(nick, hash)
    }

    pub fn create_nick_hash(&self, nick: &str) -> Result<String, bcrypt::BcryptError>
    {
        bcrypt::hash(nick, BCRYPT_COST)
    }
}