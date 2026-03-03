# Script to push P2P-Frontend changes to the isolated P2P repository
# Usage: .\scripts\push-p2p.ps1 -Branch main

param (
    [string]$Branch = "main",
    [string]$Tag = ""
)

$RemoteUrl = "https://github.com/fiddupay/fiddupay-p2p.git"

Write-Host "Pushing 'p2p-frontend' folder to $RemoteUrl branch '$Branch'..." -ForegroundColor Cyan

# Use git subtree to push only the subfolder
git subtree push --prefix p2p-frontend "$RemoteUrl" "$Branch"

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ P2P-Frontend Code Push Successful!" -ForegroundColor Green
    
    if ($Tag) {
        Write-Host "🏷️ Pushing tag '$Tag' to $RemoteUrl..." -ForegroundColor Cyan
        $CurrentCommit = git rev-parse HEAD
        git push "$RemoteUrl" "$($CurrentCommit):refs/tags/$Tag"
        if ($LASTEXITCODE -eq 0) {
            Write-Host "✅ P2P Tag Push Successful!" -ForegroundColor Green
        } else {
            Write-Host "❌ P2P Tag Push Failed." -ForegroundColor Red
        }
    }
} else {
    Write-Host "❌ P2P Code Push Failed. You might need to force push or handle conflicts." -ForegroundColor Red
    Write-Host "Try running: git subtree split --prefix p2p-frontend -b p2p-split; git push $RemoteUrl p2p-split:$($Branch) --force; git branch -D p2p-split" -ForegroundColor Yellow
}
