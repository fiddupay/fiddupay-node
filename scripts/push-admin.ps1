# Script to push Admin Dashboard changes to the isolated Admin repository
# Usage: .\scripts\push-admin.ps1 -Branch main

param (
    [string]$Branch = "main",
    [string]$Tag = ""
)

$RemoteUrl = "https://github.com/fiddupay/fiddupay-admin-dashboard.git"

Write-Host "Pushing 'admin-frontend' folder to $RemoteUrl branch '$Branch'..." -ForegroundColor Cyan

# Use git subtree to push only the subfolder
git subtree push --prefix admin-frontend "$RemoteUrl" "$Branch"

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Admin Dashboard Code Push Successful!" -ForegroundColor Green
    
    if ($Tag) {
        Write-Host "🏷️ Pushing tag '$Tag' to $RemoteUrl..." -ForegroundColor Cyan
        $CurrentCommit = git rev-parse HEAD
        git push "$RemoteUrl" "$($CurrentCommit):refs/tags/$Tag"
        if ($LASTEXITCODE -eq 0) {
            Write-Host "✅ Admin Dashboard Tag Push Successful!" -ForegroundColor Green
        } else {
            Write-Host "❌ Admin Dashboard Tag Push Failed." -ForegroundColor Red
        }
    }
} else {
    Write-Host "❌ Admin Dashboard Code Push Failed. You might need to force push or handle conflicts." -ForegroundColor Red
    Write-Host "Try running: git subtree split --prefix admin-frontend -b admin-split; git push $RemoteUrl admin-split:$($Branch) --force; git branch -D admin-split" -ForegroundColor Yellow
}
