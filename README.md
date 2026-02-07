# Roomler2

Real-time communication and collaboration platform with multi-tenancy, channels, chat, video conferencing, file sharing, cloud storage integrations, and AI-powered document recognition.

## Features

- **Multi-Tenancy** -- Organizations with plans (Free / Pro / Business / Enterprise), roles, and permissions
- **Channels** -- Hierarchical channel tree with 8 types: Category, Text, Voice, Announcement, Forum, Stage, DM, GroupDM
- **Real-Time Chat** -- Threaded messages, reactions (unicode + custom emoji), mentions, embeds, attachments, pinning
- **Video Conferencing** -- Instant / Scheduled / Recurring / Persistent meetings, recordings, transcriptions
- **File Management** -- Versioned uploads, cloud sync (Google Drive, OneDrive, Dropbox), AI document recognition via Claude API
- **Export** -- Conversation export to XLSX and PDF
- **WebSocket** -- Live presence, typing indicators, real-time message delivery

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Backend | Rust, Axum 0.8, Tokio |
| Frontend | Vue 3, Vuetify 3, Pinia, Vue Router, vue-i18n |
| Database | MongoDB 7 |
| Cache / Pub-Sub | Redis 7 |
| Object Storage | MinIO (S3-compatible) |
| TURN Server | Coturn |
| AI | Claude API (document recognition) |
| Auth | JWT (argon2 password hashing, httpOnly cookies) |
| Testing | Rust integration tests, Playwright E2E |

## Quick Start

### Prerequisites

- Rust (edition 2024)
- Node.js
- Docker & Docker Compose

### 1. Start infrastructure

```bash
docker-compose up -d
```

This starts MongoDB, Redis, MinIO, and Coturn.

### 2. Configure environment

```bash
cp .env.example .env
# Edit .env with your settings
```

### 3. Run backend

```bash
cargo build
cargo run
```

The API server starts on `http://localhost:3000`.

### 4. Run frontend

```bash
cd ui
npm install
npm run dev
```

The dev server starts on `http://localhost:5173`.

## Project Structure

```
roomler2/
├── Cargo.toml              # Workspace root
├── docker-compose.yml      # Infrastructure services
├── .env.example            # Environment variable template
├── crates/
│   ├── config/             # Configuration loading (config files + env vars)
│   ├── db/                 # MongoDB models, DAOs, indexes
│   │   └── src/models/     # 18 data models
│   ├── services/           # Business logic layer
│   │   └── src/
│   │       ├── auth/       # JWT, password hashing
│   │       ├── dao/        # Data access objects
│   │       ├── export/     # XLSX and PDF export
│   │       ├── cloud_storage/  # S3/MinIO operations
│   │       ├── background/ # Async task processing
│   │       └── media/      # Media handling
│   ├── api/                # Axum HTTP + WebSocket layer
│   │   └── src/
│   │       ├── routes/     # REST endpoint handlers
│   │       ├── ws/         # WebSocket handler, storage, dispatcher
│   │       ├── middleware/  # Auth middleware
│   │       └── extractors/ # Request extractors
│   └── tests/              # Integration test suite
│       └── src/
│           ├── fixtures/   # test_app.rs, seed.rs
│           └── *_tests.rs  # 12 test modules
└── ui/                     # Vue 3 SPA
    ├── src/
    │   ├── views/          # Page components
    │   ├── components/     # Reusable UI components
    │   ├── stores/         # 8 Pinia stores
    │   ├── composables/    # useAuth, useWebSocket
    │   ├── api/            # HTTP/WS client
    │   ├── plugins/        # Router, Vuetify, i18n
    │   └── locales/        # i18n translations
    └── e2e/                # 6 Playwright spec files
```

## Documentation

| Document | Description |
|----------|-------------|
| [Architecture](docs/architecture.md) | System design, crate graph, request flow |
| [Data Model](docs/data-model.md) | All 18 entities, ER diagram, indexes |
| [API Reference](docs/api.md) | REST endpoints, request/response schemas |
| [Frontend](docs/ui.md) | Routes, components, Pinia stores |
| [Use Cases](docs/use-cases.md) | User flows, permissions, lifecycle diagrams |
| [Real-Time](docs/real-time.md) | WebSocket protocol, presence, typing indicators |
| [Testing](docs/testing.md) | Integration tests, E2E tests, fixtures |
| [Deployment](docs/deployment.md) | Docker Compose, environment variables, config |

## License

MIT
