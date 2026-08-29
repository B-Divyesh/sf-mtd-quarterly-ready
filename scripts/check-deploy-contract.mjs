import { readFile } from 'node:fs/promises';

const deployment = await readFile(new URL('./deploy-container.sh', import.meta.url), 'utf8');
const topologyVerifier = await readFile(new URL('./verify-azure-topology.sh', import.meta.url), 'utf8');
const dockerfile = await readFile(new URL('../Dockerfile', import.meta.url), 'utf8');
const packageJson = JSON.parse(await readFile(new URL('../package.json', import.meta.url), 'utf8'));
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

if (!/^ENV .*DATA_DIR=\/data DATABASE_DIR=\/data .*SAFE_QA_FIXTURES=1/m.test(dockerfile)) {
  throw new Error('Container runtime defaults must include SAFE_QA_FIXTURES=1');
}
if (deployment.indexOf('missing approved HMRC integration secret references; refusing a release deployment') > deployment.indexOf('echo "== ACR build')) {
  throw new Error('Deployment must check approved HMRC secret references before building or changing the Container App.');
}
if (packageJson.scripts['verify:release'] !== 'VERIFY_AZURE_TOPOLOGY=1 REQUIRE_APPROVED_HMRC=1 node scripts/verify-live.mjs') {
  throw new Error('Release verification must require both the live Azure topology and approved HMRC capability.');
}

console.log('Deployment contract: durable /data, one replica, SNI binding, build identity, Key Vault-backed approved HMRC provider, explicit handoff-only fallback, and the verified non-charging QA fixture are configured in the image and app template.');
