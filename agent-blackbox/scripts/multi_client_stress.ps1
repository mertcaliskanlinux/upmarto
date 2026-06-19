# Upmarto multi-client production stress test
$ErrorActionPreference = "Continue"
Set-Location $PSScriptRoot\..

$base = if ($env:UPMARTO_URL) { $env:UPMARTO_URL } else { "http://127.0.0.1:59245" }
$env:UPMARTO_URL = $base
$ws = (Get-Location).Path

$sessionOut = & cargo run -q -p upmarto-cli -- session 2>&1 | Out-String
$sessionId = ($sessionOut -split "`n" | Where-Object { $_ -match "session_id:" }) -replace "session_id:\s*", "" | ForEach-Object { $_.Trim() }
$projectId = ($sessionOut -split "`n" | Where-Object { $_ -match "project_id:" }) -replace "project_id:\s*", "" | ForEach-Object { $_.Trim() }
Write-Host "SESSION: $sessionId | PROJECT: $projectId | API: $base"

$testStart = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
$beforeTimeline = (Invoke-WebRequest -Uri "$base/timeline?session_id=$sessionId" -UseBasicParsing).Content | ConvertFrom-Json
Write-Host "Timeline baseline: $($beforeTimeline.events.Count) events"

$payloadDir = ".upmarto\stress-payloads"
New-Item -ItemType Directory -Force -Path $payloadDir | Out-Null
Set-Content "$payloadDir\01_vscode_open.json" '{"path":"src/payment.rs","source":"vscode"}' -Encoding utf8
Set-Content "$payloadDir\03_cli_test_run.json" '{"test":"payment_integration","source":"cli"}' -Encoding utf8
Set-Content "$payloadDir\04_test_failed.json" '{"test":"payment_integration","error":"card declined","source":"cli"}' -Encoding utf8
Set-Content "$payloadDir\05_vscode_fix.json" '{"path":"src/payment.rs","source":"vscode","change":"retry logic"}' -Encoding utf8
Set-Content "$payloadDir\06_test_passed.json" '{"test":"payment_integration","source":"cli"}' -Encoding utf8
Set-Content "$payloadDir\07_git_commit.json" '{"message":"fix: payment retry on decline","source":"cli","hash":"abc123"}' -Encoding utf8

$trace = @()

Write-Host "`n=== [1] VSCode: file_opened ==="
& cargo run -q -p upmarto-cli -- track --type file_opened --payload-file "$payloadDir\01_vscode_open.json" 2>&1 | Out-Null
$trace += "[VSCode/SDK] file_opened src/payment.rs"

Start-Sleep -Milliseconds 80

Write-Host "=== [2] Cursor: file_modified (hook.js) ==="
$env:CURSOR_PROJECT_DIR = $ws
$hookPath = ($ws + "/src/payment.rs") -replace '\\', '/'
"{`"path`":`"$hookPath`",`"tool`":`"StrReplace`"}" | node upmarto-cursor/dist/hook.js afterFileEdit 2>&1 | Out-Null
Start-Sleep -Milliseconds 600
& cargo run -q -p upmarto-cli -- flush 2>&1 | Out-Null
$trace += "[Cursor/hook] file_modified src/payment.rs"

Write-Host "=== [3-7] CLI: test_run, test_failed, fix, test_passed, git_commit ==="
& cargo run -q -p upmarto-cli -- track --type test_run --payload-file "$payloadDir\03_cli_test_run.json" 2>&1 | Out-Null
$trace += "[CLI] test_run payment_integration"
& cargo run -q -p upmarto-cli -- track --type test_failed --payload-file "$payloadDir\04_test_failed.json" 2>&1 | Out-Null
$trace += "[CLI] test_failed payment_integration"
& cargo run -q -p upmarto-cli -- track --type file_modified --payload-file "$payloadDir\05_vscode_fix.json" 2>&1 | Out-Null
$trace += "[VSCode/SDK] file_modified fix"
& cargo run -q -p upmarto-cli -- track --type test_passed --payload-file "$payloadDir\06_test_passed.json" 2>&1 | Out-Null
$trace += "[CLI] test_passed payment_integration"
& cargo run -q -p upmarto-cli -- track --type git_commit --payload-file "$payloadDir\07_git_commit.json" 2>&1 | Out-Null
$trace += "[CLI] git_commit"

Write-Host "=== [8] Cursor: agent_message (hook.js) ==="
'{"response":"Applied payment retry fix and verified tests."}' | node upmarto-cursor/dist/hook.js afterAgentResponse 2>&1 | Out-Null
Start-Sleep -Milliseconds 600
& cargo run -q -p upmarto-cli -- flush 2>&1 | Out-Null
$trace += "[Cursor/hook] agent_message"

Write-Host "`n=== SDK RETRY TEST ==="
$savedUrl = $env:UPMARTO_URL
$env:UPMARTO_URL = "http://127.0.0.1:1"
$retryFail = & cargo run -q -p upmarto-cli -- track --type agent_message --message "retry_probe" 2>&1 | Out-String
$env:UPMARTO_URL = $savedUrl
$queueLines = if (Test-Path .upmarto\queue.jsonl) { (Get-Content .upmarto\queue.jsonl | Where-Object { $_.Trim() }).Count } else { 0 }
Write-Host "Failure (expected): $($retryFail.Trim())"
Write-Host "Queue retained: $queueLines line(s)"
$flushRecovery = & cargo run -q -p upmarto-cli -- flush 2>&1 | Out-String
Write-Host "Recovery flush: $($flushRecovery.Trim())"
$queueAfter = if (Test-Path .upmarto\queue.jsonl) { (Get-Content .upmarto\queue.jsonl -ErrorAction SilentlyContinue | Where-Object { $_.Trim() }).Count } else { 0 }

Write-Host "`n=== [9] CLI: explain ==="
$explainOut = & cargo run -q -p upmarto-cli -- explain $sessionId 2>&1 | Out-String

Write-Host "`n=== TIMELINE ANALYSIS ==="
$timeline = (Invoke-WebRequest -Uri "$base/timeline?session_id=$sessionId" -UseBasicParsing).Content | ConvertFrom-Json
$newEvents = @($timeline.events | Where-Object { $_.timestamp -ge $testStart })
Write-Host "Total session events: $($timeline.events.Count) | New this run: $($newEvents.Count)"
Write-Host "New event order:"
foreach ($e in $newEvents) {
    $src = if ($e.payload.source) { $e.payload.source } else { "?" }
    Write-Host "  $($e.event_type) [source=$src] ts=$($e.timestamp)"
}

$dupes = $newEvents | Group-Object { "$($_.event_type)|$($_.timestamp)|$($_.payload.path)" } | Where-Object { $_.Count -gt 1 }
Write-Host "Exact duplicates (type+ts+path): $($dupes.Count)"

$wrongSession = @($newEvents | Where-Object { $_.session_id -ne $sessionId })
Write-Host "Cross-session leaks: $($wrongSession.Count)"

$ts = @($newEvents | ForEach-Object { $_.timestamp })
$sorted = $ts | Sort-Object
$ordered = ($ts -join ',') -eq ($sorted -join ',')
Write-Host "Chronological ordering: $(if ($ordered) { 'PASS' } else { 'FAIL' })"

# Expected types in stress run (may include retry_probe agent_message)
$expectedCore = @('file_opened','file_modified','test_run','test_failed','file_modified','test_passed','git_commit','agent_message')
$actualTypes = @($newEvents | ForEach-Object { $_.event_type })
$missing = @($expectedCore | Where-Object { $_ -notin $actualTypes })
Write-Host "Missing core event types: $(if ($missing.Count -eq 0) { 'none' } else { $missing -join ', ' })"

Write-Host "`n=== INTEGRITY ==="
(Invoke-WebRequest -Uri "$base/debug/integrity" -UseBasicParsing).Content

Write-Host "`n=== EXPLAIN ==="
Write-Host $explainOut

Write-Host "`n=== EVENT TRACE ==="
$trace | ForEach-Object { Write-Host $_ }

Write-Host "`n=== RETRY: queue after recovery = $queueAfter (expect 0) ==="
