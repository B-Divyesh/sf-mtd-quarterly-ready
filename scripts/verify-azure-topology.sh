#!/usr/bin/env bash
set -euo pipefail

RESOURCE_GROUP="${AZURE_RESOURCE_GROUP:-sociobot}"
APP="${AZURE_CONTAINER_APP:-sf-mtd-quarterly-ready}"

MIN_REPLICAS="$(az containerapp show --resource-group "${RESOURCE_GROUP}" --name "${APP}" --query 'properties.template.scale.minReplicas' -o tsv)"
MAX_REPLICAS="$(az containerapp show --resource-group "${RESOURCE_GROUP}" --name "${APP}" --query 'properties.template.scale.maxReplicas' -o tsv)"
DATA_VOLUME="$(az containerapp show --resource-group "${RESOURCE_GROUP}" --name "${APP}" --query "properties.template.containers[0].volumeMounts[?mountPath=='/data'].volumeName | [0]" -o tsv)"
STORAGE_TYPE="$(az containerapp show --resource-group "${RESOURCE_GROUP}" --name "${APP}" --query "properties.template.volumes[?name=='${DATA_VOLUME}'].storageType | [0]" -o tsv)"
RUNNING_REPLICAS="$(az containerapp replica list --resource-group "${RESOURCE_GROUP}" --name "${APP}" --query "length([?properties.runningState=='Running'])" -o tsv)"

if [[ "${MIN_REPLICAS}" != "1" || "${MAX_REPLICAS}" != "1" ]]; then
  echo "topology check failed: expected minReplicas=1 and maxReplicas=1" >&2
  exit 1
fi
if [[ -z "${DATA_VOLUME}" || "${STORAGE_TYPE}" != "AzureFile" ]]; then
  echo "topology check failed: /data is not backed by an AzureFile volume" >&2
  exit 1
fi
if [[ "${RUNNING_REPLICAS}" != "1" ]]; then
  echo "topology check failed: expected exactly one running replica" >&2
  exit 1
fi

printf '{"min_replicas":1,"max_replicas":1,"running_replicas":1,"data_mount":"AzureFile"}\n'
