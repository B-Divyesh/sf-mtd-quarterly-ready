#!/usr/bin/env bash
# Deploy Quarterly Ready with the state guarantees its SQLite backend requires.
#
# This product-owned deployment contract is handoff-only. It deploys the
# records/CSV/accountant-handoff product and deliberately configures no direct
# HMRC submission capability. A filing integration needs a separately reviewed
# product-owned contract; this script must never inspect a shared vault or bind
# credentials on its behalf.
set -euo pipefail

SLUG="mtd-quarterly-ready"
APP="sf-${SLUG}"
RESOURCE_GROUP="sociobot"
ENVIRONMENT="factory-env"
REGISTRY="sociobotregistry"
SUBSCRIPTION="${AZURE_SUBSCRIPTION_ID:-283af945-693b-4a6e-b952-df928d0a18a9}"
STORAGE_ACCOUNT="sociobotblob"
FILE_SHARE="sf-mtd-quarterly-ready-data-v3"
ENV_STORAGE="mtd-quarterly-ready-data-v3"
PORT=8080
DEPLOYMENT_MODE="${DEPLOYMENT_MODE:-handoff-only}"
SOURCE_SHA="$(git rev-parse HEAD)"
IMAGE_TAG="${APP}:${SOURCE_SHA:0:12}"
IMAGE="${REGISTRY}.azurecr.io/${IMAGE_TAG}"
RESOURCE_ID="/subscriptions/${SUBSCRIPTION}/resourceGroups/${RESOURCE_GROUP}"
ENVIRONMENT_ID="${RESOURCE_ID}/providers/Microsoft.App/managedEnvironments/${ENVIRONMENT}"
IDENTITY_ID="${RESOURCE_ID}/providers/Microsoft.ManagedIdentity/userAssignedIdentities/factory-worker-identity"
CERTIFICATE_ID="${ENVIRONMENT_ID}/managedCertificates/cert-${SLUG}"
APP_URL="https://management.azure.com${RESOURCE_ID}/providers/Microsoft.App/containerApps/${APP}?api-version=2024-03-01"

EXPECTED_HMRC_MODE="not_configured"
if [[ "${DEPLOYMENT_MODE}" != "handoff-only" ]]; then
  echo "DEPLOYMENT_MODE must be handoff-only; direct HMRC submission is not configured by this product deployment" >&2
  exit 2
fi
echo "handoff-only product contract selected: no direct HMRC submission configuration will be deployed"

echo "== ACR build ${IMAGE_TAG}"
az acr build --registry "${REGISTRY}" --image "${IMAGE_TAG}" --file Dockerfile \
  --build-arg "BUILD_SHA=${SOURCE_SHA}" \
  --build-arg "GIT_SHA=${SOURCE_SHA}" \
  --build-arg "SOURCE_COMMIT=${SOURCE_SHA}" .
IMAGE_DIGEST="$(az acr repository show --name "${REGISTRY}" --image "${IMAGE_TAG}" --query digest -o tsv)"
if [[ -z "${IMAGE_DIGEST}" || "${IMAGE_DIGEST}" != sha256:* ]]; then
  echo "ACR did not return an immutable image digest for ${IMAGE_TAG}" >&2
  exit 1
fi
IMAGE="${REGISTRY}.azurecr.io/${APP}@${IMAGE_DIGEST}"
echo "== immutable image ${IMAGE}"

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
# SQLite and the in-process Governor quotas are serial process resources. A one-replica
# hand-off prevents divergent records or multiplied per-client allowances.
stop_revision_for_snapshot_handoff() {
  local revision="$1"
  if [[ -z "${revision}" ]]; then
    return 0
  fi

  az containerapp revision deactivate --resource-group "${RESOURCE_GROUP}" --name "${APP}" --revision "${revision}" --only-show-errors -o none || true
  for _ in $(seq 1 36); do
    local running
    running="$(az containerapp replica list --resource-group "${RESOURCE_GROUP}" --name "${APP}" --revision "${revision}" --query "length([?properties.runningState=='Running'])" -o tsv 2>/dev/null || echo 0)"
    if [[ "${running}" == "0" ]]; then
      return 0
    fi
    sleep 5
  done

  echo "revision ${revision} did not stop; refusing concurrent SQLite writers" >&2
  exit 1
}

READY_REVISION="$(az containerapp show --resource-group "${RESOURCE_GROUP}" --name "${APP}" --query 'properties.latestReadyRevisionName' -o tsv 2>/dev/null || true)"
stop_revision_for_snapshot_handoff "${READY_REVISION}"
az rest --method patch --url "${APP_URL}" --headers "Content-Type=application/json" --body "$(cat <<JSON
{
  "properties": {
    "configuration": {
      "activeRevisionsMode": "Single",
      "secrets": [],
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
        "env": [{"name": "PORT", "value": "${PORT}"}, {"name": "SAFE_QA_FIXTURES", "value": "1"}, {"name": "HMRC_INTEGRATION_MODE", "value": "not_configured"}],
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
DEPLOYED=0
for _ in $(seq 1 36); do
  HEALTH="$(curl --silent --show-error --fail --max-time 15 "https://${SLUG}.sociobot.in/health" || true)"
  if [[ "${HEALTH}" == *"${SOURCE_SHA}"* \
    && "${HEALTH}" == *'"safe_qa_fixtures":true'* \
    && "${HEALTH}" == *"\"hmrc_integration_mode\":\"${EXPECTED_HMRC_MODE}\""* ]]; then
    printf '%s\n' "${HEALTH}"
    DEPLOYED=1
    break
  fi
  sleep 10
done

if [[ "${DEPLOYED}" != "1" ]]; then
  echo "deployment did not expose ${SOURCE_SHA} with safe fixtures and HMRC mode ${EXPECTED_HMRC_MODE} on /health" >&2
  exit 1
fi
DEPLOYED_IMAGE="$(az containerapp show --resource-group "${RESOURCE_GROUP}" --name "${APP}" --query 'properties.template.containers[0].image' -o tsv)"
if [[ "${DEPLOYED_IMAGE}" != "${IMAGE}" ]]; then
  echo "deployment image is ${DEPLOYED_IMAGE}, expected immutable ${IMAGE}" >&2
  exit 1
fi

echo "== verify non-charging QA entitlement"
QA_FIXTURE="$(curl --silent --show-error --fail --max-time 15 "https://${SLUG}.sociobot.in/api/qa/entitlement" || true)"
if [[ "${QA_FIXTURE}" != *'"charges":false'* || "${QA_FIXTURE}" != *'"files_with_hmrc":false'* ]]; then
  echo "deployment did not enable the non-charging QA entitlement fixture" >&2
  exit 1
fi
printf '%s\n' "${QA_FIXTURE}"

echo "== verify one replica and durable /data topology"
bash scripts/verify-azure-topology.sh

echo "== prove persistence across a replica restart"
DURABILITY_PROBE_VALUE="${SOURCE_SHA}" node scripts/verify-durability.mjs seed
CURRENT_REVISION="$(az containerapp show --resource-group "${RESOURCE_GROUP}" --name "${APP}" --query 'properties.latestReadyRevisionName' -o tsv)"
az containerapp revision restart --resource-group "${RESOURCE_GROUP}" --name "${APP}" --revision "${CURRENT_REVISION}" --only-show-errors -o none
for _ in $(seq 1 36); do
  if curl --silent --show-error --fail --max-time 15 "https://${SLUG}.sociobot.in/health" | grep -q "${SOURCE_SHA}"; then break; fi
  sleep 5
done
DURABILITY_PROBE_VALUE="${SOURCE_SHA}" node scripts/verify-durability.mjs check

echo "== prove persistence across a revision replacement"
# A normal single-revision rollout can overlap an old and a new pod while the
# old one drains. That is unsafe for the one-writer SQLite service even with
# durable Azure Files, so use the same explicit stop-before-start hand-off.
stop_revision_for_snapshot_handoff "${CURRENT_REVISION}"
az containerapp update --resource-group "${RESOURCE_GROUP}" --name "${APP}" \
  --set-env-vars "PERSISTENCE_PROBE_SHA=${SOURCE_SHA}" --only-show-errors -o none
for _ in $(seq 1 36); do
  NEW_REVISION="$(az containerapp show --resource-group "${RESOURCE_GROUP}" --name "${APP}" --query 'properties.latestReadyRevisionName' -o tsv)"
  if [[ "${NEW_REVISION}" != "${CURRENT_REVISION}" ]] \
    && curl --silent --show-error --fail --max-time 15 "https://${SLUG}.sociobot.in/health" | grep -q "${SOURCE_SHA}"; then break; fi
  sleep 5
done
DURABILITY_PROBE_VALUE="${SOURCE_SHA}" node scripts/verify-durability.mjs check
bash scripts/verify-azure-topology.sh

echo "== handoff-only verification (identity, non-filing capability, paid safe fixture, and topology)"
EXPECTED_BUILD_SHA="${SOURCE_SHA}" VERIFY_AZURE_TOPOLOGY=1 REQUIRE_HANDOFF_ONLY=1 node scripts/verify-live.mjs
