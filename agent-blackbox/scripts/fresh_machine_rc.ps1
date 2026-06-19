# Fresh Machine RC Validation — zero-knowledge onboarding simulation
$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot\..

$timings = @{}
$sw = [System.Diagnostics.Stopwatch]::StartNew()

# --- Step 1: pristine state ---
Remove-Item Env:UPMARTO_URL -ErrorAction SilentlyContinue
Remove-Item Env:BLACKBOX_URL -ErrorAction SilentlyContinue
if (Test-Path .upmarto) { Remove-Item -Recurse -Force .upmarto }
$timings["clean_state"] = $sw.Elapsed.TotalSeconds

# --- Step 2: backend (simulate cargo run — probe announcement) ---
$discoveredUrl = $null
foreach ($port in 59000..60000) {
    try {
        $r = Invoke-WebRequest -Uri "http://127.0.0.1:$port/config" -UseBasicParsing -TimeoutSec 1
        $cfg = $r.Content | ConvertFrom-Json
        if ($cfg.api_version -eq "v1") { $discoveredUrl = "http://127.0.0.1:$port"; break }
    } catch { }
}
$timings["backend_discoverable"] = $sw.Elapsed.TotalSeconds
if (-not $discoveredUrl) { throw "No backend found — run: cargo run --bin upmarto-server" }

# --- Step 4: upmarto init ---
$initOut = cargo run -q -p upmarto-cli -- init 2>&1 | Out-String
$timings["after_init"] = $sw.Elapsed.TotalSeconds

$config = Get-Content .upmarto\config.json -Raw | ConvertFrom-Json
$runtime = if (Test-Path .upmarto\runtime.json) { Get-Content .upmarto\runtime.json -Raw | ConvertFrom-Json } else { $null }

# --- Step 6: workflow ---
Remove-Item Env:UPMARTO_URL -ErrorAction SilentlyContinue
$sessionOut = cargo run -q -p upmarto-cli -- session 2>&1 | Out-String
$sessionId = ($sessionOut -split "`n" | Where-Object { $_ -match "session_id:" }) -replace "session_id:\s*", "" | ForEach-Object { $_.Trim() }
$wfStart = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()

$CLI = "cargo run -q -p upmarto-cli --"
& $CLI track --type file_opened --path "src/main.rs" 2>&1 | Out-Null
& $CLI track --type file_modified --path "src/auth.rs" 2>&1 | Out-Null
& $CLI track --type test_failed --test "auth_session_expiry" --error "session token not refreshed" 2>&1 | Out-Null
& $CLI track --type file_modified --path "src/auth.rs" 2>&1 | Out-Null
& $CLI track --type test_passed --test "auth_session_expiry" 2>&1 | Out-Null
& $CLI track --type git_commit --message "fix: auth session expiry handling" 2>&1 | Out-Null

$timings["first_timeline"] = $sw.Elapsed.TotalSeconds

# --- Step 7: explain ---
$explainOut = cargo run -q -p upmarto-cli -- explain $sessionId 2>&1 | Out-String
$timings["first_explain"] = $sw.Elapsed.TotalSeconds

# --- Step 8: verification ---
$apiUrl = $config.api_url
$integrity = (Invoke-WebRequest -Uri "$apiUrl/debug/integrity" -UseBasicParsing).Content | ConvertFrom-Json
$timeline = (Invoke-WebRequest -Uri "$apiUrl/timeline?session_id=$sessionId" -UseBasicParsing).Content | ConvertFrom-Json
$newEvents = @($timeline.events | Where-Object { $_.timestamp -ge $wfStart })
$queueLines = if (Test-Path .upmarto\queue.jsonl) { (Get-Content .upmarto\queue.jsonl | Where-Object { $_.Trim() }).Count } else { 0 }

$workflowTypes = @("file_opened","file_modified","test_failed","file_modified","test_passed","git_commit")
$missing = @($workflowTypes | Where-Object { $_ -notin ($newEvents | ForEach-Object { $_.event_type }) })

$rootCause = if ($explainOut -match "Root cause[\s\S]*?\r?\n(.+)") { $Matches[1].Trim() } else { "?" }

$result = [ordered]@{
    discovered_backend_url = $discoveredUrl
    init_output = $initOut.Trim()
    config_api_url = $config.api_url
    runtime_api_url = $runtime.api_url
    url_match = ($config.api_url -eq $discoveredUrl)
    session_id = $sessionId
    new_event_count = $newEvents.Count
    missing_event_types = ($missing -join ",")
    queue_lines = $queueLines
    integrity_status = $integrity.status
    jsonl_sqlite_parity = ($integrity.jsonl_line_count -eq $integrity.sqlite_indexed_count)
    explain_root_cause = $rootCause
    timings_seconds = $timings
    onboarding_commands = @(
        "cargo run --bin upmarto-server",
        "cargo run -p upmarto-cli -- init",
        "cargo run -p upmarto-cli -- track ... (6 events)",
        "cargo run -p upmarto-cli -- explain <session_id>"
    )
}
$result | ConvertTo-Json -Depth 5
