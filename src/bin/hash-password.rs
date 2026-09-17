use argon2::{password_hash::PasswordHasher, Argon2};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let password = rpassword::prompt_password("Admin password: ")?;
    let confirmation = rpassword::prompt_password("Confirm password: ")?;

    if password.is_empty() {
        return Err("Password cannot be empty.".into());
    }
    if password != confirmation {
        return Err("Passwords do not match.".into());
    }

    let hash = Argon2::default()
        .hash_password(password.as_bytes())?
        .to_string();

    println!("{hash}");
    Ok(())
}
