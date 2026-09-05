import { access, readFile } from 'node:fs/promises';
import { constants } from 'node:fs';

const deployment = await readFile(new URL('./deploy-container.sh', import.meta.url), 'utf8');
const topologyVerifier = await readFile(new URL('./verify-azure-topology.sh', import.meta.url), 'utf8');
const dockerfile = await readFile(new URL('../Dockerfile', import.meta.url), 'utf8');
const server = await readFile(new URL('../src/main.rs', import.meta.url), 'utf8');
const packageJson = JSON.parse(await readFile(new URL('../package.json', import.meta.url), 'utf8'));
const verifyUrlScript = new URL('./verify-url.sh', import.meta.url);
const verifyRateLimitScript = new URL('./verify-rate-limit.mjs', import.meta.url);
const verifyConcurrencyScript = new URL('./verify-concurrent-workspaces.mjs', import.meta.url);
const liveVerifier = await readFile(new URL('./verify-live.mjs', import.meta.url), 'utf8');
const required = [
  'FILE_SHARE="sf-mtd-quarterly-ready-data-v3"',
  'ENV_STORAGE="mtd-quarterly-ready-data-v3"',
  '"mountPath": "/data"',
  '"storageType": "AzureFile"',
  '"minReplicas": 1, "maxReplicas": 1',
  '"activeRevisionsMode": "Single"',
  '"bindingType": "SniEnabled"',
  '--headers "Content-Type=application/json"',
  'BUILD_SHA=${SOURCE_SHA}',
  'az acr repository show --name "${REGISTRY}" --image "${IMAGE_TAG}" --query digest -o tsv',
  '${APP}@${IMAGE_DIGEST}',
  'deployment image is ${DEPLOYED_IMAGE}, expected immutable ${IMAGE}',
  'az containerapp revision deactivate',
  '"name": "SAFE_QA_FIXTURES", "value": "1"',
  'verify non-charging QA entitlement',
  '/api/qa/entitlement',
  '"charges":false',
  '"files_with_hmrc":false',
  'scripts/verify-azure-topology.sh',
  'prove persistence across a replica restart',
  'prove persistence across a revision replacement',
  'DEPLOYMENT_MODE="${DEPLOYMENT_MODE:-handoff-only}"',
  'DEPLOYMENT_MODE must be handoff-only',
  '"secrets": []',
  '"name": "HMRC_INTEGRATION_MODE", "value": "not_configured"',
  'REQUIRE_HANDOFF_ONLY=1',
  'VERIFY_AZURE_TOPOLOGY=1',
];

for (const text of required) {
  if (!deployment.includes(text)) throw new Error(`Deployment contract is missing ${text}`);
}

for (const text of [
  'sociobot-keyvault1',
  'az keyvault',
  'keyVaultUrl',
  'secretRef',
  'HMRC_SUBMISSION_URL_SECRET',
  'HMRC_SERVICE_TOKEN_SECRET',
  'HMRC_CLIENT_SECRET_SECRET',
]) {
  if (deployment.includes(text)) throw new Error(`Deployment isolation regression: out-of-scope secret configuration remains (${text})`);
}

for (const text of [
  'activeRevisionsMode',
  'environment storage is not the expected read-write Azure Files share',
  'storage_name',
  'file_share',
  'expected exactly one running replica',
  'container image must use an immutable digest',
]) {
  if (!topologyVerifier.includes(text)) throw new Error(`Topology regression check is missing ${text}`);
}

if (!/^ENV .*DATA_DIR=\/data .*SAFE_QA_FIXTURES=1/m.test(dockerfile)
  || !server.includes('"/tmp/quarterly-ready"')
  || !server.includes('persist_database_snapshot')
  || !server.includes('destination.sync_all().await')
  || !server.includes('fs::rename(&temporary, snapshot).await')) {
  throw new Error('Container runtime must atomically persist each local SQLite mutation to the mounted /data snapshot before success.');
}
for (const text of [
  'tower_governor',
  'RATE_LIMIT_REFILL_SECONDS: u64 = 60',
  'READ_RATE_LIMIT_BURST: u32 = 40',
  'WRITE_RATE_LIMIT_BURST: u32 = 12',
  'x-forwarded-for',
  'GovernorLayer',
]) {
  if (!server.includes(text)) throw new Error(`Server rate-limit contract is missing ${text}`);
}
if (!server.includes('hmrc_consent_callback.layer(write_limit.clone())')) {
  throw new Error('The OAuth callback must use the stricter write quota.');
}
const durabilityVerifier = await readFile(new URL('./verify-durability.mjs', import.meta.url), 'utf8');
for (const text of ['length: 10', 'concurrent_workspaces', 'readWorkspace(probe', 'Promise.all(probes.map']) {
  if (!durabilityVerifier.includes(text)) throw new Error(`Durability regression probe is missing ${text}`);
}
const revisionReplacement = deployment.indexOf('echo "== prove persistence across a revision replacement"');
const protectedReplacement = deployment.indexOf('stop_revision_for_snapshot_handoff "${CURRENT_REVISION}"', revisionReplacement);
const replacementUpdate = deployment.indexOf('az containerapp update', revisionReplacement);
if (revisionReplacement === -1 || protectedReplacement === -1 || replacementUpdate === -1 || protectedReplacement > replacementUpdate) {
  throw new Error('Revision-replacement durability verification must stop the current replica before creating its successor.');
}
if (packageJson.scripts['verify:release'] !== 'VERIFY_AZURE_TOPOLOGY=1 REQUIRE_HANDOFF_ONLY=1 node scripts/verify-live.mjs') {
  throw new Error('Release verification must require both the live Azure topology and handoff-only HMRC state.');
}
if (packageJson.scripts['verify:url'] !== 'bash scripts/verify-url.sh') {
  throw new Error('Package scripts must expose the repeatable URL accessibility check.');
}
if (packageJson.scripts['verify:rate-limit'] !== 'node scripts/verify-rate-limit.mjs') {
  throw new Error('Package scripts must expose the stable-connection rate-limit check.');
}
if (packageJson.scripts['verify:concurrency'] !== 'node scripts/verify-concurrent-workspaces.mjs') {
  throw new Error('Package scripts must expose the concurrent acknowledged-save check.');
}
if (!liveVerifier.includes("['scripts/verify-concurrent-workspaces.mjs']")
  || !liveVerifier.includes('acknowledged_documents_preserved === 20')) {
  throw new Error('Live verification must reproduce both ten-way acknowledged-save rounds.');
}
await access(verifyUrlScript, constants.X_OK);
await access(verifyRateLimitScript, constants.R_OK);
await access(verifyConcurrencyScript, constants.R_OK);

const rateLimitVerifier = await readFile(verifyRateLimitScript, 'utf8');
for (const text of [
  "'/api/hmrc/consent/callback?state=missing'",
  'OAuth callback after ${allowance} writes returned',
  'OAuth callback 429 omitted a positive Retry-After value',
]) {
  if (!rateLimitVerifier.includes(text)) throw new Error(`Live rate-limit regression check is missing ${text}`);
}

if (!liveVerifier.includes("process.env.REQUIRE_HANDOFF_ONLY === '1'")
  || !liveVerifier.includes("health.hmrc_integration_mode === 'not_configured'")) {
  throw new Error('Release verification must prove that the handoff-only deployment has no direct HMRC submission capability.');
}

console.log('Deployment contract: synced durable /data SQLite snapshot, one replica, SNI binding, build identity, explicit handoff-only HMRC state with no shared-vault secret references, concurrent persistence verification, non-charging QA fixture, and repeatable URL accessibility check are configured.');
