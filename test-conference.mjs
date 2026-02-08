#!/usr/bin/env node
/**
 * Conference signaling test script.
 * Tests the full REST + WS flow without any mocking.
 */
import WebSocket from 'ws';

const API = 'http://localhost:5001';
const WS_URL = 'ws://localhost:5001/ws';

const unique = `test_${Date.now()}`;

async function api(method, path, token, body) {
  const headers = { 'Content-Type': 'application/json' };
  if (token) headers['Authorization'] = `Bearer ${token}`;
  const opts = { method, headers };
  if (body) opts.body = JSON.stringify(body);
  const resp = await fetch(`${API}${path}`, opts);
  const text = await resp.text();
  let json;
  try { json = JSON.parse(text); } catch { json = text; }
  if (!resp.ok) {
    console.error(`  [FAIL] ${method} ${path} => ${resp.status}`, json);
    throw new Error(`${method} ${path} failed: ${resp.status}`);
  }
  return json;
}

function connectWs(token) {
  return new Promise((resolve, reject) => {
    const ws = new WebSocket(`${WS_URL}?token=${token}`);
    const messages = [];
    ws.on('open', () => {});
    ws.on('message', (data) => {
      const msg = JSON.parse(data.toString());
      messages.push(msg);
      if (msg.type === 'connected') {
        resolve({ ws, messages });
      }
    });
    ws.on('error', (err) => reject(err));
    setTimeout(() => reject(new Error('WS connect timeout')), 5000);
  });
}

function waitForMessage(messages, ws, type, timeoutMs = 10000) {
  // Check if already received
  const idx = messages.findIndex(m => m.type === type);
  if (idx !== -1) {
    const msg = messages.splice(idx, 1)[0];
    return Promise.resolve(msg);
  }
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      console.error(`  [TIMEOUT] Waiting for "${type}". Messages received so far:`,
        messages.map(m => m.type));
      reject(new Error(`Timeout waiting for ${type}`));
    }, timeoutMs);

    const handler = (data) => {
      const msg = JSON.parse(data.toString());
      messages.push(msg);
      if (msg.type === type) {
        clearTimeout(timer);
        ws.off('message', handler);
        messages.splice(messages.indexOf(msg), 1);
        resolve(msg);
      }
    };
    ws.on('message', handler);
  });
}

function wsSend(ws, type, data) {
  const msg = JSON.stringify({ type, data });
  console.log(`  >> Sending: ${type}`);
  ws.send(msg);
}

async function main() {
  console.log('=== Conference Signaling Test ===\n');

  // Step 1: Register
  console.log('1. Register user');
  const user = await api('POST', '/api/auth/register', null, {
    username: unique,
    email: `${unique}@test.com`,
    password: 'Test1234!',
    display_name: 'Test User',
  });
  const token = user.access_token;
  console.log(`  [OK] Registered, token: ${token.slice(0, 20)}...`);

  // Step 2: Create tenant
  console.log('\n2. Create tenant');
  const tenant = await api('POST', '/api/tenant', token, {
    name: 'Test Org',
    slug: `conf-${Date.now()}`,
  });
  const tenantId = tenant.id;
  console.log(`  [OK] Tenant: ${tenantId}`);

  // Step 3: Create conference
  console.log('\n3. Create conference');
  const conf = await api('POST', `/api/tenant/${tenantId}/conference`, token, {
    subject: 'Signaling Test',
  });
  const confId = conf.id;
  console.log(`  [OK] Conference: ${confId}, status: ${conf.status}`);

  // Step 4: Start conference
  console.log('\n4. Start conference');
  const startResp = await api('POST', `/api/tenant/${tenantId}/conference/${confId}/start`, token);
  console.log(`  [OK] Started: ${startResp.started}`);
  console.log(`  rtp_capabilities codecs: ${JSON.stringify(startResp.rtp_capabilities?.codecs?.length ?? 'missing')}`);

  // Step 5: Verify conference status
  console.log('\n5. Fetch conference status');
  const confDetail = await api('GET', `/api/tenant/${tenantId}/conference/${confId}`, token);
  console.log(`  [OK] Status: ${confDetail.status}`);

  // Step 6: REST join
  console.log('\n6. REST join');
  const joinResp = await api('POST', `/api/tenant/${tenantId}/conference/${confId}/join`, token);
  console.log(`  [OK] Joined: ${joinResp.joined}, participant_id: ${joinResp.participant_id}`);
  console.log(`  transports in REST response: ${joinResp.transports ? 'YES' : 'NO (expected - handled via WS)'}`);

  // Step 7: Connect WebSocket
  console.log('\n7. Connect WebSocket');
  const { ws, messages } = await connectWs(token);
  console.log(`  [OK] WS connected, got "connected" message`);

  // Step 8: Send media:join
  console.log('\n8. Send media:join');
  wsSend(ws, 'media:join', { conference_id: confId });

  // Step 9: Wait for router_capabilities
  console.log('\n9. Wait for media:router_capabilities');
  try {
    const capsMsg = await waitForMessage(messages, ws, 'media:router_capabilities', 5000);
    console.log(`  [OK] Got router_capabilities`);
    const codecs = capsMsg.data?.rtp_capabilities?.codecs;
    console.log(`  codecs count: ${codecs?.length}`);
    if (codecs) {
      codecs.forEach((c, i) => console.log(`    codec[${i}]: ${c.mimeType}`));
    }
  } catch (err) {
    console.error(`  [FAIL] ${err.message}`);
    // Log any messages we did receive
    console.log('  All buffered messages:', messages.map(m => JSON.stringify(m)).join('\n    '));
    ws.close();
    process.exit(1);
  }

  // Step 10: Wait for transport_created
  console.log('\n10. Wait for media:transport_created');
  let transportData;
  try {
    const tMsg = await waitForMessage(messages, ws, 'media:transport_created', 5000);
    transportData = tMsg.data;
    console.log(`  [OK] Got transport_created`);
    console.log(`  send_transport.id: ${transportData.send_transport?.id}`);
    console.log(`  recv_transport.id: ${transportData.recv_transport?.id}`);
    console.log(`  send ice_candidates count: ${transportData.send_transport?.ice_candidates?.length}`);
    console.log(`  send ice_parameters: ${JSON.stringify(transportData.send_transport?.ice_parameters)?.slice(0, 80)}...`);
    console.log(`  send dtls_parameters fingerprints: ${transportData.send_transport?.dtls_parameters?.fingerprints?.length}`);
  } catch (err) {
    console.error(`  [FAIL] ${err.message}`);
    ws.close();
    process.exit(1);
  }

  // Step 11: Send media:connect_transport (send transport)
  console.log('\n11. Send media:connect_transport (send)');
  // We need valid DTLS parameters. In a real flow, mediasoup-client generates these.
  // For testing, we send a minimal DTLS params to see if the server accepts the format.
  wsSend(ws, 'media:connect_transport', {
    conference_id: confId,
    transport_id: transportData.send_transport.id,
    dtls_parameters: {
      role: 'client',
      fingerprints: [
        {
          algorithm: 'sha-256',
          value: 'AA:BB:CC:DD:EE:FF:00:11:22:33:44:55:66:77:88:99:AA:BB:CC:DD:EE:FF:00:11:22:33:44:55:66:77:88:99',
        },
      ],
    },
  });

  // Wait a moment for any error response
  console.log('  Waiting for potential error...');
  try {
    const errMsg = await waitForMessage(messages, ws, 'media:error', 2000);
    console.log(`  [WARN] Got media:error: ${errMsg.data?.message}`);
  } catch {
    console.log(`  [OK] No error (connect_transport accepted or processed)`);
  }

  // Step 12: REST leave
  console.log('\n12. REST leave');
  wsSend(ws, 'media:leave', { conference_id: confId });
  await new Promise(r => setTimeout(r, 500));
  await api('POST', `/api/tenant/${tenantId}/conference/${confId}/leave`, token);
  console.log(`  [OK] Left conference`);

  // Done
  ws.close();
  console.log('\n=== ALL SIGNALING STEPS PASSED ===');
  process.exit(0);
}

main().catch((err) => {
  console.error('\n=== TEST FAILED ===');
  console.error(err);
  process.exit(1);
});
