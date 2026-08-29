import { readFile } from 'node:fs/promises';

const deployment = await readFile(new URL('./deploy-container.sh', import.meta.url), 'utf8');
const required = [
  'FILE_SHARE="sf-mtd-quarterly-ready-data-v2"',
  'ENV_STORAGE="mtd-quarterly-ready-data-v2"',
  '"mountPath": "/data"',
  '"storageType": "AzureFile"',
  '"minReplicas": 1, "maxReplicas": 1',
  '"bindingType": "SniEnabled"',
  'BUILD_SHA=${SOURCE_SHA}',
  'az containerapp revision deactivate',
];

for (const text of required) {
  if (!deployment.includes(text)) throw new Error(`Deployment contract is missing ${text}`);
}

console.log('Deployment contract: durable /data, one replica, SNI binding, and build identity are configured.');
