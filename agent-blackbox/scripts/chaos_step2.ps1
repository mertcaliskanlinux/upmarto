# Chaos Step 2 — failure injection (observe only, no fixes)
$ErrorActionPreference = "Continue"
Set-Location $PSScriptRoot\..

$API = "http://127.0.0.1:59245"
$SESSION = "83998a9b-45ba-49aa-c178-eb39dcc43b6a"
$PROJECT = "agent-blackbox"
$results = @{}

function Get-Integrity {
    (Invoke-WebRequest -Uri "$API/debug/integrity" -UseBasicParsing).Content | ConvertFrom-Json
}

function Get-QueueCount {
    if (Test-Path .upmarto\queue.jsonl) {
        (Get-Content .upmarto\queue.jsonl | Where-Object { $_.Trim() }).Count
    } else { 0 }
}

# ========== SCENARIO 1: Backend Down ==========
Write-Host "`n=== SCENARIO 1: Backend Down ==="
$before1 = Get-Integrity
$qBefore1 = Get-QueueCount
$env:UPMARTO_URL = "http://127.0.0.1:1"
$s1Errors = 0
for ($i = 1; $i -le 5; $i++) {
    $out = & cargo run -q -p upmarto-cli -- track --type file_opened --path "chaos/s1_open_$i.rs" 2>&1 | Out-String
    if ($out -match "Error") { $s1Errors++ }
}
for ($i = 1; $i -le 5; $i++) {
    $out = & cargo run -q -p upmarto-cli -- track --type file_modified --path "chaos/s1_mod_$i.rs" 2>&1 | Out-String
    if ($out -match "Error") { $s1Errors++ }
}
for ($i = 1; $i -le 2; $i++) {
    $out = & cargo run -q -p upmarto-cli -- track --type test_failed --test "chaos_test_$i" --error "injected failure" 2>&1 | Out-String
    if ($out -match "Error") { $s1Errors++ }
}
$qDuring1 = Get-QueueCount
$env:UPMARTO_URL = $API
$flush1 = & cargo run -q -p upmarto-cli -- flush 2>&1 | Out-String
$qAfter1 = Get-QueueCount
$after1 = Get-Integrity
$delivered1 = $after1.jsonl_line_count - $before1.jsonl_line_count
$results["s1"] = @{
    errors = $s1Errors
    queue_during = $qDuring1
    queue_after = $qAfter1
    delivered = $delivered1
    flush = $flush1.Trim()
}

# ========== SCENARIO 2: Dual Client Race ==========
Write-Host "`n=== SCENARIO 2: Dual Client Race ==="
$ws = (Get-Location).Path
$raceStart = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
$env:UPMARTO_URL = $API
$env:CURSOR_PROJECT_DIR = $ws

$raceJobs = @()
# CLI batch (10 events)
$raceJobs += Start-Job -ScriptBlock {
    Set-Location $using:ws
    $env:UPMARTO_URL = $using:API
    for ($i = 1; $i -le 3; $i++) { & cargo run -q -p upmarto-cli -- track --type test_run --test "race_cli_$i" 2>&1 | Out-Null }
    for ($i = 1; $i -le 3; $i++) { & cargo run -q -p upmarto-cli -- track --type test_failed --test "race_cli_$i" --error "race" 2>&1 | Out-Null }
    & cargo run -q -p upmarto-cli -- track --type git_commit --message "race: cli commit" 2>&1 | Out-Null
    for ($i = 1; $i -le 3; $i++) { & cargo run -q -p upmarto-cli -- track --type git_commit --message "race: cli commit $i" 2>&1 | Out-Null }
}

# Cursor batch (10 events via hooks)
$raceJobs += Start-Job -ScriptBlock {
    Set-Location $using:ws
    $env:UPMARTO_URL = $using:API
    $env:CURSOR_PROJECT_DIR = $using:ws
    for ($i = 1; $i -le 5; $i++) {
        "{`"path`":`"$($using:ws)/src/race_cursor_$i.rs`",`"tool`":`"StrReplace`"}" | node upmarto-cursor/dist/hook.js afterFileEdit 2>&1 | Out-Null
    }
    for ($i = 1; $i -le 5; $i++) {
        "{`"response`":`"race agent message $i`"}" | node upmarto-cursor/dist/hook.js afterAgentResponse 2>&1 | Out-Null
    }
}

# VSCode-simulated batch (10 events via CLI with vscode source in path namespace)
$raceJobs += Start-Job -ScriptBlock {
    Set-Location $using:ws
    $env:UPMARTO_URL = $using:API
    for ($i = 1; $i -le 5; $i++) { & cargo run -q -p upmarto-cli -- track --type file_opened --path "vscode_sim/race_open_$i.rs" 2>&1 | Out-Null }
    for ($i = 1; $i -le 5; $i++) { & cargo run -q -p upmarto-cli -- track --type file_modified --path "vscode_sim/race_mod_$i.rs" 2>&1 | Out-Null }
}

$raceJobs | Wait-Job | Out-Null
$raceJobs | Remove-Job
& cargo run -q -p upmarto-cli -- flush 2>&1 | Out-Null
Start-Sleep -Milliseconds 1500
& cargo run -q -p upmarto-cli -- flush 2>&1 | Out-Null

$timeline = (Invoke-WebRequest -Uri "$API/timeline?session_id=$SESSION" -UseBasicParsing).Content | ConvertFrom-Json
$raceEvents = @($timeline.events | Where-Object { $_.timestamp -ge $raceStart })
$dupes = $raceEvents | Group-Object { "$($_.event_type)|$($_.timestamp)|$($_.payload.path)|$($_.payload.test)" } | Where-Object { $_.Count -gt 1 }
$wrongSession = @($raceEvents | Where-Object { $_.session_id -ne $SESSION })
$ts = @($raceEvents | ForEach-Object { $_.timestamp })
$sorted = $ts | Sort-Object
$ordered = ($ts -join ',') -eq ($sorted -join ',')
$results["s2"] = @{
    new_events = $raceEvents.Count
    duplicates = $dupes.Count
    wrong_session = $wrongSession.Count
    ordered = $ordered
}

# ========== SCENARIO 3: Session Drift Attack ==========
Write-Host "`n=== SCENARIO 3: Session Drift ==="
$tsFirst = node -e "import { resolveSessionId } from './upmarto-sdk-ts/dist/session.js'; console.log(resolveSessionId(process.cwd()));"
Start-Sleep -Seconds 3
$rustAfter = (& cargo run -q -p upmarto-cli -- session 2>&1 | Select-String "session_id:").ToString() -replace "session_id:\s*",""
$results["s3"] = @{
    ts_first = $tsFirst.Trim()
    rust_after_3s = $rustAfter.Trim()
    match = ($tsFirst.Trim() -eq $rustAfter.Trim())
}

# ========== SCENARIO 4: Queue Corruption ==========
Write-Host "`n=== SCENARIO 4: Queue Corruption ==="
$qBackup = Get-Content .upmarto\queue.jsonl -Raw -ErrorAction SilentlyContinue
$corruptLines = @(
    '{"project_id":"agent-blackbox","session_id":"83998a9b-45ba-49aa-c178-eb39dcc43b6a","event_type":"file_modified","timestamp":99990001,"payload":{"path":"corrupt/inject_1.rs","chaos":true},"_attempts":99}',
    '{"project_id":"agent-blackbox","session_id":"83998a9b-45ba-49aa-c178-eb39dcc43b6a","event_type":"test_failed","timestamp":99990002,"payload":{"test":"corrupt_test","error":"injected"},"_attempts":42}',
    '{"project_id":"agent-blackbox","session_id":"83998a9b-45ba-49aa-c178-eb39dcc43b6a","event_type":"agent_message","timestamp":99990003,"payload":{"message":"corrupt queue"},"_attempts":7}'
)
Add-Content .upmarto\queue.jsonl ($corruptLines -join "`n") -Encoding utf8
$qAfterInject = Get-QueueCount

# Simulate SDK restart + flush via node
$nodeFlush = node -e @"
import { Upmarto } from './upmarto-sdk-ts/dist/client.js';
process.env.UPMARTO_URL = '$API';
const c = Upmarto.fromWorkspace(process.cwd());
await c.flush();
console.log('flushed');
"@ 2>&1 | Out-String

$qAfterFlush4 = Get-QueueCount
$integrity4 = Get-Integrity
# Check if corrupt events landed in backend
$tl4 = (Invoke-WebRequest -Uri "$API/timeline?session_id=$SESSION" -UseBasicParsing).Content | ConvertFrom-Json
$corruptInTimeline = @($tl4.events | Where-Object { $_.payload.chaos -eq $true -or $_.payload.test -eq 'corrupt_test' })
$results["s4"] = @{
    queue_after_inject = $qAfterInject
    queue_after_flush = $qAfterFlush4
    corrupt_in_timeline = $corruptInTimeline.Count
    node_flush = $nodeFlush.Trim()
}

# ========== SCENARIO 5: Explain Engine Stress ==========
Write-Host "`n=== SCENARIO 5: Explain Stress ==="
$explainSession = "chaos-explain-$(Get-Date -Format 'yyyyMMddHHmmss')"
$explainEvents = @(
    @{ type = "test_failed"; payload = @{ test = "payment_gateway"; error = "timeout" } },
    @{ type = "test_failed"; payload = @{ test = "auth_token"; error = "expired" } },
    @{ type = "test_failed"; payload = @{ test = "db_migration"; error = "schema mismatch" } },
    @{ type = "file_modified"; payload = @{ path = "src/a.rs" } },
    @{ type = "file_modified"; payload = @{ path = "src/b.rs" } },
    @{ type = "file_modified"; payload = @{ path = "src/c.rs" } },
    @{ type = "file_modified"; payload = @{ path = "src/d.rs" } },
    @{ type = "file_modified"; payload = @{ path = "src/e.rs" } },
    @{ type = "file_modified"; payload = @{ path = "src/f.rs" } },
    @{ type = "file_modified"; payload = @{ path = "src/g.rs" } },
    @{ type = "file_modified"; payload = @{ path = "src/h.rs" } },
    @{ type = "git_commit"; payload = @{ message = "fix: partial" } },
    @{ type = "git_commit"; payload = @{ message = "fix: retry logic" } },
    @{ type = "agent_message"; payload = @{ message = "investigating failures" } },
    @{ type = "agent_message"; payload = @{ message = "applied workaround" } }
)
$ts5 = 1700010000000
foreach ($ev in $explainEvents) {
    $body = @{
        project_id = $PROJECT
        session_id = $explainSession
        event_type = $ev.type
        timestamp = $ts5
        payload = $ev.payload
    } | ConvertTo-Json -Compress -Depth 5
    Invoke-WebRequest -Uri "$API/event" -Method POST -Body $body -ContentType "application/json" -UseBasicParsing | Out-Null
    $ts5 += 100
}

$explainBody = (@{ session_id = $explainSession } | ConvertTo-Json)
$explainRes = (Invoke-WebRequest -Uri "$API/explain" -Method POST -Body $explainBody -ContentType "application/json" -UseBasicParsing).Content | ConvertFrom-Json
$results["s5"] = @{
    session = $explainSession
    event_count = $explainEvents.Count
    root_cause = $explainRes.root_cause
    problem = $explainRes.problem_statement
    summary = $explainRes.summary
}

# Output JSON results
$results | ConvertTo-Json -Depth 5
