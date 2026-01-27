use std::process::Command;

fn main() {
    println!("🔍 Debugging FidduPay Authentication Issue");
    
    // Test 1: Check if backend is responding to health
    println!("\n1️⃣ Testing health endpoint...");
    let health_output = Command::new("curl")
        .args(&["-s", "http://127.0.0.1:8080/health"])
        .output()
        .expect("Failed to execute curl");
    
    if health_output.status.success() {
        println!("   ✅ Health: {}", String::from_utf8_lossy(&health_output.stdout));
    } else {
        println!("   ❌ Health check failed");
        return;
    }
    
    // Test 2: Register a new merchant
    println!("\n2️⃣ Registering test merchant...");
    let reg_output = Command::new("curl")
        .args(&[
            "-s", "-X", "POST",
            "-H", "Content-Type: application/json",
            "-d", r#"{"email":"debug2@test.com","business_name":"Debug Test 2","password":"Test123!"}"#,
            "http://127.0.0.1:8080/api/v1/merchants/register"
        ])
        .output()
        .expect("Failed to execute curl");
    
    if reg_output.status.success() {
        let response = String::from_utf8_lossy(&reg_output.stdout);
        println!("   ✅ Registration: {}", response);
        
        // Extract API key from response
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response) {
            if let Some(api_key) = json.get("api_key").and_then(|k| k.as_str()) {
                println!("   🔑 API Key: {}", api_key);
                
                // Test 3: Try authentication
                println!("\n3️⃣ Testing authentication...");
                let auth_header = format!("Authorization: Bearer {}", api_key);
                let auth_output = Command::new("curl")
                    .args(&[
                        "-v", "-s",
                        "-H", &auth_header,
                        "http://127.0.0.1:8080/api/v1/merchants/profile"
                    ])
                    .output()
                    .expect("Failed to execute curl");
                
                println!("   📤 Auth Status: {}", auth_output.status);
                println!("   📥 Auth Response: {}", String::from_utf8_lossy(&auth_output.stdout));
                println!("   🚨 Auth Error: {}", String::from_utf8_lossy(&auth_output.stderr));
            }
        }
    } else {
        println!("   ❌ Registration failed");
    }
}
