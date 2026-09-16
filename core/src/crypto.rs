//! Custom encryption/decryption logic matching the Python implementation.

/// Encrypts input plain text string into underscore-separated 3-digit token string.
pub fn hash_password(password: &str) -> Result<String, String> {
    let mut state: u32 = 17;
    let mut out: Vec<String> = Vec::new();

    for (i, &b) in password.as_bytes().iter().enumerate() {
        let b_u32 = b as u32;
        let x = (b_u32 + state + (i as u32) * 7) % 256;
        out.push(format!("{:03}", x));
        state = (state * 31 + b_u32 + (i as u32)) % 256;
    }

    Ok(out.join("_"))
}

/// Decrypts the token string back into plain text and compares it against the input password.
pub fn verify_password(entered: &str, stored_encrypted: &str) -> bool {
    if stored_encrypted.is_empty() {
        return false;
    }

    let mut state: u32 = 17;
    let mut bytes: Vec<u8> = Vec::new();

    for (i, token) in stored_encrypted.split('_').enumerate() {
        let x = match token.parse::<i32>() {
            Ok(val) => val,
            Err(_) => return false,
        };

        let i_u32 = i as u32;
        
        let mut diff = x - (state as i32) - ((i_u32 * 7) as i32);
        diff = diff % 256;
        if diff < 0 {
            diff += 256;
        }

        let b = diff as u8;
        bytes.push(b);
        state = (state * 31 + (b as u32) + i_u32) % 256;
    }

    match String::from_utf8(bytes) {
        Ok(decrypted) => decrypted == entered,
        Err(_) => false,
    }
}