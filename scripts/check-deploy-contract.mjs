import { access, readFile } from 'node:fs/promises';
import { constants } from 'node:fs';

const deployment = await readFile(new URL('./deploy-container.sh', import.meta.url), 'utf8');
const topologyVerifier = await readFile(new URL('./verify-azure-topology.sh', import.meta.url), 'utf8');
const dockerfile = await readFile(new URL('../Dockerfile', import.meta.url), 'utf8');
const server = await readFile(new URL('../src/main.rs', import.meta.url), 'utf8');
const packageJson = JSON.parse(await readFile(new URL('../package.json', import.meta.url), 'utf8'));
const verifyUrlScript = new URL('./verify-url.sh', import.meta.url);
const verifyRateLimitScript = new URL('./verify-rate-limit.mjs', import.meta.url);
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
  'mtd-quarterly-ready-approved-provider-submission-url',
  'mtd-quarterly-ready-approved-provider-service-token',
  'mtd-quarterly-ready-approved-provider-authorize-url',
  'mtd-quarterly-ready-approved-provider-token-url',
  'mtd-quarterly-ready-approved-provider-client-id',
  'mtd-quarterly-ready-approved-provider-client-secret',
  'mtd-quarterly-ready-approved-provider-approval-reference',
  'DEPLOYMENT_MODE="${DEPLOYMENT_MODE:-approved}"',
  'handoff-only',
  '"name":"HMRC_INTEGRATION_MODE","value":"approved_provider"',
  'missing approved HMRC provider or taxpayer-consent secret references; refusing a release deployment',
  'REQUIRE_APPROVED_HMRC=1',
  'VERIFY_AZURE_TOPOLOGY=1',
];

for (const text of required) {
  if (!deployment.includes(text)) throw new Error(`Deployment contract is missing ${text}`);
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
  || !server.includes('destination.sync_all().await?')) {
  throw new Error('Container runtime must persist each local SQLite mutation to the mounted /data snapshot before success.');
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
if (deployment.indexOf('missing approved HMRC provider or taxpayer-consent secret references; refusing a release deployment') > deployment.indexOf('echo "== ACR build')) {
  throw new Error('Deployment must check approved HMRC secret references before building or changing the Container App.');
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
if (packageJson.scripts['verify:release'] !== 'VERIFY_AZURE_TOPOLOGY=1 REQUIRE_APPROVED_HMRC=1 node scripts/verify-live.mjs') {
  throw new Error('Release verification must require both the live Azure topology and approved HMRC capability.');
}
if (packageJson.scripts['verify:url'] !== 'bash scripts/verify-url.sh') {
  throw new Error('Package scripts must expose the repeatable URL accessibility check.');
}
if (packageJson.scripts['verify:rate-limit'] !== 'node scripts/verify-rate-limit.mjs') {
  throw new Error('Package scripts must expose the stable-connection rate-limit check.');
}
await access(verifyUrlScript, constants.X_OK);
await access(verifyRateLimitScript, constants.R_OK);

console.log('Deployment contract: synced durable /data SQLite snapshot, one replica, SNI binding, build identity, Key Vault-backed approved HMRC provider with taxpayer OAuth consent, explicit handoff-only fallback, concurrent persistence verification, non-charging QA fixture, and repeatable URL accessibility check are configured.');
