$ErrorActionPreference = "Stop"
Write-Host "--- Starting Publish Script ---"

# -----------------------------------------------------------------------------
# 1. Configuration & Environment Loading
# -----------------------------------------------------------------------------

# Load .env file if it exists
$envLocations = @(
    (Join-Path $PSScriptRoot ".env"),
    (Join-Path $PSScriptRoot "..\.env")
)

foreach ($envPath in $envLocations) {
    if (Test-Path $envPath) {
        Write-Host "Loading .env from $envPath"
        Get-Content $envPath | ForEach-Object {
            if ($_ -match "^\s*([^#=]+?)\s*=\s*(.*)$") {
                $key = $matches[1]
                $value = $matches[2].Trim()
                # Remove quotes if present
                if ($value -match "^['`"](.*)['`"]$") { $value = $matches[1] }
                [Environment]::SetEnvironmentVariable($key, $value, "Process")
            }
        }
        break # Stop after finding the first .env
    }
}

# Check Environment Variables
Write-Host "Checking environment variables..."
if (-not $env:TAURI_SIGNING_PRIVATE_KEY) { Write-Error "TAURI_SIGNING_PRIVATE_KEY is not set."; exit 1 }
if (-not $env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD) { Write-Error "TAURI_SIGNING_PRIVATE_KEY_PASSWORD is not set."; exit 1 }
if (-not $env:TARGET_REPO_URL) { Write-Error "TARGET_REPO_URL is not set."; exit 1 }
if (-not $env:TARGET_REPO_TOKEN) { Write-Error "TARGET_REPO_TOKEN is not set."; exit 1 }
Write-Host "Environment variables present."

# Parse Repo Owner and Name
Write-Host "Parsing target repository info..."
$repoUrl = $env:TARGET_REPO_URL
if ($repoUrl -match "github\.com[:/]([^/]+)/([^/]+?)(\.git)?$") {
    $owner = $matches[1]
    $repo = $matches[2]
} else {
    Write-Error "Could not parse owner and repo from TARGET_REPO_URL: $repoUrl"
    exit 1
}
Write-Host "Target Repo: $owner/$repo"

# GitHub API Setup
$headers = @{
    "Authorization" = "token $env:TARGET_REPO_TOKEN"
    "Accept"        = "application/vnd.github.v3+json"
}

# -----------------------------------------------------------------------------
# 2. Pre-flight Checks (Fail Fast)
# -----------------------------------------------------------------------------
Write-Host "`n--- Performing Pre-flight Checks ---"

# 2.1 Check Local Files & Version
Write-Host "Checking local configuration..."
$tauriConfPath = Join-Path $PSScriptRoot "..\src-tauri\tauri.conf.json"
if (-not (Test-Path $tauriConfPath)) { Write-Error "tauri.conf.json not found at $tauriConfPath"; exit 1 }
$tauriConf = Get-Content $tauriConfPath | ConvertFrom-Json
$version = $tauriConf.version
$tagName = "v$version"
Write-Host "Detected Version: $version"

$templatePath = Join-Path $PSScriptRoot "..\update_template.json"
if (-not (Test-Path $templatePath)) { Write-Error "Template file not found at $templatePath"; exit 1 }
Write-Host "Template file found."

# 2.2 Verify Token & Permissions
Write-Host "Verifying GitHub Token..."
try {
    $response = Invoke-WebRequest -Uri "https://api.github.com/rate_limit" -Headers $headers -Method Get -UseBasicParsing
    $scopes = $response.Headers["X-OAuth-Scopes"]
    Write-Host "Token Scopes: $scopes"
    
    # Check if we can access the repo
    $repoInfo = Invoke-RestMethod -Uri "https://api.github.com/repos/$owner/$repo" -Headers $headers -Method Get
    Write-Host "Repo Access: OK ($($repoInfo.full_name))"
    
    if ($repoInfo.permissions.push -ne $true) {
        Write-Warning "Token does not appear to have push permissions to this repository."
    }
} catch {
    Write-Error "Token verification failed. Please check if TARGET_REPO_TOKEN is valid and has correct permissions.`nError: $_"
    exit 1
}

# 2.3 Check Remote State (releases.json & Release Tag)
Write-Host "Checking remote repository state..."
$releasesJsonPath = "releases.json"
$fileApiUrl = "https://api.github.com/repos/$owner/$repo/contents/$releasesJsonPath"
$remoteSha = $null

try {
    $fileInfo = Invoke-RestMethod -Uri $fileApiUrl -Headers $headers -Method Get -ErrorAction Stop
    $remoteSha = $fileInfo.sha
    Write-Host "Found existing releases.json (SHA: $remoteSha)"
} catch {
    if ($_.Exception.Response.StatusCode -eq [System.Net.HttpStatusCode]::NotFound) {
        Write-Host "Remote releases.json not found (will be created)."
    } else {
        Write-Error "Failed to check releases.json: $_"
        exit 1
    }
}

$releasesApiUrl = "https://api.github.com/repos/$owner/$repo/releases"
try {
    $existingRelease = Invoke-RestMethod -Uri "$releasesApiUrl/tags/$tagName" -Headers $headers -Method Get -ErrorAction Stop
    Write-Host "Release $tagName already exists. Assets will be updated."
    
    # Validate Upload URL immediately if release exists
    if (-not $existingRelease.upload_url) {
        Write-Error "Existing release is missing 'upload_url' property."
        exit 1
    }
    Write-Host "DEBUG: Raw upload_url: '$($existingRelease.upload_url)'"
    $baseUploadUrl = $existingRelease.upload_url -replace "\{.*\}", ""
    $baseUploadUrl = $baseUploadUrl.Trim()
    Write-Host "DEBUG: Processed baseUploadUrl: '$baseUploadUrl'"

    if ([string]::IsNullOrWhiteSpace($baseUploadUrl)) {
        Write-Error "Processed baseUploadUrl is empty. Raw: '$($existingRelease.upload_url)'"
        exit 1
    }
    
    # -------------------------------------------------------------------------
    # 2.4 Test Asset Upload (Pre-flight)
    # -------------------------------------------------------------------------
    Write-Host "Testing asset upload capability..."
    $testFileName = "preflight_test_$((Get-Date).Ticks).txt"
    $testFilePath = Join-Path $env:TEMP $testFileName
    "Test content" | Set-Content $testFilePath

    try {
        # Ensure baseUploadUrl is clean
        $cleanBaseUrl = $baseUploadUrl.Trim()
        Write-Host "DEBUG: Using Base URL: '$cleanBaseUrl'"
        
        # Construct URI using concatenation
        $testUploadUriString = $cleanBaseUrl + "?name=" + $testFileName
        Write-Host "DEBUG: Constructed URI String: '$testUploadUriString'"
        
        $testUploadUri = New-Object System.Uri($testUploadUriString)
        Write-Host "Test Upload URI: $($testUploadUri.AbsoluteUri)"

        Write-Host "Attempting to upload test file..."
        try {
            $testContent = Get-Content $testFilePath -Raw
            # Use Invoke-RestMethod to avoid HttpClient deadlocks
            $testResponse = Invoke-RestMethod -Uri $testUploadUri.AbsoluteUri -Method Post -Headers $headers -Body $testContent -ContentType "text/plain" -TimeoutSec 60
            Write-Host "Test upload successful. Asset URL: $($testResponse.browser_download_url)"
            
            # Cleanup: Delete the test asset
            $assetUrl = $testResponse.url
            Write-Host "Cleaning up test asset..."
            Invoke-RestMethod -Uri $assetUrl -Headers $headers -Method Delete
            Write-Host "Test asset deleted."
        } catch {
            Write-Error "Test upload failed. Error: $_"
            exit 1
        }

    } catch {
        Write-Error "Pre-flight upload test failed. Error: $_"
        exit 1
    } finally {
        if ($testFileStream) { $testFileStream.Close() }
        if ($testClient) { $testClient.Dispose() }
        if (Test-Path $testFilePath) { Remove-Item $testFilePath }
    }
    
} catch {
    if ($_.Exception.Response.StatusCode -eq [System.Net.HttpStatusCode]::NotFound) {
        Write-Host "Release $tagName does not exist (will be created)."
    } else {
        Write-Error "Failed to check release status: $_"
        exit 1
    }
}

Write-Host "Pre-flight checks passed."

# -----------------------------------------------------------------------------
# 3. Build
# -----------------------------------------------------------------------------
Write-Host "`n--- Starting Build ---"
Write-Host "Building Tauri app..."
cargo tauri build
if ($LASTEXITCODE -ne 0) { Write-Error "Build failed"; exit $LASTEXITCODE }
Write-Host "Tauri build completed successfully."

# -----------------------------------------------------------------------------
# 4. Post-Build Processing
# -----------------------------------------------------------------------------
Write-Host "`n--- Post-Build Processing ---"

# 4.1 Find Artifacts
Write-Host "Locating build artifacts..."
$nsisDir = Join-Path $PSScriptRoot "..\src-tauri\target\release\bundle\nsis"
$exeFile = Get-ChildItem -Path $nsisDir -Filter "*${version}*-setup.exe" | Select-Object -First 1
if (-not $exeFile) { Write-Error "Could not find setup .exe in $nsisDir matching version $version"; exit 1 }

$sigFile = Get-Item "$($exeFile.FullName).sig"
if (-not $sigFile) { Write-Error "Could not find .sig file for $($exeFile.Name)"; exit 1 }

Write-Host "Found artifact: $($exeFile.Name)"

# 4.2 Generate releases.json content
Write-Host "Generating releases.json content..."
$fileName = $exeFile.Name

Write-Host "Reading signature from $($sigFile.FullName)..."
$sigContent = Get-Content $sigFile.FullName -Raw
if (-not $sigContent) { Write-Error "Signature file is empty"; exit 1 }
$sigContent = $sigContent.Trim()

# Construct the object explicitly
$updateData = @{
    "version" = "v$version"
    "notes" = "Update to v$version"
    "pub_date" = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
    "platforms" = @{
        "windows-x86_64" = @{
            "signature" = $sigContent
            "url" = "https://github.com/$owner/$repo/releases/download/$tagName/$fileName"
        }
    }
}

$releasesJsonContent = $updateData | ConvertTo-Json -Depth 10
Write-Host "releases.json content generated."

# Save releases.json to root
$releasesJsonPath = Join-Path $PSScriptRoot "..\releases.json"
[System.IO.File]::WriteAllText($releasesJsonPath, $releasesJsonContent)
Write-Host "releases.json saved to $releasesJsonPath"

# 4.3 Commit and Push
Write-Host "Committing and pushing changes..."
git add .
git commit -m "chore: update releases.json for v$version"
git push
Write-Host "Changes pushed successfully."

# 4.4 Create or Get Release
Write-Host "Handling Release $tagName..."
try {
    $release = Invoke-RestMethod -Uri "$releasesApiUrl/tags/$tagName" -Headers $headers -Method Get -ErrorAction Stop
    Write-Host "Release $tagName retrieved."
} catch {
    Write-Host "Creating release $tagName..."
    $body = @{
        tag_name = $tagName
        name     = "v$version"
        body     = "Release v$version"
        draft    = $false
        prerelease = $false
    } | ConvertTo-Json

    $release = Invoke-RestMethod -Uri $releasesApiUrl -Headers $headers -Method Post -Body $body -ContentType "application/json"
    Write-Host "Release created."
}

# 4.5 Upload Asset
Write-Host "Handling Asset Upload..."

if (-not $release.upload_url) {
    Write-Error "Release object is missing upload_url. Release info: $($release | ConvertTo-Json -Depth 2)"
    exit 1
}

Write-Host "DEBUG: Raw upload_url: '$($release.upload_url)'"
$uploadUrl = $release.upload_url -replace "\{.*\}", ""
$uploadUrl = $uploadUrl.Trim()
Write-Host "DEBUG: Processed uploadUrl: '$uploadUrl'"

if ([string]::IsNullOrWhiteSpace($uploadUrl)) {
    Write-Error "Processed uploadUrl is empty."
    exit 1
}

# Encode filename to ensure valid URI
$encodedFileName = [Uri]::EscapeDataString($fileName)
# Use concatenation to avoid interpolation issues
$assetUploadUriString = $uploadUrl + "?name=" + $encodedFileName

Write-Host "Constructed Upload URI String: '$assetUploadUriString'"

try {
    $assetUploadUri = New-Object System.Uri($assetUploadUriString)
    Write-Host "Validated URI Object: $($assetUploadUri.AbsoluteUri)"
} catch {
    Write-Error "Failed to create URI object from string: '$assetUploadUriString'. Error: $_"
    exit 1
}

# Check for existing asset
$assets = Invoke-RestMethod -Uri $release.assets_url -Headers $headers -Method Get
$existingAsset = $assets | Where-Object { $_.name -eq $fileName }
if ($existingAsset) {
    Write-Host "Deleting existing asset $($existingAsset.name)..."
    Invoke-RestMethod -Uri $existingAsset.url -Headers $headers -Method Delete
}

Write-Host "Uploading $fileName..."

try {
    Write-Host "Reading file: $($exeFile.FullName)"
    $fileBytes = [System.IO.File]::ReadAllBytes($exeFile.FullName)
    Write-Host "File size: $([math]::Round($fileBytes.Length / 1MB, 2)) MB"
    
    # Use Invoke-RestMethod which is more stable in PS 5.1 for this than HttpClient
    # Timeout set to 10 minutes (600 seconds)
    $response = Invoke-RestMethod -Uri $assetUploadUri.AbsoluteUri -Method Post -Headers $headers -Body $fileBytes -ContentType "application/octet-stream" -TimeoutSec 600
    
    Write-Host "Upload complete."
} catch {
    Write-Error "Failed to upload asset: $_"
    exit 1
}

Write-Host "Publishing complete!"
