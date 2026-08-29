#!/usr/bin/env bash
# Deploy Quarterly Ready with the state guarantees its SQLite backend requires.
#
# Default mode is an approved HMRC provider release. It fails closed unless the
# provider credentials have already been supplied through Key Vault. The only
# alternative is the explicit handoff-only mode, which deploys the useful
# records/CSV/handoff product without pretending it can file a return.
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
DEPLOYMENT_MODE="${DEPLOYMENT_MODE:-approved}"
SOURCE_SHA="$(git rev-parse HEAD)"
IMAGE_TAG="${APP}:${SOURCE_SHA:0:12}"
IMAGE="${REGISTRY}.azurecr.io/${IMAGE_TAG}"
RESOURCE_ID="/subscriptions/${SUBSCRIPTION}/resourceGroups/${RESOURCE_GROUP}"
ENVIRONMENT_ID="${RESOURCE_ID}/providers/Microsoft.App/managedEnvironments/${ENVIRONMENT}"
IDENTITY_ID="${RESOURCE_ID}/providers/Microsoft.ManagedIdentity/userAssignedIdentities/factory-worker-identity"
CERTIFICATE_ID="${ENVIRONMENT_ID}/managedCertificates/cert-${SLUG}"
KEY_VAULT="sociobot-keyvault1"
HMRC_URL_SECRET="mtd-quarterly-ready-approved-hmrc-url"
HMRC_TOKEN_SECRET="mtd-quarterly-ready-approved-hmrc-token"
APP_URL="https://management.azure.com${RESOURCE_ID}/providers/Microsoft.App/containerApps/${APP}?api-version=2024-03-01"

HMRC_SECRET_CONFIG="[]"
HMRC_ENV_CONFIG=""
EXPECTED_HMRC_MODE="not_configured"
case "${DEPLOYMENT_MODE}" in
  approved)
    # Query only secret metadata. Never read, log, or embed a credential.
    if ! az keyvault secret show --vault-name "${KEY_VAULT}" --name "${HMRC_URL_SECRET}" --query id -o none >/dev/null 2>&1 \
      || ! az keyvault secret show --vault-name "${KEY_VAULT}" --name "${HMRC_TOKEN_SECRET}" --query id -o none >/dev/null 2>&1; then
      echo "missing approved HMRC integration secret references; refusing a release deployment" >&2
      echo "expected Key Vault secrets: ${HMRC_URL_SECRET} and ${HMRC_TOKEN_SECRET}" >&2
      exit 1
    fi
    HMRC_SECRET_CONFIG="[{\"name\":\"hmrc-approved-url\",\"keyVaultUrl\":\"https://${KEY_VAULT}.vault.azure.net/secrets/${HMRC_URL_SECRET}\",\"identity\":\"${IDENTITY_ID}\"},{\"name\":\"hmrc-approved-token\",\"keyVaultUrl\":\"https://${KEY_VAULT}.vault.azure.net/secrets/${HMRC_TOKEN_SECRET}\",\"identity\":\"${IDENTITY_ID}\"}]"
    HMRC_ENV_CONFIG=', {"name":"HMRC_INTEGRATION_URL","secretRef":"hmrc-approved-url"}, {"name":"HMRC_INTEGRATION_TOKEN","secretRef":"hmrc-approved-token"}, {"name":"HMRC_INTEGRATION_MODE","value":"approved_provider"}'
    EXPECTED_HMRC_MODE="approved_provider"
    echo "approved HMRC provider secret references found; binding them without reading values"
    ;;
  handoff-only)
    echo "handoff-only deployment selected: direct HMRC submission remains unavailable and is not advertised"
    ;;
  *)
    echo "DEPLOYMENT_MODE must be approved or handoff-only" >&2
    exit 2
    ;;
esac

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
# SQLite and the built-in limiter are serial process resources. A one-replica
# hand-off prevents divergent records or multiplied per-client allowances.
READY_REVISION="$(az containerapp show --resource-group "${RESOURCE_GROUP}" --name "${APP}" --query 'properties.latestReadyRevisionName' -o tsv 2>/dev/null || true)"
if [[ -n "${READY_REVISION}" ]]; then
  az containerapp revision deactivate --resource-group "${RESOURCE_GROUP}" --name "${APP}" --revision "${READY_REVISION}" --only-show-errors -o none || true
  STOPPED=0
  for _ in $(seq 1 36); do
    RUNNING="$(az containerapp replica list --resource-group "${RESOURCE_GROUP}" --name "${APP}" --revision "${READY_REVISION}" --query "length([?properties.runningState=='Running'])" -o tsv 2>/dev/null || echo 0)"
    if [[ "${RUNNING}" == "0" ]]; then
      STOPPED=1
      break
    fi
    sleep 5
  done
  if [[ "${STOPPED}" != "1" ]]; then
    echo "previous revision did not stop; refusing concurrent SQLite snapshot writers" >&2
    exit 1
  fi
fi
az rest --method patch --url "${APP_URL}" --headers "Content-Type=application/json" --body "$(cat <<JSON
{
  "properties": {
    "configuration": {
      "activeRevisionsMode": "Single",
      "secrets": ${HMRC_SECRET_CONFIG},
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
        "env": [{"name": "PORT", "value": "${PORT}"}, {"name": "SAFE_QA_FIXTURES", "value": "1"}${HMRC_ENV_CONFIG}],
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

if [[ "${DEPLOYMENT_MODE}" == "approved" ]]; then
  echo "== release verification (identity, approved HMRC capability, paid safe fixture, and topology)"
  EXPECTED_BUILD_SHA="${SOURCE_SHA}" VERIFY_AZURE_TOPOLOGY=1 REQUIRE_APPROVED_HMRC=1 node scripts/verify-live.mjs
else
  echo "== handoff-only verification (identity, paid safe fixture, and topology)"
  EXPECTED_BUILD_SHA="${SOURCE_SHA}" VERIFY_AZURE_TOPOLOGY=1 node scripts/verify-live.mjs
fi
