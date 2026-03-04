use std::io::{self, Write};
use sqlx::postgres::PgPoolOptions;
use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
use rand::rngs::OsRng;
use chrono::Utc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env");

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    println!("Welcome to the FidduPay Admin Seeder");
    println!("------------------------------------");

    print!("Enter Admin Username (default: admin): ");
    io::stdout().flush()?;
    let mut username = String::new();
    io::stdin().read_line(&mut username)?;
    let username = username.trim();
    let username = if username.is_empty() { "admin" } else { username };
    
    // We'll use this as the email prefix/identifier
    let email = format!("{}@fiddupay.com", username);

    print!("Enter Admin Password (will be hidden/masked if possible, else visible): ");
    io::stdout().flush()?;
    
    // Simple visible input since we don't have rpassword dependency, 
    // but this is a maintenance script run by admin, so acceptable for now.
    let mut password = String::new();
    io::stdin().read_line(&mut password)?;
    let password = password.trim();

    if password.len() < 8 {
        eprintln!("Error: Password must be at least 8 characters");
        return Ok(());
    }

    println!("\nHashing password...");
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)
        .map_err(|e| format!("Hashing failed: {}", e))?
        .to_string();

    println!("Creating admin user '{}' ({})", username, email);

    // Insert into INDEPENDENT admin_users table
    // This provides complete isolation from merchant data
    let result = sqlx::query(
        r#"
        INSERT INTO admin_users (
            username, email, password_hash, role
        )
        VALUES ($1, $2, $3, 'SUPER_ADMIN')
        ON CONFLICT (username) DO UPDATE 
        SET password_hash = $3, email = $2
        RETURNING id
        "#
    )
    .bind(username)
    .bind(&email)
    .bind(&password_hash)
    .fetch_one(&pool)
    .await?;

    use sqlx::Row;
    let admin_id: i32 = result.get("id");
    println!("Success! Super Admin created with ID: {}", admin_id);
    println!("Login Username: {}", username);
    println!("Login Email: {}", email);
    println!("Login Password: [HIDDEN]");

    Ok(())
}
