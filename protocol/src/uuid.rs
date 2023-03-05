use std::fmt::Debug;

use md5::Digest;

#[derive(Copy, Clone, PartialEq, Hash, Eq, Debug)]
pub struct UUID3 {
    raw: u128,
}

impl From<u128> for UUID3 {
    fn from(raw: u128) -> Self {
        Self { raw }
    }
}

impl From<UUID3> for u128 {
    fn from(uuid: UUID3) -> Self {
        uuid.raw
    }
}

impl Into<[u8; 16]> for UUID3 {
    fn into(self) -> [u8; 16] {
        self.raw.to_be_bytes()
    }
}

impl TryFrom<String> for UUID3 {
    type Error = anyhow::Error;

    /// I have no clue if this is correct...
    fn try_from(data: String) -> anyhow::Result<Self> {
        if data.len() != 36 {
            return Err(anyhow::anyhow!("Invalid UUID length: {}", data.len()));
        }

        let data = data.replace("-", "");
        let raw = u128::from_str_radix(&data, 16)?; 

        Ok(Self { raw })
    }
}

impl ToString for UUID3 {
    fn to_string(&self) -> String {
        let mut data = [0u8; 16];

        for i in 0..16 {
            data[i] = ((self.raw >> (i * 8)) & 0xFF) as u8;
        }

        let dig = Digest(data);
        let string = format!("{:x}", dig);

        format!("{}-{}-{}-{}-{}", &string[0..8], &string[8..12], &string[12..16], &string[16..20], &string[20..32])
    }
}

impl UUID3 {
    pub fn new(data: String) -> Self {
        let hash = md5::compute(data.as_bytes());

        let mut raw = 0u128;

        for i in 0..16 {
            raw |= (hash[i] as u128) << (i * 8);
        }

        Self { raw }
    }
}