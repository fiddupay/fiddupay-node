const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const MONOREPO_PKG_PATH = path.join(__dirname, '../package.json');
const SDK_PKG_PATH = path.join(__dirname, '../fiddupay-node-sdk/package.json');

function getVersion() {
    const pkg = JSON.parse(fs.readFileSync(MONOREPO_PKG_PATH, 'utf8'));
    return pkg.version;
}

function updateVersion(newVersion) {
    // Update Monorepo package.json
    const monorepoPkg = JSON.parse(fs.readFileSync(MONOREPO_PKG_PATH, 'utf8'));
    monorepoPkg.version = newVersion;
    fs.writeFileSync(MONOREPO_PKG_PATH, JSON.stringify(monorepoPkg, null, 2) + '\n');

    // Update SDK package.json
    const sdkPkg = JSON.parse(fs.readFileSync(SDK_PKG_PATH, 'utf8'));
    sdkPkg.version = newVersion;
    fs.writeFileSync(SDK_PKG_PATH, JSON.stringify(sdkPkg, null, 2) + '\n');

    console.log(`✅ Updated version to ${newVersion} in both packages.`);
}

function run() {
    const args = process.argv.slice(2);
    const newVersion = args[0];

    if (!newVersion) {
        console.error('❌ Please provide a version (e.g., node scripts/release.js 2.4.1)');
        process.exit(1);
    }

    try {
        // 1. Update versions
        updateVersion(newVersion);

        // 2. Commit changes
        console.log('📦 Committing version bump...');
        execSync(`git add . && git commit -m "chore: bump version to ${newVersion}"`, { stdio: 'inherit' });

        // 3. Push to main repo
        console.log('🚀 Pushing to main repository...');
        execSync('git push origin main', { stdio: 'inherit' });

        // 4. Sync to SDK and tag
        console.log(`📡 Syncing to SDK repository with tag v${newVersion}...`);
        execSync(`bash scripts/push-sdk.sh main v${newVersion}`, { stdio: 'inherit' });

        console.log(`\n🎉 Release v${newVersion} complete!`);
        console.log(`🔗 Main: https://github.com/fiddupay/fiddupay`);
        console.log(`🔗 SDK:  https://github.com/fiddupay/fiddupay-node`);
    } catch (error) {
        console.error('❌ Release failed:', error.message);
        process.exit(1);
    }
}

run();
