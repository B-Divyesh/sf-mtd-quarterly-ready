#!/usr/bin/env bash
# Provision the non-filing HMRC sandbox configuration without writing secret
# values to this repository or command output. Existing credentials are kept.
set -euo pipefail

KEY_VAULT="${AZURE_KEY_VAULT:-sociobot-keyvault1}"
URL_SECRET="mtd-quarterly-ready-hmrc-integration-url"
TOKEN_SECRET="mtd-quarterly-ready-hmrc-integration-token"
HMRC_TEST_URL="https://test-api.service.hmrc.gov.uk/hello/world"
VAULT_ID="$(az keyvault show --name "${KEY_VAULT}" --query id -o tsv)"

put_secret() {
  local name="$1"
  local value="$2"
  local content_type="$3"
  az rest --method put \
    --url "https://management.azure.com${VAULT_ID}/secrets/${name}?api-version=2023-07-01" \
    --headers "Content-Type=application/json" \
    --body "{\"properties\":{\"value\":\"${value}\",\"contentType\":\"${content_type}\"}}" \
    --only-show-errors -o none
}

if ! az keyvault secret show --vault-name "${KEY_VAULT}" --name "${URL_SECRET}" --query id -o none >/dev/null 2>&1; then
  put_secret "${URL_SECRET}" "${HMRC_TEST_URL}" "Quarterly Ready HMRC non-filing sandbox endpoint"
fi

if ! az keyvault secret show --vault-name "${KEY_VAULT}" --name "${TOKEN_SECRET}" --query id -o none >/dev/null 2>&1; then
  SANDBOX_ATTESTATION="$(openssl rand -hex 32)"
  put_secret "${TOKEN_SECRET}" "${SANDBOX_ATTESTATION}" "Quarterly Ready non-filing sandbox attestation"
  unset SANDBOX_ATTESTATION
fi

for secret in "${URL_SECRET}" "${TOKEN_SECRET}"; do
  az keyvault secret show --vault-name "${KEY_VAULT}" --name "${secret}" \
    --query '{name:name,enabled:attributes.enabled}' -o json
done
