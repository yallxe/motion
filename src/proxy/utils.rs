use sha1::{Sha1, Digest};

pub fn generate_offline_uuid(name: &String) -> String {
    let mut hasher = Sha1::new();
    hasher.update("OfflinePlayer:".as_bytes());
    hasher.update(name.as_bytes());
    let hash = hasher.finalize();
    
    let mut uuid = String::new();
    uuid.push_str(&format!("{:x}", hash[0..4].iter().fold(0, |acc, x| (acc << 8) | *x as u32)));
    uuid.push('-');
    uuid.push_str(&format!("{:x}", hash[4..6].iter().fold(0, |acc, x| (acc << 8) | *x as u32)));
    uuid.push('-');
    uuid.push_str(&format!("{:x}", hash[6..8].iter().fold(0, |acc, x| (acc << 8) | *x as u32)));
    uuid.push('-');
    uuid.push_str(&format!("{:x}", hash[8..10].iter().fold(0, |acc, x| (acc << 8) | *x as u32)));
    uuid.push('-');
    uuid.push_str(&format!("{:x}", hash[10..16].iter().fold(0, |acc, x| (acc << 8) | *x as u32)));

    uuid
}