#!/usr/bin/env bash
set -euo pipefail

RESOURCE_GROUP="${AZURE_RESOURCE_GROUP:-sociobot}"
APP="${AZURE_CONTAINER_APP:-sf-mtd-quarterly-ready}"
ENVIRONMENT="${AZURE_CONTAINER_ENVIRONMENT:-factory-env}"
STORAGE_NAME="${AZURE_CONTAINER_STORAGE:-mtd-quarterly-ready-data-v3}"
STORAGE_ACCOUNT="${AZURE_STORAGE_ACCOUNT:-sociobotblob}"
FILE_SHARE="${AZURE_FILE_SHARE:-sf-mtd-quarterly-ready-data-v3}"

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

printf '{"active_revisions_mode":"Single","min_replicas":1,"max_replicas":1,"running_replicas":1,"data_mount":"AzureFile","storage_name":"%s","file_share":"%s"}\n' "${STORAGE_NAME}" "${FILE_SHARE}"
