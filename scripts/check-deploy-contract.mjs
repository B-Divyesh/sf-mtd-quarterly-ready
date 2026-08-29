import { access, readFile } from 'node:fs/promises';
import { constants } from 'node:fs';

const deployment = await readFile(new URL('./deploy-container.sh', import.meta.url), 'utf8');
const topologyVerifier = await readFile(new URL('./verify-azure-topology.sh', import.meta.url), 'utf8');
const dockerfile = await readFile(new URL('../Dockerfile', import.meta.url), 'utf8');
const packageJson = JSON.parse(await readFile(new URL('../package.json', import.meta.url), 'utf8'));
const verifyUrlScript = new URL('./verify-url.sh', import.meta.url);
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
  'az containerapp revision deactivate',
  '"name": "SAFE_QA_FIXTURES", "value": "1"',
  'verify non-charging QA entitlement',
  '/api/qa/entitlement',
  '"charges":false',
  '"files_with_hmrc":false',
  'scripts/verify-azure-topology.sh',
  'prove persistence across a replica restart',
  'prove persistence across a revision replacement',
  'mtd-quarterly-ready-approved-hmrc-url',
  'mtd-quarterly-ready-approved-hmrc-token',
  'DEPLOYMENT_MODE="${DEPLOYMENT_MODE:-approved}"',
  'handoff-only',
  '"name":"HMRC_INTEGRATION_MODE","value":"approved_provider"',
  'missing approved HMRC integration secret references; refusing a release deployment',
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
]) {
  if (!topologyVerifier.includes(text)) throw new Error(`Topology regression check is missing ${text}`);
}

if (!/^ENV .*DATA_DIR=\/data DATABASE_DIR=\/tmp\/quarterly-ready .*SAFE_QA_FIXTURES=1/m.test(dockerfile)) {
  throw new Error('Container runtime must keep the live SQLite file local and its durable snapshot under /data.');
}
if (deployment.indexOf('missing approved HMRC integration secret references; refusing a release deployment') > deployment.indexOf('echo "== ACR build')) {
  throw new Error('Deployment must check approved HMRC secret references before building or changing the Container App.');
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
await access(verifyUrlScript, constants.X_OK);

console.log('Deployment contract: durable /data, one replica, SNI binding, build identity, Key Vault-backed approved HMRC provider, explicit handoff-only fallback, non-charging QA fixture, and repeatable URL accessibility check are configured.');
