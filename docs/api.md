# API Reference

Base URL: `http://localhost:3000`

All API routes are nested under `/api`. Authentication is via JWT in an httpOnly cookie (`access_token`) or an `Authorization: Bearer <token>` header.

## Auth Routes

No tenant prefix. No authentication required for register/login.

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/api/auth/register` | No | Register a new user |
| POST | `/api/auth/login` | No | Login by username or email |
| POST | `/api/auth/logout` | No | Clear auth cookie |
| POST | `/api/auth/refresh` | No | Refresh access token |
| GET | `/api/auth/me` | Yes | Get current user profile |
| PUT | `/api/auth/me` | Yes | Update current user profile |

### POST `/api/auth/register`

```json
// Request
{
  "email": "user@example.com",
  "username": "user1",
  "display_name": "User One",
  "password": "secret",
  "tenant_name": "My Org",      // optional: creates a default tenant
  "tenant_slug": "my-org"       // optional: required if tenant_name is set
}

// Response (201 Created)
{
  "access_token": "eyJ...",
  "refresh_token": "eyJ...",
  "expires_in": 3600,
  "user": {
    "id": "6...",
    "email": "user@example.com",
    "username": "user1",
    "display_name": "User One",
    "avatar": null
  }
}
```

Sets httpOnly cookie: `access_token=<JWT>; HttpOnly; Path=/; SameSite=Lax`

### POST `/api/auth/login`

```json
// Request (either username or email required)
{
  "username": "user1",
  "password": "secret"
}

// Response (200 OK) — same shape as register
```

### POST `/api/auth/refresh`

```json
// Request
{
  "refresh_token": "eyJ..."
}

// Response (200 OK) — same shape as register
```

## Tenant Routes

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/api/tenant` | Yes | List tenants for current user |
| POST | `/api/tenant` | Yes | Create a new tenant |
| GET | `/api/tenant/{tenant_id}` | Yes | Get tenant details |

## Member Routes

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/api/tenant/{tenant_id}/member` | Yes | List members of a tenant |

## Channel Routes

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/api/tenant/{tenant_id}/channel` | Yes | List channels the user has joined |
| POST | `/api/tenant/{tenant_id}/channel` | Yes | Create a new channel |
| GET | `/api/tenant/{tenant_id}/channel/explore` | Yes | Browse all public channels |
| GET | `/api/tenant/{tenant_id}/channel/{channel_id}` | Yes | Get channel details |
| PUT | `/api/tenant/{tenant_id}/channel/{channel_id}` | Yes | Update a channel |
| DELETE | `/api/tenant/{tenant_id}/channel/{channel_id}` | Yes | Delete a channel |
| POST | `/api/tenant/{tenant_id}/channel/{channel_id}/join` | Yes | Join a channel |
| POST | `/api/tenant/{tenant_id}/channel/{channel_id}/leave` | Yes | Leave a channel |
| GET | `/api/tenant/{tenant_id}/channel/{channel_id}/member` | Yes | List channel members |

## Message Routes

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/api/tenant/{tenant_id}/channel/{channel_id}/message` | Yes | List messages (paginated) |
| POST | `/api/tenant/{tenant_id}/channel/{channel_id}/message` | Yes | Send a message |
| GET | `/api/tenant/{tenant_id}/channel/{channel_id}/message/pin` | Yes | List pinned messages |
| PUT | `/api/tenant/{tenant_id}/channel/{channel_id}/message/{message_id}` | Yes | Edit a message |
| DELETE | `/api/tenant/{tenant_id}/channel/{channel_id}/message/{message_id}` | Yes | Delete a message |
| PUT | `/api/tenant/{tenant_id}/channel/{channel_id}/message/{message_id}/pin` | Yes | Toggle pin on a message |
| GET | `/api/tenant/{tenant_id}/channel/{channel_id}/message/{message_id}/thread` | Yes | Get thread replies |
| POST | `/api/tenant/{tenant_id}/channel/{channel_id}/message/{message_id}/reaction` | Yes | Add a reaction |
| DELETE | `/api/tenant/{tenant_id}/channel/{channel_id}/message/{message_id}/reaction/{emoji}` | Yes | Remove a reaction |

## Conference Routes

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/api/tenant/{tenant_id}/conference` | Yes | List conferences |
| POST | `/api/tenant/{tenant_id}/conference` | Yes | Create a conference |
| GET | `/api/tenant/{tenant_id}/conference/{conference_id}` | Yes | Get conference details |
| POST | `/api/tenant/{tenant_id}/conference/{conference_id}/start` | Yes | Start a conference |
| POST | `/api/tenant/{tenant_id}/conference/{conference_id}/join` | Yes | Join a conference |
| POST | `/api/tenant/{tenant_id}/conference/{conference_id}/leave` | Yes | Leave a conference |
| POST | `/api/tenant/{tenant_id}/conference/{conference_id}/end` | Yes | End a conference |
| GET | `/api/tenant/{tenant_id}/conference/{conference_id}/participant` | Yes | List participants |

## Recording Routes

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/api/tenant/{tenant_id}/conference/{conference_id}/recording` | Yes | List recordings |
| POST | `/api/tenant/{tenant_id}/conference/{conference_id}/recording` | Yes | Create a recording |
| DELETE | `/api/tenant/{tenant_id}/conference/{conference_id}/recording/{recording_id}` | Yes | Delete a recording |

## Transcription Routes

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/api/tenant/{tenant_id}/conference/{conference_id}/transcript` | Yes | List transcriptions |
| POST | `/api/tenant/{tenant_id}/conference/{conference_id}/transcript` | Yes | Create a transcription |
| GET | `/api/tenant/{tenant_id}/conference/{conference_id}/transcript/{transcription_id}` | Yes | Get transcription details |

## File Routes

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/api/tenant/{tenant_id}/channel/file/upload` | Yes | Upload a file |
| GET | `/api/tenant/{tenant_id}/channel/file/{file_id}` | Yes | Get file metadata |
| GET | `/api/tenant/{tenant_id}/channel/file/{file_id}/download` | Yes | Download a file |
| DELETE | `/api/tenant/{tenant_id}/channel/file/{file_id}` | Yes | Delete a file |
| POST | `/api/tenant/{tenant_id}/channel/file/{file_id}/recognize` | Yes | AI document recognition (Claude API) |
| GET | `/api/tenant/{tenant_id}/channel/{channel_id}/file` | Yes | List files in a channel |

## Background Task Routes

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/api/tenant/{tenant_id}/task` | Yes | List background tasks |
| GET | `/api/tenant/{tenant_id}/task/{task_id}` | Yes | Get task status |
| GET | `/api/tenant/{tenant_id}/task/{task_id}/download` | Yes | Download task output file |

## Export Routes

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| POST | `/api/tenant/{tenant_id}/export/conversation` | Yes | Export conversation to XLSX |
| POST | `/api/tenant/{tenant_id}/export/conversation-pdf` | Yes | Export conversation to PDF (via Claude API) |

## WebSocket

| Path | Auth | Description |
|------|------|-------------|
| `/ws?token=<JWT>` | Yes (via query param) | WebSocket connection |

JWT is passed as a query parameter since WebSocket connections cannot use cookies or headers for the initial handshake. See [Real-Time](real-time.md) for protocol details.

## Health Check

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/health` | No | Health check (returns `{ "status": "ok", "version": "0.1.0" }`) |
