# Upmarto — Kullanıcı Kurulum ve Kullanım Rehberi

**Upmarto**, AI kodlama oturumlarını yerel makinede kaydeder, timeline olarak oynatır ve deterministik **“neden?”** analizi üretir. Veriler buluta zorunlu değildir; tüm capture `127.0.0.1` üzerindeki backend’e gider.

---

## İçindekiler

1. [Mimari ve bileşenler](#1-mimari-ve-bileşenler)
2. [Ön koşullar](#2-ön-koşullar)
3. [Backend kurulumu](#3-backend-kurulumu-tüm-platformlar-için-zorunlu)
4. [Proje yapılandırması (`init`)](#4-proje-yapılandırması-init)
5. [CLI kurulumu ve kullanımı](#5-cli-kurulumu-ve-kullanımı)
6. [VS Code extension](#6-vs-code-extension)
7. [Cursor hooks](#7-cursor-hooks)
8. [TypeScript SDK](#8-typescript-sdk)
9. [Rust SDK](#9-rust-sdk)
10. [Web UI (timeline + explain)](#10-web-ui-timeline--explain)
11. [Docker ile çalıştırma](#11-docker-ile-çalıştırma)
12. [Yapılandırma referansı](#12-yapılandırma-referansı)
13. [Olay tipleri ve analiz](#13-olay-tipleri-ve-analiz)
14. [Sorun giderme](#14-sorun-giderme)
15. [Yayınlanmış paket linkleri](#15-yayınlanmış-paket-linkleri)

---

## 1. Mimari ve bileşenler

```
┌──────────────────────────────────────────────────────────────┐
│  BACKEND (upmarto-server) — yerel HTTP API                   │
│  POST /event  ·  GET /timeline  ·  POST /explain  ·  GET /config │
│  Depolama: data/events.log + data/metadata.db                │
└────────────────────────────┬─────────────────────────────────┘
                             │
     ┌───────────────────────┼───────────────────────┐
     ▼                       ▼                       ▼
 VS Code ext            Cursor hooks              CLI
 (@upmarto/sdk)         (@upmarto/cursor)    (upmarto-cli)
     │                       │                       │
     └───────────────────────┴───────────────────────┘
                    .upmarto/config.json
                    .upmarto/queue.jsonl (offline kuyruk)
```

| Bileşen | Registry / kaynak | Rol |
|---------|-------------------|-----|
| **Backend** | GitHub repo veya Docker | Olayları saklar, timeline ve explain üretir |
| **upmarto-cli** | [crates.io/upmarto-cli](https://crates.io/crates/upmarto-cli) | `init`, `explain`, `workflow`, manuel `track` |
| **@upmarto/sdk** | [npm/@upmarto/sdk](https://www.npmjs.com/package/@upmarto/sdk) | TS/Node istemci, kuyruk, retry |
| **upmarto-sdk** | [crates.io/upmarto-sdk](https://crates.io/crates/upmarto-sdk) | Rust istemci |
| **@upmarto/cursor** | [npm/@upmarto/cursor](https://www.npmjs.com/package/@upmarto/cursor) | Cursor IDE hook’ları |
| **upmarto-vscode** | [VS Marketplace](https://marketplace.visualstudio.com/items?itemName=upmarto.upmarto-vscode) | VS Code pasif capture |
| **Web UI** | Repo `agent-blackbox/ui/` | Tarayıcıda session / timeline / explain |

> **Kritik:** Extension veya Cursor hooks **tek başına çalışmaz**. Önce backend açık olmalı, sonra proje `init` edilmelidir.

---

## 2. Ön koşullar

### Tüm kullanıcılar

| Araç | Minimum sürüm | Neden |
|------|---------------|-------|
| **Rust toolchain** | stable | Backend çalıştırmak için (veya Docker) |
| **Git** | herhangi | Repo klonlama |

### Platforma göre ek gereksinimler

| Platform | VS Code | Cursor | CLI | SDK (TS) |
|----------|---------|--------|-----|----------|
| **Windows** | VS Code 1.85+ | Cursor (güncel) | `cargo install` | Node 18+ |
| **macOS** | VS Code 1.85+ | Cursor | `cargo install` | Node 18+ |
| **Linux** | VS Code 1.85+ | Cursor | `cargo install` | Node 18+ |

---

## 3. Backend kurulumu (tüm platformlar için zorunlu)

Backend crates.io’da yayımlı değildir. İki yol:

### Yol A — Kaynak koddan (önerilen, geliştirme)

```bash
git clone https://github.com/mertcaliskanlinux/upmarto.git
cd upmarto/agent-blackbox
```

**Windows (PowerShell), macOS, Linux:**

```bash
cargo run
```

Başlangıç logunda şunu görürsünüz:

```
[Upmarto] listening on http://127.0.0.1:54321
API base URL: http://127.0.0.1:54321
```

**Bu URL’yi kopyalayın.** `APP_PORT=0` olduğunda port her çalıştırmada değişebilir; sabit port varsaymayın.

#### Opsiyonel `.env`

```bash
cp .env.example .env
```

| Değişken | Varsayılan | Açıklama |
|----------|------------|----------|
| `APP_HOST` | `0.0.0.0` | Dinleme adresi (`127.0.0.1` = sadece local) |
| `APP_PORT` | `0` | `0` = OS boş port atar |
| `DATABASE_PATH` | `./data/events.log` | JSONL olay günlüğü |
| `SQLITE_PATH` | `./data/metadata.db` | SQLite indeks |
| `PUBLIC_BASE_URL` | _(otomatik)_ | Reverse proxy arkasında dış URL |
| `TEST_MODE` | `false` | İzole test depolama |

Backend doğrulama:

```bash
curl http://127.0.0.1:PORT/config
```

### Yol B — Docker

```bash
cd agent-blackbox
docker compose up --build
```

`docker-compose.yml` varsayılanı: **http://127.0.0.1:8080**

Kalıcılık için `upmarto-data` volume kullanılır. Dış istemciler için `PUBLIC_BASE_URL` ayarlayın.

---

## 4. Proje yapılandırması (`init`)

Her çalışma alanı (proje klasörü) için bir kez:

### CLI ile (önerilen)

```bash
cargo install upmarto-cli
cd /path/to/your-project
upmarto-cli init --api-url http://127.0.0.1:54321
```

`--api-url` verilmezse CLI backend’i otomatik keşfetmeye çalışır (`GET /config`).

### Oluşturulan dosyalar

```
your-project/
└── .upmarto/
    ├── config.json      # API URL, batch, retry
    ├── runtime.json     # init sırasında yazılır
    ├── active_session   # CLI session (workflow/explain)
    └── queue.jsonl      # offline kuyruk (gerekirse)
```

Örnek `config.json`:

```json
{
  "api_url": "http://127.0.0.1:54321",
  "project_id": "auto",
  "auto_capture": true,
  "batch_size": 50,
  "flush_interval_ms": 2000,
  "retry_max": 5
}
```

Ortam değişkeni `UPMARTO_URL` — `api_url` üzerine yazar.

---

## 5. CLI kurulumu ve kullanımı

### Kurulum

```bash
cargo install upmarto-cli
```

Yüklenen binary: **`upmarto-cli`** (isterseniz `upmarto` alias’ı tanımlayın).

### Komutlar

| Komut | Açıklama |
|-------|----------|
| `upmarto-cli init [--api-url URL]` | `.upmarto/config.json` oluşturur |
| `upmarto-cli workflow` | İzole 6 olaylı demo senaryo (`wf-*` session) |
| `upmarto-cli demo` | Kısa 4 olaylı demo |
| `upmarto-cli explain [session_id]` | WHY engine — session yoksa aktif session |
| `upmarto-cli session` | Aktif session ID |
| `upmarto-cli track --type TYPE ...` | Tek olay gönder |
| `upmarto-cli flush` | Bekleyen kuyruğu gönder |

Global flag: `--workspace /path` (varsayılan: cwd)

### Örnek akış

```bash
# Terminal 1 — backend çalışıyor olmalı
cd my-app
upmarto-cli init --api-url http://127.0.0.1:54321
upmarto-cli workflow
upmarto-cli explain
```

`explain` çıktısı: `summary`, `root_cause`, `problem_statement`, `resolution_flow`, `decision_chain` — **LLM yok**, deterministik.

### `track` örnekleri

**Bash / macOS / Linux:**

```bash
upmarto-cli track --type file_modified --path src/main.rs
upmarto-cli track --type test_failed --test auth_test --error "token expired"
upmarto-cli track --type command_executed --command "cargo test"
```

**Windows PowerShell** (JSON kaçış sorunları için kısayol flag’leri):

```powershell
upmarto-cli track --type file_modified --path src/main.rs
upmarto-cli track --type test_failed --test auth_test --error "token expired"
```

---

## 6. VS Code extension

### Kurulum

**Marketplace:**

1. VS Code → Extensions (`Ctrl+Shift+X`)
2. Ara: **Upmarto** veya `upmarto.upmarto-vscode`
3. Install

**Komut satırı:**

```bash
code --install-extension upmarto.upmarto-vscode
```

**VSIX (offline):**

GitHub Release’ten `upmarto-vscode-*.vsix` indir → *Install from VSIX…*

### Kurulum sonrası

1. Backend çalışıyor olmalı
2. Workspace’te `upmarto-cli init` yapılmış olmalı
3. Klasörü VS Code’da açın (multi-root desteklenir)

Extension otomatik capture başlatır. Output kanalı: **Upmarto**.

### VS Code ayarı

| Ayar | Varsayılan | Açıklama |
|------|------------|----------|
| `upmarto.enabled` | `true` | Pasif capture aç/kapa |

API URL ve batch ayarları `.upmarto/config.json` üzerinden (`@upmarto/sdk`).

### Yakalanan olaylar

| Aktivite | `event_type` |
|----------|--------------|
| Dosya açma | `file_opened` |
| Düzenleme / kaydetme | `file_modified` |
| Yeni dosya | `file_created` |
| Task / terminal | `command_executed` |
| Test task’ları | `test_run`, `test_passed`, `test_failed` |
| Extension aktivasyonu | `agent_message` |

### Doğrulama

1. Bir dosyayı düzenleyip kaydedin
2. `upmarto-cli explain` veya Web UI timeline kontrol edin

---

## 7. Cursor hooks

### Kurulum

Proje kökünde:

```bash
npm install @upmarto/cursor
```

Ön koşul: `upmarto-cli init` + backend çalışır durumda.

### `hooks.json` yapılandırması

`.cursor/hooks.json` oluşturun. **npm kurulumu sonrası yollar `node_modules` altına işaret etmelidir:**

```json
{
  "version": 1,
  "hooks": {
    "sessionStart": [
      { "command": "node node_modules/@upmarto/cursor/dist/hook.js sessionStart" }
    ],
    "beforeReadFile": [
      { "command": "node node_modules/@upmarto/cursor/dist/hook.js beforeReadFile" }
    ],
    "afterFileEdit": [
      { "command": "node node_modules/@upmarto/cursor/dist/hook.js afterFileEdit" }
    ],
    "beforeShellExecution": [
      { "command": "node node_modules/@upmarto/cursor/dist/hook.js beforeShellExecution" }
    ],
    "afterShellExecution": [
      { "command": "node node_modules/@upmarto/cursor/dist/hook.js afterShellExecution" }
    ],
    "afterAgentResponse": [
      { "command": "node node_modules/@upmarto/cursor/dist/hook.js afterAgentResponse" }
    ],
    "postToolUse": [
      {
        "command": "node node_modules/@upmarto/cursor/dist/hook.js postToolUse",
        "matcher": "Write|StrReplace"
      }
    ]
  }
}
```

**Windows:** `node` PATH’te olmalı; yol ayırıcıları yukarıdaki gibi `/` kullanılabilir.

### Son adım

**Cursor’u tamamen kapatıp yeniden açın.**

### Yakalanan olaylar

Dosya okuma/düzenleme, shell komutları, agent yanıtları, araç kullanımı → v1 `event_type` değerleri.

### Global kurulum (opsiyonel)

```bash
npm install -g @upmarto/cursor
```

`hooks.json` içinde: `node $(npm root -g)/@upmarto/cursor/dist/hook.js ...` veya tam global yol.

---

## 8. TypeScript SDK

### Kurulum

```bash
npm install @upmarto/sdk
```

### Kullanım

```typescript
import { Upmarto } from "@upmarto/sdk";

const client = await Upmarto.fromWorkspace();
client.track({
  event_type: "file_modified",
  payload: { path: "src/main.rs" },
});
await client.flush();
```

### Özellikler

- ESM + TypeScript tipleri
- `.upmarto/config.json` okuma
- Offline kuyruk: `.upmarto/queue.jsonl`
- Batch: 50 olay veya 2 sn flush
- Exponential backoff, max 5 retry

---

## 9. Rust SDK

### Kurulum

```bash
cargo add upmarto-sdk
```

### Kullanım

```rust
use upmarto_sdk::{EventType, TrackEvent, Upmarto};
use serde_json::json;

#[tokio::main]
async fn main() -> upmarto_sdk::Result<()> {
    let client = Upmarto::from_workspace(".").await?;
    client.session("my-session").await;
    client.track(TrackEvent {
        event_type: EventType::FileModified,
        payload: json!({ "path": "src/main.rs" }),
        timestamp: None,
    }).await?;
    client.flush().await?;
    Ok(())
}
```

---

## 10. Web UI (timeline + explain)

### Geliştirme modu

**Terminal 1 — backend:**

```bash
cd agent-blackbox
cargo run
# API base URL'yi not edin
```

**Terminal 2 — UI:**

```bash
cd agent-blackbox/ui
npm install
cp .env.example .env
# .env içinde:
# VITE_API_PROXY_TARGET=http://127.0.0.1:ACTUAL_PORT
npm run dev
```

Vite’ın yazdığı URL’yi tarayıcıda açın.

### Rotalar

| Rota | API |
|------|-----|
| `/sessions` | `GET /project/:id/sessions` |
| `/timeline/:session_id` | `GET /timeline` |
| `/explain/:session_id` | `POST /explain` |

### Production build

```bash
VITE_API_BASE_URL=https://your-host npm run build
```

---

## 11. Docker ile çalıştırma

```bash
cd agent-blackbox
docker compose up --build
```

| Parametre | Değer |
|-----------|-------|
| Host port | `8080` (varsayılan) |
| Init URL | `http://127.0.0.1:8080` |

```bash
upmarto-cli init --api-url http://127.0.0.1:8080
```

---

## 12. Yapılandırma referansı

### Öncelik sırası (API URL)

1. `UPMARTO_URL` ortam değişkeni
2. Proje `.upmarto/config.json`
3. Global `~/.upmarto/config.json`

### Session modeli

- **Günlük session:** SDK otomatik türetir
- **CLI workflow session:** `wf-{timestamp}` — `workflow` komutu yazar
- **explain:** `active_session` dosyası varsa onu kullanır

### Offline davranış

Backend kapalıyken olaylar `.upmarto/queue.jsonl` dosyasına yazılır. Backend açılınca `flush` veya otomatik retry ile gönderilir.

---

## 13. Olay tipleri ve analiz

### v1 frozen event types

```
file_opened | file_modified | file_created | command_executed
test_run | test_failed | test_passed | git_commit | agent_message
```

### API ile timeline

```bash
curl "http://127.0.0.1:PORT/timeline?session_id=SESSION_ID"
```

### API ile explain

```bash
curl -X POST http://127.0.0.1:PORT/explain \
  -H "Content-Type: application/json" \
  -d '{"session_id": "SESSION_ID"}'
```

---

## 14. Sorun giderme

| Belirti | Olası neden | Çözüm |
|---------|-------------|-------|
| Olay gitmiyor | Backend kapalı | `cargo run` veya `docker compose up` |
| `init` başarısız | Yanlış port | `cargo run` logundaki URL’yi kullanın |
| Extension sessiz | Workspace yok / init yok | Klasör açın + `upmarto-cli init` |
| Cursor hook çalışmıyor | Yanlış `hooks.json` yolu | `node_modules/@upmarto/cursor/dist/hook.js` |
| `explain` boş / kısa | Session’da olay yok | `workflow` çalıştırın veya IDE’de işlem yapın |
| PowerShell JSON hatası | `--payload` kaçışı | `--path`, `--test`, `--command` kullanın |
| Kuyruk birikiyor | Backend uzun süre kapalı | Backend açın → `upmarto-cli flush` |

---

## 15. Yayınlanmış paket linkleri

| Paket | Link | Kurulum |
|-------|------|---------|
| VS Code | [upmarto.upmarto-vscode](https://marketplace.visualstudio.com/items?itemName=upmarto.upmarto-vscode) | Marketplace |
| Cursor hooks | [@upmarto/cursor](https://www.npmjs.com/package/@upmarto/cursor) | `npm i @upmarto/cursor` |
| TS SDK | [@upmarto/sdk](https://www.npmjs.com/package/@upmarto/sdk) | `npm i @upmarto/sdk` |
| CLI | [upmarto-cli](https://crates.io/crates/upmarto-cli) | `cargo install upmarto-cli` |
| Rust SDK | [upmarto-sdk](https://crates.io/crates/upmarto-sdk) | `cargo add upmarto-sdk` |
| Backend | [GitHub](https://github.com/mertcaliskanlinux/upmarto) | `git clone` + `cargo run` |

---

## Hızlı başlangıç özeti (5 dakika)

```bash
# 1. Backend
git clone https://github.com/mertcaliskanlinux/upmarto.git
cd upmarto/agent-blackbox && cargo run
# → API URL'yi kopyala

# 2. CLI + proje
cargo install upmarto-cli
cd ~/my-project
upmarto-cli init --api-url http://127.0.0.1:PORT

# 3a. VS Code → Marketplace'ten "Upmarto" kur, workspace aç
# 3b. Cursor → npm i @upmarto/cursor, .cursor/hooks.json, restart

# 4. Analiz
upmarto-cli workflow    # demo (opsiyonel)
upmarto-cli explain
```

Daha fazla teknik detay: [SDK.md](./SDK.md) · [API_CONTRACT.md](./API_CONTRACT.md) · [DEPLOYMENT.md](./DEPLOYMENT.md)
