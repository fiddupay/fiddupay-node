# Script to push Main Frontend changes to the isolated Frontend repository
# Usage: .\scripts\push-frontend.ps1 -Branch main

param (
    [string]$Branch = "main",
    [string]$Tag = ""
)

$RemoteUrl = "https://github.com/fiddupay/fiddupay-frontend.git"

Write-Host "Pushing 'frontend' folder to $RemoteUrl branch '$Branch'..." -ForegroundColor Cyan

# Use git subtree to push only the subfolder
git subtree push --prefix frontend "$RemoteUrl" "$Branch"

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Frontend Code Push Successful!" -ForegroundColor Green
    
    if ($Tag) {
        Write-Host "🏷️ Pushing tag '$Tag' to $RemoteUrl..." -ForegroundColor Cyan
        $CurrentCommit = git rev-parse HEAD
        git push "$RemoteUrl" "$($CurrentCommit):refs/tags/$Tag"
        if ($LASTEXITCODE -eq 0) {
            Write-Host "✅ Frontend Tag Push Successful!" -ForegroundColor Green
        } else {
            Write-Host "❌ Frontend Tag Push Failed." -ForegroundColor Red
        }
    }
} else {
    Write-Host "❌ Frontend Code Push Failed. You might need to force push or handle conflicts." -ForegroundColor Red
    Write-Host "Try running: git subtree split --prefix frontend -b frontend-split; git push $RemoteUrl frontend-split:$($Branch) --force; git branch -D frontend-split" -ForegroundColor Yellow
}
