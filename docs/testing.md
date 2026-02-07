# Testing

Roomler2 has two test layers: Rust integration tests and Playwright E2E tests.

## Integration Tests

Located in `crates/tests/src/`. These tests spin up the full Axum server and interact with it via HTTP using `reqwest`.

### Test Modules

| File | Coverage Area |
|------|--------------|
| `auth_tests.rs` | Registration, login, logout, refresh, /me |
| `channel_tests.rs` | Channel join, leave, list, explore |
| `channel_crud_tests.rs` | Channel create, update, delete |
| `message_tests.rs` | Send, edit, delete, list, pin, threads |
| `reaction_tests.rs` | Add and remove reactions |
| `conference_tests.rs` | Create, start, join, leave, end conferences |
| `recording_tests.rs` | Create, list, delete recordings |
| `transcription_tests.rs` | Create, list, get transcriptions |
| `file_tests.rs` | Upload, get, download, delete, list files |
| `export_tests.rs` | Conversation export to XLSX |
| `pdf_export_tests.rs` | Conversation export to PDF |
| `multi_tenancy_tests.rs` | Cross-tenant data isolation |

### Test Fixtures

| File | Purpose |
|------|---------|
| `fixtures/test_app.rs` | Starts a test server on a random port, provides a configured `reqwest::Client` |
| `fixtures/seed.rs` | Creates test users, tenants, channels, and messages for test setup |

### Running Integration Tests

```bash
# Run all integration tests
cargo test -p tests

# Run with race detection (recommended)
cargo test -p tests -- --test-threads=1

# Run a specific test module
cargo test -p tests auth_tests

# Run with output
cargo test -p tests -- --nocapture
```

Integration tests require a running MongoDB instance (see `docker-compose.yml`).

## E2E Tests

Located in `ui/e2e/`. Playwright tests that run against the full stack (backend + frontend).

### Test Specs

| File | Coverage Area |
|------|--------------|
| `auth.spec.ts` | Login and registration flows |
| `dashboard.spec.ts` | Dashboard rendering, tenant cards |
| `channels.spec.ts` | Channel creation, browsing |
| `chat.spec.ts` | Message sending, display |
| `conference.spec.ts` | Conference view, join button |
| `files.spec.ts` | File upload, browsing |

### Test Helpers

| File | Purpose |
|------|---------|
| `fixtures/test-helpers.ts` | Login helper, page setup, API utilities |

### Running E2E Tests

```bash
cd ui

# Run all E2E tests
npm run e2e

# Run with Playwright UI
npm run e2e:ui

# Run a specific spec
npx playwright test e2e/auth.spec.ts

# Run in headed mode
npx playwright test --headed
```

E2E tests require both the backend (`cargo run`) and frontend (`npm run dev`) to be running.

## Test Architecture

```
Integration Tests (Rust)              E2E Tests (Playwright)
┌─────────────────────┐              ┌─────────────────────┐
│ reqwest HTTP Client  │              │ Chromium Browser     │
│         │            │              │        │             │
│         ▼            │              │        ▼             │
│  Axum Test Server    │              │  Vue 3 SPA (:5173)  │
│  (random port)       │              │        │             │
│         │            │              │        ▼             │
│         ▼            │              │  Axum API (:3000)    │
│  MongoDB (test DB)   │              │        │             │
│                      │              │        ▼             │
└─────────────────────┘              │  MongoDB + Redis +   │
                                     │  MinIO               │
                                     └─────────────────────┘
```

Integration tests use an isolated test database and server instance per test. E2E tests run against the development stack.
