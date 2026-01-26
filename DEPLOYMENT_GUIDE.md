# FidduPay Node.js SDK - Modern 2026 Deployment Guide

##  Complete Deployment Guide for NPM & GitHub

### Prerequisites
- Node.js 18+ (LTS recommended)
- npm 9+ or yarn 3+
- GitHub account with 2FA enabled
- NPM account with 2FA enabled

---

## 📦 Step 1: NPM Package Preparation

### 1.1 Update Package.json for Production
```bash
cd fiddupay-node-sdk
```

Update `package.json` with production settings:
```json
{
  "name": "fiddupay-node",
  "version": "1.0.0",
  "description": "Official Node.js SDK for FidduPay cryptocurrency payment gateway",
  "main": "dist/index.js",
  "types": "dist/index.d.ts",
  "repository": {
    "type": "git",
    "url": "git+https://github.com/YOUR_USERNAME/fiddupay-node.git"
  },
  "bugs": {
    "url": "https://github.com/YOUR_USERNAME/fiddupay-node/issues"
  },
  "homepage": "https://github.com/YOUR_USERNAME/fiddupay-node#readme",
  "publishConfig": {
    "access": "public",
    "registry": "https://registry.npmjs.org/"
  }
}
```

### 1.2 Create Essential Files

**Create `.npmignore`:**
```bash
cat > .npmignore << 'EOF'
# Source files
src/
tests/
examples/

# Development files
*.test.ts
*.spec.ts
jest.config.js
tsconfig.json
.eslintrc.js

# Build artifacts
coverage/
.nyc_output/

# Development dependencies
node_modules/
.npm/

# IDE files
.vscode/
.idea/
*.swp
*.swo

# OS files
.DS_Store
Thumbs.db

# Git files
.git/
.gitignore

# Documentation (keep README.md)
docs/
*.md
!README.md

# Environment files
.env*
EOF
```

**Create `LICENSE` file:**
```bash
cat > LICENSE << 'EOF'
MIT License

Copyright (c) 2026 TechyTro Software

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
EOF
```

---

## 🐙 Step 2: GitHub Repository Setup (2026 Method)

### 2.1 Create Repository with GitHub CLI (Modern Way)
```bash
# Install GitHub CLI if not installed
# macOS: brew install gh
# Ubuntu: sudo apt install gh
# Windows: winget install GitHub.cli

# Authenticate with GitHub
gh auth login

# Create repository
gh repo create fiddupay-node --public --description "Official Node.js SDK for FidduPay cryptocurrency payment gateway" --clone

# Move SDK files to the new repo
cp -r fiddupay-node-sdk/* fiddupay-node/
cd fiddupay-node
```

### 2.2 Setup Modern GitHub Actions (2026)

**Create `.github/workflows/ci.yml`:**
```yaml
name: CI/CD Pipeline

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]
  release:
    types: [ published ]

jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        node-version: [18.x, 20.x, 22.x]
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Setup Node.js ${{ matrix.node-version }}
      uses: actions/setup-node@v4
      with:
        node-version: ${{ matrix.node-version }}
        cache: 'npm'
    
    - name: Install dependencies
      run: npm ci
    
    - name: Run linter
      run: npm run lint
    
    - name: Run tests
      run: npm test
    
    - name: Build package
      run: npm run build
    
    - name: Upload coverage to Codecov
      uses: codecov/codecov-action@v4
      if: matrix.node-version == '20.x'

  security:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    
    - name: Run security audit
      run: npm audit --audit-level high
    
    - name: Run Snyk security scan
      uses: snyk/actions/node@master
      env:
        SNYK_TOKEN: ${{ secrets.SNYK_TOKEN }}

  publish:
    needs: [test, security]
    runs-on: ubuntu-latest
    if: github.event_name == 'release'
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Setup Node.js
      uses: actions/setup-node@v4
      with:
        node-version: '20.x'
        registry-url: 'https://registry.npmjs.org'
        cache: 'npm'
    
    - name: Install dependencies
      run: npm ci
    
    - name: Build package
      run: npm run build
    
    - name: Publish to NPM
      run: npm publish
      env:
        NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
    
    - name: Create GitHub Release Assets
      run: |
        tar -czf fiddupay-node-${{ github.event.release.tag_name }}.tar.gz dist/
        gh release upload ${{ github.event.release.tag_name }} fiddupay-node-${{ github.event.release.tag_name }}.tar.gz
      env:
        GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

**Create `.github/workflows/codeql.yml` (Security Analysis):**
```yaml
name: CodeQL Security Analysis

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]
  schedule:
    - cron: '0 6 * * 1'  # Weekly on Monday

jobs:
  analyze:
    name: Analyze
    runs-on: ubuntu-latest
    permissions:
      actions: read
      contents: read
      security-events: write

    steps:
    - name: Checkout repository
      uses: actions/checkout@v4

    - name: Initialize CodeQL
      uses: github/codeql-action/init@v3
      with:
        languages: javascript

    - name: Perform CodeQL Analysis
      uses: github/codeql-action/analyze@v3
```

### 2.3 Setup Repository Settings

**Create `.github/ISSUE_TEMPLATE/bug_report.yml`:**
```yaml
name: Bug Report
description: File a bug report
title: "[Bug]: "
labels: ["bug", "triage"]
body:
  - type: markdown
    attributes:
      value: |
        Thanks for taking the time to fill out this bug report!
  
  - type: input
    id: version
    attributes:
      label: SDK Version
      description: What version of fiddupay-node are you using?
      placeholder: ex. 1.0.0
    validations:
      required: true
  
  - type: textarea
    id: what-happened
    attributes:
      label: What happened?
      description: Also tell us, what did you expect to happen?
      placeholder: Tell us what you see!
    validations:
      required: true
  
  - type: textarea
    id: code-sample
    attributes:
      label: Code Sample
      description: Please provide a minimal code sample that reproduces the issue
      render: typescript
    validations:
      required: true
```

---

##  Step 3: NPM Account Setup (2026 Security)

### 3.1 Modern NPM Authentication
```bash
# Login to NPM with modern auth
npm login

# Enable 2FA (REQUIRED for publishing in 2026)
npm profile enable-2fa auth-and-writes

# Generate automation token for CI/CD
npm token create --type=automation --cidr=0.0.0.0/0
```

### 3.2 Setup NPM Organization (Recommended)
```bash
# Create organization (optional but professional)
npm org create fiddupay

# Add team members
npm team create fiddupay:developers
npm team add fiddupay:developers your-username
```

---

##  Step 4: Modern Deployment Process (2026)

### 4.1 Pre-deployment Checklist
```bash
# 1. Verify build works
npm run build

# 2. Run all tests
npm test

# 3. Check for vulnerabilities
npm audit

# 4. Verify package contents
npm pack --dry-run

# 5. Test local installation
npm pack
npm install -g fiddupay-node-1.0.0.tgz
```

### 4.2 Version Management with Semantic Release
```bash
# Install semantic-release (modern versioning)
npm install --save-dev semantic-release @semantic-release/changelog @semantic-release/git

# Create .releaserc.json
cat > .releaserc.json << 'EOF'
{
  "branches": ["main"],
  "plugins": [
    "@semantic-release/commit-analyzer",
    "@semantic-release/release-notes-generator",
    "@semantic-release/changelog",
    "@semantic-release/npm",
    "@semantic-release/github",
    ["@semantic-release/git", {
      "assets": ["CHANGELOG.md", "package.json"],
      "message": "chore(release): ${nextRelease.version} [skip ci]\n\n${nextRelease.notes}"
    }]
  ]
}
EOF
```

### 4.3 Automated Deployment Setup

**Add to `package.json` scripts:**
```json
{
  "scripts": {
    "release": "semantic-release",
    "release:dry": "semantic-release --dry-run"
  }
}
```

---

##  Step 5: Publication Process (2026 Method)

### 5.1 Manual Publication (First Release)
```bash
# 1. Final build and test
npm run build
npm test

# 2. Update version (if not using semantic-release)
npm version patch  # or minor/major

# 3. Publish to NPM
npm publish

# 4. Create GitHub release
gh release create v1.0.0 --title "v1.0.0 - Initial Release" --notes " Initial release of FidduPay Node.js SDK"

# 5. Push tags
git push --tags
```

### 5.2 Automated Publication (Recommended)
```bash
# 1. Commit with conventional commits
git add .
git commit -m "feat: initial SDK release with payment processing"

# 2. Push to main branch
git push origin main

# 3. Semantic release will automatically:
#    - Determine version bump
#    - Generate changelog
#    - Create GitHub release
#    - Publish to NPM
```

---

##  Step 6: Security & Secrets Setup

### 6.1 GitHub Secrets (Required)
Go to GitHub repo → Settings → Secrets and variables → Actions

Add these secrets:
- `NPM_TOKEN`: Your NPM automation token
- `SNYK_TOKEN`: Snyk security scanning token (optional)

### 6.2 NPM Package Security
```bash
# Enable package signing (2026 feature)
npm config set sign-git-commits true
npm config set sign-git-tags true

# Set up package provenance
npm publish --provenance
```

---

##  Step 7: Monitoring & Analytics (2026)

### 7.1 NPM Package Analytics
```bash
# View download stats
npm view fiddupay-node

# Monitor package health
npx npm-check-updates
```

### 7.2 GitHub Insights Setup
- Enable Dependency Graph
- Enable Dependabot alerts
- Enable Code scanning alerts
- Set up branch protection rules

---

##  Step 8: Post-Deployment

### 8.1 Documentation Updates
```bash
# Update README with installation instructions
# Add badges for build status, coverage, version
# Create CONTRIBUTING.md
# Set up GitHub Pages for documentation
```

### 8.2 Community Setup
```bash
# Create CONTRIBUTING.md
# Set up issue templates
# Create pull request template
# Add code of conduct
```

---

##  Step 9: Continuous Updates (2026 Best Practices)

### 9.1 Automated Dependency Updates
```yaml
# .github/dependabot.yml
version: 2
updates:
  - package-ecosystem: "npm"
    directory: "/"
    schedule:
      interval: "weekly"
    open-pull-requests-limit: 10
    reviewers:
      - "your-username"
```

### 9.2 Release Automation
```bash
# Use conventional commits for automatic versioning:
# feat: new feature (minor version bump)
# fix: bug fix (patch version bump)
# BREAKING CHANGE: breaking change (major version bump)
```

---

##  Verification Checklist

- [ ] Package builds without errors
- [ ] All tests pass
- [ ] NPM package published successfully
- [ ] GitHub repository created with proper settings
- [ ] CI/CD pipeline working
- [ ] Security scanning enabled
- [ ] Documentation complete
- [ ] Version management automated
- [ ] Community guidelines in place

---

## 🚨 Common Issues & Solutions

**Issue**: NPM publish fails with 2FA
**Solution**: Use `npm publish --otp=123456` with your 2FA code

**Issue**: GitHub Actions fails
**Solution**: Check secrets are properly set and have correct permissions

**Issue**: Package size too large
**Solution**: Review `.npmignore` and exclude unnecessary files

**Issue**: TypeScript declarations missing
**Solution**: Ensure `"declaration": true` in `tsconfig.json`

---

This guide follows 2026 best practices with modern security, automation, and community standards! 
