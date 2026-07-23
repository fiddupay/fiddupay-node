# Script to push SDK changes to the isolated SDK repository
# Usage: .\scripts\push-sdk.ps1 -Branch main
# Usage with tag: .\scripts\push-sdk.ps1 -Branch main -Tag v2.6.22

param (
    [string]$Branch = "main",
    [string]$Tag = ""
)

$RemoteUrl = "git@github.com:fiddupay/fiddupay-node.git"

Write-Host "Pushing 'fiddupay-node-sdk' folder to $RemoteUrl branch '$Branch'..." -ForegroundColor Cyan

# Run npm audit fix inside the SDK folder before pushing
Write-Host "Running npm audit fix in fiddupay-node-sdk..." -ForegroundColor Cyan
Push-Location fiddupay-node-sdk
npm audit fix
Pop-Location
Write-Host "npm audit step complete (warnings above are non-blocking)." -ForegroundColor Yellow

# Use git subtree to push only the subfolder
git subtree push --prefix fiddupay-node-sdk "$RemoteUrl" "$Branch"

if ($LASTEXITCODE -eq 0) {
    Write-Host "SDK Code Push Successful!" -ForegroundColor Green

    if ($Tag) {
        Write-Host "Pushing tag '$Tag' to $RemoteUrl..." -ForegroundColor Cyan
        $CurrentCommit = git rev-parse HEAD
        git push "$RemoteUrl" "${CurrentCommit}:refs/tags/${Tag}" --force
        if ($LASTEXITCODE -eq 0) {
            Write-Host "SDK Tag Push Successful!" -ForegroundColor Green
        } else {
            Write-Host "SDK Tag Push Failed." -ForegroundColor Red
        }
    }
} else {
    Write-Host "SDK Code Push Failed. You might need to force push or handle conflicts." -ForegroundColor Red
}
