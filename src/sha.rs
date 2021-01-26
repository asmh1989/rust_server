use digest::Digest;
use sha2::Sha256;

fn hash_password<D: Digest>(password: &[u8], salt: &str, output: &mut [u8]) {
    let mut hasher = D::new();

    hasher.reset();

    if !salt.is_empty() {
        hasher.update(salt.as_bytes());
    }
    hasher.update(password);

    output.copy_from_slice(hasher.finalize().as_slice());
}

fn to_hex_string(bytes: Vec<u8>) -> String {
    let strs: Vec<String> = bytes.iter().map(|b| format!("{:02x}", b)).collect();
    strs.join("")
}

pub fn sha256_encode(password: &str, salt: &str) -> String {
    let mut buf: [u8; 32] = [0; 32];
    hash_password::<Sha256>(password.as_bytes(), salt, &mut buf);

    for _ in 0..15 {
        let mut buf2: [u8; 32] = [0; 32];
        buf2.copy_from_slice(&buf);
        hash_password::<Sha256>(&buf2, "", &mut buf);
    }

    to_hex_string((&buf).to_vec())
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_sha256() {
        assert_eq!(
            "ce059eb08c180b4ea4233d4c65af00ec57f6b38d90400f2f6d1f6d9f11db2b67",
            super::sha256_encode("666666", "jmfjNfLUHYWx2Kjj1JFx")
        );
    }
}
