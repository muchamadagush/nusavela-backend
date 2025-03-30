# NusaVela Conflict-Free Git Sync & Push Script (PowerShell)
param(
    [switch]$ForceWithLease
)
$ErrorActionPreference = "Stop"

Write-Host "==> Checking Git status and remote origin..." -ForegroundColor Cyan
git remote -v

if ($ForceWithLease) {
    Write-Host "==> Pushing local commit history to origin/main (--force-with-lease)..." -ForegroundColor Green
    git push -u origin main --force-with-lease
} else {
    Write-Host "==> Fetching latest origin/main..." -ForegroundColor Cyan
    git fetch origin main 2>$null
    if ($LASTEXITCODE -eq 0) {
        Write-Host "==> Rebasing local feature commits cleanly on top of origin/main..." -ForegroundColor Cyan
        git pull --rebase -X theirs origin main
    }
    Write-Host "==> Pushing to origin/main..." -ForegroundColor Green
    git push -u origin main
}
Write-Host "==> Done! Repository is synchronized with GitHub." -ForegroundColor Green
