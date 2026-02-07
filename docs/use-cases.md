# Use Cases

## Permission System

Roomler2 uses a **u64 bitfield** for permissions. Each bit represents one permission flag. Roles are assigned to tenant members, and the effective permission is the union of all assigned role permissions.

### Permission Flags (24 bits)

| Bit | Flag | Description |
|-----|------|-------------|
| 0 | `VIEW_CHANNELS` | See channels |
| 1 | `MANAGE_CHANNELS` | Create, edit, delete channels |
| 2 | `MANAGE_ROLES` | Create, edit, delete roles |
| 3 | `MANAGE_TENANT` | Edit tenant settings |
| 4 | `KICK_MEMBERS` | Remove members |
| 5 | `BAN_MEMBERS` | Ban members |
| 6 | `INVITE_MEMBERS` | Create invites |
| 7 | `SEND_MESSAGES` | Send messages |
| 8 | `SEND_THREADS` | Reply in threads |
| 9 | `EMBED_LINKS` | URLs auto-preview |
| 10 | `ATTACH_FILES` | Upload file attachments |
| 11 | `READ_HISTORY` | Read message history |
| 12 | `MENTION_EVERYONE` | Use @everyone and @here |
| 13 | `MANAGE_MESSAGES` | Delete/pin others' messages |
| 14 | `ADD_REACTIONS` | Add emoji reactions |
| 15 | `CONNECT_VOICE` | Join voice channels |
| 16 | `SPEAK` | Speak in voice channels |
| 17 | `STREAM_VIDEO` | Share video/screen |
| 18 | `MUTE_MEMBERS` | Server-mute others |
| 19 | `DEAFEN_MEMBERS` | Server-deafen others |
| 20 | `MOVE_MEMBERS` | Move members between voice channels |
| 21 | `MANAGE_MEETINGS` | Create, start, end conferences |
| 22 | `MANAGE_DOCUMENTS` | Manage files and documents |
| 23 | `ADMINISTRATOR` | Bypasses all permission checks |

### Default Role Permissions

| Role | Flags Included |
|------|---------------|
| **DEFAULT_MEMBER** | VIEW_CHANNELS, SEND_MESSAGES, SEND_THREADS, EMBED_LINKS, ATTACH_FILES, READ_HISTORY, ADD_REACTIONS, CONNECT_VOICE, SPEAK, STREAM_VIDEO |
| **DEFAULT_ADMIN** | DEFAULT_MEMBER + MANAGE_CHANNELS, MANAGE_ROLES, KICK_MEMBERS, BAN_MEMBERS, INVITE_MEMBERS, MENTION_EVERYONE, MANAGE_MESSAGES, MUTE_MEMBERS, DEAFEN_MEMBERS, MOVE_MEMBERS, MANAGE_MEETINGS, MANAGE_DOCUMENTS |
| **ALL (Owner)** | All 24 bits set (includes ADMINISTRATOR) |

### Permission Check Logic

```
has(permissions, flag) = (permissions & ADMINISTRATOR != 0) || (permissions & flag == flag)
```

The `ADMINISTRATOR` flag bypasses all other checks.

### Channel Permission Overwrites

Channels can override base permissions per-role or per-user:

```
effective = (base_permissions & ~deny) | allow
```

Each `PermissionOverwrite` specifies a `target_id` (role or user), an `allow` mask, and a `deny` mask.

## Authentication Flow

```
┌──────────┐   POST /api/auth/register   ┌──────────┐
│  Browser  ├───────────────────────────►│  Axum    │
│           │   { email, username,       │  API     │
│           │     display_name, password,│          │
│           │     tenant_name?,          │          │
│           │     tenant_slug? }         │          │
│           │                            │          │
│           │◄───────────────────────────┤          │
│           │   Set-Cookie: access_token │          │
│           │   { access_token,          │          │
│           │     refresh_token,         │          │
│           │     expires_in, user }     │          │
└──────────┘                             └──────────┘
```

1. **Register** -- user provides email, username, display_name, password. Optionally creates a tenant.
2. **Login** -- by username or email + password. Argon2 hash verification.
3. **Token delivery** -- JWT access token set as httpOnly cookie. Refresh token returned in body.
4. **Protected requests** -- access token read from cookie or `Authorization: Bearer` header.
5. **Token refresh** -- POST refresh_token to get new access + refresh tokens.

## Channel Lifecycle

```
Create Channel              Join Channel
     │                           │
     ▼                           ▼
  Channel exists ──────► ChannelMember created
     │                           │
     ▼                           ▼
  Send messages             Read messages
     │                           │
     ├── Start thread            ├── Unread tracking
     ├── Add reactions           ├── Mention tracking
     ├── Pin messages            ├── Notification prefs
     └── Upload files            └── Mute channel
                                     │
                                     ▼
                               Leave Channel
                                     │
                                     ▼
                             ChannelMember deleted
```

### Channel Types

| Type | Description |
|------|-------------|
| `category` | Grouping container, no messages, acts as parent |
| `text` | Standard text messaging channel |
| `voice` | Voice/video channel with media settings |
| `announcement` | Restricted posting (admins/mods only) |
| `forum` | Thread-based discussion |
| `stage` | Presentation mode with speakers/audience |
| `dm` | Direct message (2 users) |
| `group_dm` | Group direct message (multiple users) |

Channels support hierarchy via `parent_id` -- a `category` channel can contain `text`, `voice`, and other channel types.

## Conference Lifecycle

```
Create Conference
     │
     ▼
  Status: Scheduled
     │
     ▼
  Start Conference
     │
     ▼
  Status: In Progress
     │
     ├── Participants join/leave
     ├── Screen sharing
     ├── Recording starts
     └── Transcription runs
     │
     ▼
  End Conference
     │
     ▼
  Status: Ended
     │
     ├── Recordings processed (BackgroundTask)
     └── Transcriptions generated (BackgroundTask)
```

### Conference Types

| Type | Description |
|------|-------------|
| `instant` | Start immediately |
| `scheduled` | Planned for a future time |
| `recurring` | Repeats daily/weekly/monthly |
| `persistent` | Always-available room |

### Participant Roles

| Role | Description |
|------|-------------|
| `organizer` | Created the conference, full control |
| `co_organizer` | Delegated control |
| `presenter` | Can share screen/present |
| `attendee` | Default role, can view and speak |

## File Lifecycle

```
Upload File
     │
     ▼
  FileContext assigned (message/document/channel/conference/profile)
     │
     ├── Stored in MinIO (S3-compatible)
     ├── Virus scan (scan_status: pending → clean/malware)
     └── Version tracking (version chain via previous_version_id)
     │
     ▼
  AI Recognition (optional)
     │
     ▼
  Claude API extracts text/structure
     │
     ▼
  recognized_content populated
     │
     ├── raw_text
     ├── structured_data (JSON)
     ├── document_type
     └── confidence score
     │
     ▼
  Download / Cloud Sync
     │
     ├── Google Drive
     ├── OneDrive
     └── Dropbox
```

## Multi-Tenant Data Flow

All data is scoped to a tenant via `tenant_id`. The API URL structure enforces this:

```
/api/tenant/{tenant_id}/channel/{channel_id}/message
```

- Users can belong to multiple tenants (via `TenantMember`)
- Each tenant has its own roles, channels, conferences, and files
- A user's permissions differ per tenant (based on assigned roles in that tenant)
- Cross-tenant data access is prevented at the DAO layer

## Invite Flow

```
Admin creates invite
     │
     ▼
  Invite { code, max_uses, expires_at, assign_role_ids }
     │
     ▼
  Share code/link
     │
     ▼
  Recipient accepts invite
     │
     ├── TenantMember created (or updated)
     ├── Roles assigned (assign_role_ids)
     └── use_count incremented
     │
     ▼
  Invite status:
     ├── active (still usable)
     ├── exhausted (use_count >= max_uses)
     ├── expired (past expires_at)
     └── revoked (manually disabled)
```

Invites can target a specific email or be open. They can optionally scope to a channel.
