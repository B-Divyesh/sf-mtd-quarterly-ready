import { readFile } from 'node:fs/promises';

const deployment = await readFile(new URL('./deploy-container.sh', import.meta.url), 'utf8');
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
  'mtd-quarterly-ready-hmrc-integration-url',
  'mtd-quarterly-ready-hmrc-integration-token',
  'missing approved HMRC integration secret references; refusing a release deployment',
  'REQUIRE_APPROVED_HMRC=1',
  'VERIFY_AZURE_TOPOLOGY=1',
];

for (const text of required) {
  if (!deployment.includes(text)) throw new Error(`Deployment contract is missing ${text}`);
}

if (!/^ENV .*SAFE_QA_FIXTURES=1/m.test(dockerfile)) {
  throw new Error('Container runtime defaults must include SAFE_QA_FIXTURES=1');
}
if (packageJson.scripts['verify:release'] !== 'VERIFY_AZURE_TOPOLOGY=1 REQUIRE_APPROVED_HMRC=1 node scripts/verify-live.mjs') {
  throw new Error('Release verification must require both the live Azure topology and approved HMRC capability.');
}

console.log('Deployment contract: durable /data, one replica, SNI binding, build identity, approved HMRC capability, and the verified non-charging QA fixture are configured in both the image and app template.');
