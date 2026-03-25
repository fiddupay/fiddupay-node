# Security Hardening Guide

## Critical: Credentials Management

### ✅ DO - Correct Practices

1. **Use Environment Variables Only**
   - Never commit `.env`, `.env.production`, or any file with credentials
   - Use `.env.example` with placeholder values for documentation
   - Set environment variables via deployment platform (Vercel, Railway, Docker, Kubernetes, etc.)

2. **Implement a Secrets Manager**
   - AWS Secrets Manager
   - HashiCorp Vault
   - Azure Key Vault
   - Google Cloud Secret Manager

   Example setup for Rust:

   ```rust
   // Load secrets from AWS Secrets Manager
   let secret = aws_secretsmanager_client.get_secret_value("fiddupay/jwt_secret").await?;
   let jwt_secret = secret.secret_string;
   ```

3. **Rotate Credentials Regularly**
   - JWT_SECRET: Rotate every 3-6 months
   - API_KEYS: Rotate every 90 days
   - Database credentials: Rotate every 6 months
   - Webhook signing keys: Rotate every year

4. **Monitor Secret Access**
   - Enable audit logging for all secrets access
   - Alert on unauthorized access attempts
   - Use least-privilege access controls

### ❌ DON'T - Avoid These Practices

- **Never commit `.env` files to git** - Add to `.gitignore`
- **Never log sensitive data** - Not even debug logs
- **Never expose secrets in API responses**
- **Never hardcode API keys, tokens, or passwords**
- **Never return internal error details to clients**
- **Never store plain-text passwords** - Always use bcrypt/argon2

---

## Fixed Issues

### 1. ✅ Webhook Signing Secret Removed from API Response

**File**: `backend/src/api/settings_handlers.rs`

- Removed `webhook_signing_secret` from JSON response
- Clients should fetch this only once during initial setup

### 2. ✅ Error Logging Sanitized

**Files Modified**:

- `backend/src/api/settings_handlers.rs` - Replaced `eprintln!()` with structured logging
- `backend/src/api/public_handlers.rs` - Replaced `eprintln!()` with structured logging
- Now uses `tracing::error!()` and `tracing::warn!()` which respect log levels

### 3. ✅ Authorization Headers Protected

**File**: `backend/src/middleware/auth.rs`

- Removed full bearer token logging
- Now only logs "Malformed Authorization header format" without the actual token

### 4. ✅ Sensitive Data Removed from Debug Logs

**File**: `backend/src/payment/verifier.rs`

- Removed wallet addresses from debug logs
- Removed payment amounts from debug logs
- Removed transaction timestamps from debug logs
- Now logs only payment_id and generic failure reason

### 5. ✅ Webhook Response Bodies Not Logged

**File**: `backend/src/services/webhook_service.rs`

- Removed full response body from warning logs
- Now only logs HTTP status code

---

## Deployment Checklist

- [ ] Rotate all compromised secrets (JWT_SECRET, ENCRYPTION_KEY, WEBHOOK_SIGNING_KEY, DATABASE_PASSWORD)
- [ ] Remove/invalidate all old JWT tokens
- [ ] Force password reset for all admin users
- [ ] Enable `.env` in `.gitignore` (if not already)
- [ ] Set LOG_LEVEL to `info` or `warn` in production (never `debug`)
- [ ] Use secrets manager for all credentials
- [ ] Enable audit logging for all secrets access
- [ ] Set up monitoring/alerting for unauthorized secret access
- [ ] Review webhook receivers - signatures are now invalid after secret rotation
- [ ] Test all endpoints with new secrets before production deployment

---

## Environment Setup Examples

### Docker Compose with Secrets

```yaml
version: "3.8"
services:
  backend:
    image: backend:latest
    environment:
      DATABASE_URL: ${DATABASE_URL}
      JWT_SECRET: ${JWT_SECRET}
      ENCRYPTION_KEY: ${ENCRYPTION_KEY}
      LOG_LEVEL: "info"
    secrets:
      - jwt_secret
      - encryption_key
secrets:
  jwt_secret:
    external: true
  encryption_key:
    external: true
```

### Kubernetes Secrets

```bash
kubectl create secret generic fiddupay-secrets \
  --from-literal=JWT_SECRET=$JWT_SECRET \
  --from-literal=ENCRYPTION_KEY=$ENCRYPTION_KEY \
  --from-literal=DATABASE_PASSWORD=$DB_PASSWORD
```

### Vercel Deployment

Use Environment Variables settings in Vercel dashboard:

- Set `JWT_SECRET`, `ENCRYPTION_KEY`, etc. in Production environment
- These are injected at runtime, never committed to git

---

## Ongoing Security Monitoring

1. **Enable Secret Scanning in CI/CD**

   ```bash
   # Install TruffleHog or GitGuardian
   pip install truffleHog
   truffleHog filesystem . --json
   ```

2. **Monitor Log Output**
   - Configure centralized logging (ELK, CloudWatch, Datadog)
   - Set up alerts for sensitive data patterns
   - Regularly audit logs for accidental data exposure

3. **Regular Security Audits**
   - Quarterly code reviews for security
   - Monitor dependencies for vulnerabilities
   - Test authentication/authorization flows

---

## Additional Resources

- [OWASP: Secrets Management](https://owasp.org/www-community/Secrets_Management_Cheat_Sheet)
- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)
- [PostgreSQL Security](https://www.postgresql.org/docs/current/sql-security.html)
