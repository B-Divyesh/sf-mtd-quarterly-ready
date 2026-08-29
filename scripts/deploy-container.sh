#!/usr/bin/env bash
# Deploy Quarterly Ready with the state guarantees its SQLite backend requires.
# The factory invokes this from the repository root after the normal clean build.
set -euo pipefail

SLUG="mtd-quarterly-ready"
APP="sf-${SLUG}"
RESOURCE_GROUP="sociobot"
ENVIRONMENT="factory-env"
REGISTRY="sociobotregistry"
SUBSCRIPTION="${AZURE_SUBSCRIPTION_ID:-283af945-693b-4a6e-b952-df928d0a18a9}"
STORAGE_ACCOUNT="sociobotblob"
FILE_SHARE="sf-mtd-quarterly-ready-data"
ENV_STORAGE="mtd-quarterly-ready-data"
PORT=8080
SOURCE_SHA="$(git rev-parse HEAD)"
IMAGE_TAG="${APP}:${SOURCE_SHA:0:12}"
IMAGE="${REGISTRY}.azurecr.io/${IMAGE_TAG}"
RESOURCE_ID="/subscriptions/${SUBSCRIPTION}/resourceGroups/${RESOURCE_GROUP}"
ENVIRONMENT_ID="${RESOURCE_ID}/providers/Microsoft.App/managedEnvironments/${ENVIRONMENT}"
IDENTITY_ID="${RESOURCE_ID}/providers/Microsoft.ManagedIdentity/userAssignedIdentities/factory-worker-identity"
CERTIFICATE_ID="${ENVIRONMENT_ID}/managedCertificates/cert-${SLUG}"
APP_URL="https://management.azure.com${RESOURCE_ID}/providers/Microsoft.App/containerApps/${APP}?api-version=2024-03-01"

echo "== ACR build ${IMAGE_TAG}"
az acr build --registry "${REGISTRY}" --image "${IMAGE_TAG}" --file Dockerfile \
  --build-arg "BUILD_SHA=${SOURCE_SHA}" \
  --build-arg "GIT_SHA=${SOURCE_SHA}" \
  --build-arg "SOURCE_COMMIT=${SOURCE_SHA}" .

echo "== durable Azure Files workspace volume"
az storage share-rm create --resource-group "${RESOURCE_GROUP}" \
  --storage-account "${STORAGE_ACCOUNT}" --name "${FILE_SHARE}" --quota 1 --only-show-errors -o none \
  || true
STORAGE_KEY="$(az storage account keys list --resource-group "${RESOURCE_GROUP}" --account-name "${STORAGE_ACCOUNT}" --query '[0].value' -o tsv)"
az containerapp env storage set --resource-group "${RESOURCE_GROUP}" --name "${ENVIRONMENT}" \
  --storage-name "${ENV_STORAGE}" --access-mode ReadWrite \
  --azure-file-account-name "${STORAGE_ACCOUNT}" --azure-file-share-name "${FILE_SHARE}" \
  --azure-file-account-key "${STORAGE_KEY}" --only-show-errors -o none
unset STORAGE_KEY

echo "== container app (one replica, mounted /data)"
az rest --method patch --url "${APP_URL}" --body "$(cat <<JSON
{
  "properties": {
    "configuration": {
      "ingress": {
        "customDomains": [{
          "name": "${SLUG}.sociobot.in",
          "bindingType": "SniEnabled",
          "certificateId": "${CERTIFICATE_ID}"
        }]
      }
    },
    "template": {
      "containers": [{
        "name": "app",
        "image": "${IMAGE}",
        "resources": {"cpu": 0.5, "memory": "1Gi"},
        "env": [{"name": "PORT", "value": "${PORT}"}],
        "volumeMounts": [{"volumeName": "workspace-data", "mountPath": "/data"}]
      }],
      "scale": {"minReplicas": 1, "maxReplicas": 1},
      "volumes": [{"name": "workspace-data", "storageType": "AzureFile", "storageName": "${ENV_STORAGE}"}]
    }
  }
}
JSON
)" --only-show-errors -o none

echo "== wait for deployment"
for _ in $(seq 1 36); do
  HEALTH="$(curl --silent --show-error --fail --max-time 15 "https://${SLUG}.sociobot.in/health" || true)"
  if [[ "${HEALTH}" == *"${SOURCE_SHA}"* ]]; then
    printf '%s\n' "${HEALTH}"
    exit 0
  fi
  sleep 10
done

echo "deployment did not expose ${SOURCE_SHA} on /health" >&2
exit 1
