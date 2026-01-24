# Git Security Configuration for PayFlow

## 🔒 Automated Security Measures

This repository includes several security measures to prevent accidental exposure of sensitive data:

### 1. Comprehensive .gitignore Files

**Main Project (.gitignore):**
- Environment files (`.env*`)
- Build artifacts (`target/`, `node_modules/`)
- Log files (`*.log`, `server.log`)
- Database files (`*.db`, `*.sqlite`)
- Security files (`*.key`, `*.pem`, `*.crt`)
- IDE and OS files

**Frontend (frontend/.gitignore):**
- Node.js specific exclusions
- Build outputs (`/dist`, `/build`)
- Cache directories (`.vite`, `.cache`)
- Development files

### 2. Pre-commit Hook

Automatically prevents commits of:
- Files with sensitive extensions (`.env`, `.key`, `.pem`, etc.)
- Log files (`server.log`, `nohup.out`)
- Potential secrets in code (API keys, passwords)

### 3. Security Checklist

Before committing, ensure:
- [ ] No `.env` files are staged
- [ ] No log files are included
- [ ] No database files are staged
- [ ] No private keys or certificates
- [ ] Code doesn't contain hardcoded secrets

### 4. Safe Files to Commit

✅ **Always safe:**
- Source code (`src/`, `frontend/src/`)
- Configuration templates (`.env.example`)
- Documentation (`docs/`, `README.md`)
- Build configurations (`Cargo.toml`, `package.json`)
- Migration files (`migrations/`)

❌ **Never commit:**
- `.env` (contains real API keys)
- `server.log` (runtime logs)
- `target/` (build artifacts)
- `frontend/node_modules/` (dependencies)
- Any files with actual secrets

### 5. Emergency: If Secrets Were Committed

If sensitive data was accidentally committed:

```bash
# Remove from history (DANGER: rewrites history)
git filter-branch --force --index-filter \
  'git rm --cached --ignore-unmatch .env' \
  --prune-empty --tag-name-filter cat -- --all

# Force push (if working alone)
git push origin --force --all

# Rotate all exposed secrets immediately
```

### 6. Environment Setup

```bash
# Copy template
cp .env.example .env

# Edit with your secrets
nano .env

# Verify it's ignored
git status  # Should not show .env
```

## 🚨 Security Reminders

1. **API Keys**: Never commit real API keys
2. **Database URLs**: Keep connection strings private  
3. **Encryption Keys**: Generate new keys for each environment
4. **Webhook Secrets**: Use strong, unique secrets
5. **Log Files**: May contain sensitive request data

## 📞 Security Contact

If you discover a security issue, please contact the development team immediately.

---

**Remember: Security is everyone's responsibility!** 🔐
