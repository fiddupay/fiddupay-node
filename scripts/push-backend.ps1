# Script to push Backend changes to the isolated Backend repository
# Usage: .\scripts\push-backend.ps1 -Branch main

param (
    [string]$Branch = "main",
    [string]$Tag = ""
)

$RemoteUrl = "https://github.com/fiddupay/fiddupay-backend.git"

Write-Host "Pushing 'backend' folder to $RemoteUrl branch '$Branch'..." -ForegroundColor Cyan

# Use git subtree to push only the subfolder
git subtree push --prefix backend "$RemoteUrl" "$Branch"

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Backend Code Push Successful!" -ForegroundColor Green
    
    if ($Tag) {
        Write-Host "🏷️ Pushing tag '$Tag' to $RemoteUrl..." -ForegroundColor Cyan
        $CurrentCommit = git rev-parse HEAD
        git push "$RemoteUrl" "$($CurrentCommit):refs/tags/$Tag"
        if ($LASTEXITCODE -eq 0) {
            Write-Host "✅ Backend Tag Push Successful!" -ForegroundColor Green
        } else {
            Write-Host "❌ Backend Tag Push Failed." -ForegroundColor Red
        }
    }
} else {
    Write-Host "❌ Backend Code Push Failed. You might need to force push or handle conflicts." -ForegroundColor Red
    Write-Host "Try running: git subtree split --prefix backend -b backend-split; git push $RemoteUrl backend-split:$($Branch) --force; git branch -D backend-split" -ForegroundColor Yellow
}
