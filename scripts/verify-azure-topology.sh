#!/usr/bin/env bash
set -euo pipefail

RESOURCE_GROUP="${AZURE_RESOURCE_GROUP:-sociobot}"
APP="${AZURE_CONTAINER_APP:-sf-mtd-quarterly-ready}"
ENVIRONMENT="${AZURE_CONTAINER_ENVIRONMENT:-factory-env}"
STORAGE_NAME="${AZURE_CONTAINER_STORAGE:-mtd-quarterly-ready-data-v3}"
STORAGE_ACCOUNT="${AZURE_STORAGE_ACCOUNT:-sociobotblob}"
FILE_SHARE="${AZURE_FILE_SHARE:-sf-mtd-quarterly-ready-data-v3}"
KEY_VAULT="${AZURE_KEY_VAULT:-sociobot-keyvault1}"
SUBSCRIPTION="${AZURE_SUBSCRIPTION_ID:-283af945-693b-4a6e-b952-df928d0a18a9}"
IDENTITY_ID="/subscriptions/${SUBSCRIPTION}/resourceGroups/${RESOURCE_GROUP}/providers/Microsoft.ManagedIdentity/userAssignedIdentities/factory-worker-identity"

MIN_REPLICAS="$(az containerapp show --resource-group "${RESOURCE_GROUP}" --name "${APP}" --query 'properties.template.scale.minReplicas' -o tsv)"
MAX_REPLICAS="$(az containerapp show --resource-group "${RESOURCE_GROUP}" --name "${APP}" --query 'properties.template.scale.maxReplicas' -o tsv)"
ACTIVE_REVISIONS_MODE="$(az containerapp show --resource-group "${RESOURCE_GROUP}" --name "${APP}" --query 'properties.configuration.activeRevisionsMode' -o tsv)"
DATA_VOLUME="$(az containerapp show --resource-group "${RESOURCE_GROUP}" --name "${APP}" --query "properties.template.containers[0].volumeMounts[?mountPath=='/data'].volumeName | [0]" -o tsv)"
STORAGE_TYPE="$(az containerapp show --resource-group "${RESOURCE_GROUP}" --name "${APP}" --query "properties.template.volumes[?name=='${DATA_VOLUME}'].storageType | [0]" -o tsv)"
VOLUME_STORAGE_NAME="$(az containerapp show --resource-group "${RESOURCE_GROUP}" --name "${APP}" --query "properties.template.volumes[?name=='${DATA_VOLUME}'].storageName | [0]" -o tsv)"
RUNNING_REPLICAS="$(az containerapp replica list --resource-group "${RESOURCE_GROUP}" --name "${APP}" --query "length([?properties.runningState=='Running'])" -o tsv)"
ENV_STORAGE_ACCOUNT="$(az containerapp env storage show --resource-group "${RESOURCE_GROUP}" --name "${ENVIRONMENT}" --storage-name "${STORAGE_NAME}" --query 'properties.azureFile.accountName' -o tsv)"
ENV_FILE_SHARE="$(az containerapp env storage show --resource-group "${RESOURCE_GROUP}" --name "${ENVIRONMENT}" --storage-name "${STORAGE_NAME}" --query 'properties.azureFile.shareName' -o tsv)"
ENV_ACCESS_MODE="$(az containerapp env storage show --resource-group "${RESOURCE_GROUP}" --name "${ENVIRONMENT}" --storage-name "${STORAGE_NAME}" --query 'properties.azureFile.accessMode' -o tsv)"
HMRC_MODE="$(az containerapp show --resource-group "${RESOURCE_GROUP}" --name "${APP}" --query "properties.template.containers[0].env[?name=='HMRC_INTEGRATION_MODE'].value | [0]" -o tsv)"
HMRC_URL_REF="$(az containerapp show --resource-group "${RESOURCE_GROUP}" --name "${APP}" --query "properties.template.containers[0].env[?name=='HMRC_INTEGRATION_URL'].secretRef | [0]" -o tsv)"
HMRC_TOKEN_REF="$(az containerapp show --resource-group "${RESOURCE_GROUP}" --name "${APP}" --query "properties.template.containers[0].env[?name=='HMRC_INTEGRATION_TOKEN'].secretRef | [0]" -o tsv)"
HMRC_URL_VAULT="$(az containerapp show --resource-group "${RESOURCE_GROUP}" --name "${APP}" --query "properties.configuration.secrets[?name=='hmrc-integration-url'].keyVaultUrl | [0]" -o tsv)"
HMRC_TOKEN_VAULT="$(az containerapp show --resource-group "${RESOURCE_GROUP}" --name "${APP}" --query "properties.configuration.secrets[?name=='hmrc-integration-token'].keyVaultUrl | [0]" -o tsv)"
HMRC_URL_IDENTITY="$(az containerapp show --resource-group "${RESOURCE_GROUP}" --name "${APP}" --query "properties.configuration.secrets[?name=='hmrc-integration-url'].identity | [0]" -o tsv)"
HMRC_TOKEN_IDENTITY="$(az containerapp show --resource-group "${RESOURCE_GROUP}" --name "${APP}" --query "properties.configuration.secrets[?name=='hmrc-integration-token'].identity | [0]" -o tsv)"

if [[ "${MIN_REPLICAS}" != "1" || "${MAX_REPLICAS}" != "1" ]]; then
  echo "topology check failed: expected minReplicas=1 and maxReplicas=1" >&2
  exit 1
fi
if [[ "${ACTIVE_REVISIONS_MODE}" != "Single" ]]; then
  echo "topology check failed: expected activeRevisionsMode=Single for the SQLite volume hand-off" >&2
  exit 1
fi
if [[ -z "${DATA_VOLUME}" || "${STORAGE_TYPE}" != "AzureFile" ]]; then
  echo "topology check failed: /data is not backed by an AzureFile volume" >&2
  exit 1
fi
if [[ "${VOLUME_STORAGE_NAME}" != "${STORAGE_NAME}" ]]; then
  echo "topology check failed: /data must use environment storage ${STORAGE_NAME}" >&2
  exit 1
fi
if [[ "${ENV_STORAGE_ACCOUNT}" != "${STORAGE_ACCOUNT}" || "${ENV_FILE_SHARE}" != "${FILE_SHARE}" || "${ENV_ACCESS_MODE}" != "ReadWrite" ]]; then
  echo "topology check failed: environment storage is not the expected read-write Azure Files share" >&2
  exit 1
fi
if [[ "${RUNNING_REPLICAS}" != "1" ]]; then
  echo "topology check failed: expected exactly one running replica" >&2
  exit 1
fi
if [[ "${HMRC_MODE}" != "hmrc_sandbox_no_filing" || "${HMRC_URL_REF}" != "hmrc-integration-url" || "${HMRC_TOKEN_REF}" != "hmrc-integration-token" ]]; then
  echo "topology check failed: HMRC sandbox environment is not bound through Container App secret references" >&2
  exit 1
fi
if [[ "${HMRC_URL_VAULT}" != "https://${KEY_VAULT}.vault.azure.net/secrets/mtd-quarterly-ready-hmrc-integration-url" \
  || "${HMRC_TOKEN_VAULT}" != "https://${KEY_VAULT}.vault.azure.net/secrets/mtd-quarterly-ready-hmrc-integration-token" ]]; then
  echo "topology check failed: HMRC sandbox configuration is not backed by the expected Key Vault secrets" >&2
  exit 1
fi
if [[ "${HMRC_URL_IDENTITY}" != "${IDENTITY_ID}" || "${HMRC_TOKEN_IDENTITY}" != "${IDENTITY_ID}" ]]; then
  echo "topology check failed: HMRC Key Vault references do not use the factory managed identity" >&2
  exit 1
fi

printf '{"active_revisions_mode":"Single","min_replicas":1,"max_replicas":1,"running_replicas":1,"data_mount":"AzureFile","storage_name":"%s","file_share":"%s","hmrc_mode":"hmrc_sandbox_no_filing","hmrc_config":"key_vault_references"}\n' "${STORAGE_NAME}" "${FILE_SHARE}"
