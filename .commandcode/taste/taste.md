# Taste (Continuously Learned by [CommandCode][cmd])

[cmd]: https://commandcode.ai/

# rust
- Use WSL to compile Rust projects instead of native Windows/MSVC. Confidence: 0.70
- After final implementation, run quality checklist: cargo fmt, cargo fmt --check, cargo clippy -- -D warnings, cargo check --bin fiddupay, cargo test, and cargo audit. Cargo audit is critical and should not be skipped even if it requires retries. Confidence: 0.70

