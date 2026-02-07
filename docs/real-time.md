# Real-Time

Roomler2 uses WebSocket for real-time features: presence updates, typing indicators, and server-pushed events.

## Connection Flow

```
Browser                                     Axum Server
  │                                              │
  │  GET /ws?token=<JWT>                         │
  ├─────────────────────────────────────────────►│
  │                                              │
  │  (Server verifies JWT, extracts user_id)     │
  │                                              │
  │  101 Switching Protocols                     │
  │◄─────────────────────────────────────────────┤
  │                                              │
  │  { "type": "connected",                      │
  │    "user_id": "6..." }                       │
  │◄─────────────────────────────────────────────┤
  │                                              │
  │  ─── bidirectional messages ───              │
  │                                              │
```

1. Client opens WebSocket to `/ws?token=<JWT>` (JWT is passed as query parameter since WS handshake cannot use cookies/headers)
2. Server verifies the JWT before accepting the upgrade
3. On success, connection is registered in `WsStorage` under the user's ID
4. Server sends a `connected` confirmation message
5. Bidirectional message exchange begins

## Message Types

### Server → Client

| Type | Payload | Description |
|------|---------|-------------|
| `connected` | `{ user_id }` | Connection established confirmation |
| `pong` | `{}` | Response to client ping |
| `typing:start` | `{ channel_id, user_id }` | User started typing in channel |
| `typing:stop` | `{ channel_id, user_id }` | User stopped typing in channel |
| `presence:update` | `{ user_id, presence }` | User presence changed |

### Client → Server

| Type | Payload | Description |
|------|---------|-------------|
| `ping` | `{}` | Application-level keepalive |
| `typing:start` | `{ channel_id }` | Notify channel members of typing |
| `typing:stop` | `{ channel_id }` | Notify channel members typing stopped |
| `presence:update` | `{ presence }` | Update own presence status |

All messages are JSON:

```json
{
  "type": "typing:start",
  "data": {
    "channel_id": "6..."
  }
}
```

## WsStorage

`WsStorage` tracks all active WebSocket connections using a `DashMap<ObjectId, Vec<WsSender>>`.

- Each user can have **multiple connections** (multiple browser tabs, devices)
- Connections are identified by `Arc<Mutex<SplitSink<WebSocket, Message>>>` pointer equality
- On disconnect, the specific connection is removed; the user entry is cleaned up when no connections remain

```rust
pub struct WsStorage {
    connections: DashMap<ObjectId, Vec<WsSender>>,
}
```

Key operations:
- `add(user_id, sender)` -- register a new connection
- `remove(user_id, sender)` -- unregister using Arc pointer equality
- `get_senders(user_id)` -- get all senders for a user
- `all_user_ids()` -- list all connected users
- `connection_count()` -- total active connections across all users

## Dispatcher

The dispatcher sends messages to specific users or broadcasts to groups:

- **`send_to_user(ws_storage, user_id, message)`** -- send to all connections of a specific user
- **`broadcast(ws_storage, user_ids, message)`** -- send to all connections of multiple users

### Broadcast Scoping

| Event | Recipients |
|-------|-----------|
| `typing:start` / `typing:stop` | All members of the channel **except** the sender |
| `presence:update` | All connected users |
| `pong` | Only the sender |

For typing indicators, the server looks up channel member IDs and broadcasts to all members except the typing user. For presence, the update goes to all connected users.

## Presence

Users have one of five presence states:

| State | Description |
|-------|-------------|
| `online` | Actively connected and interacting |
| `idle` | Connected but inactive |
| `dnd` | Do not disturb (suppresses notifications) |
| `offline` | Not connected (default) |
| `invisible` | Connected but appears offline to others |

Presence is updated via the WebSocket `presence:update` message and broadcast to all connected users.

## Protocol-Level Ping/Pong

In addition to application-level `ping`/`pong` messages, the server handles WebSocket protocol-level `Ping` frames by responding with `Pong` frames automatically. This keeps the connection alive at the transport layer.

## Future: mediasoup Integration (Phase 5)

The codebase includes placeholder configuration for mediasoup (SFU for WebRTC):

- `ROOMLER__MEDIASOUP__NUM_WORKERS` -- number of worker processes
- `ROOMLER__MEDIASOUP__LISTEN_IP` -- bind address
- `ROOMLER__MEDIASOUP__ANNOUNCED_IP` -- public IP
- `ROOMLER__MEDIASOUP__RTC_MIN_PORT` / `RTC_MAX_PORT` -- UDP port range

The mediasoup dependency is commented out in `Cargo.toml` (`# mediasoup = "0.20"`). When enabled, it will provide:
- SFU-based audio/video routing (no peer-to-peer limitations)
- Server-side recording
- Scalable conference support

TURN server (Coturn) is already configured for NAT traversal.
