#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  ./scripts/bootstrap-aws-secrets.sh <staging|production> [--apply]

Behavior:
  - Without --apply, prints the aws commands it would run.
  - With --apply, creates any missing Secrets Manager secrets with placeholder values.
  - Existing secrets are left untouched.

Optional environment overrides:
  SERVICE_SLUG
  AWS_REGION
  AWS_ACCOUNT_ID
  DATABASE_URL_SECRET_NAME
  R2_ACCOUNT_ID_SECRET_NAME
  R2_ACCESS_KEY_ID_SECRET_NAME
  R2_SECRET_ACCESS_KEY_SECRET_NAME
  PRINTFUL_ACCESS_TOKEN_SECRET_NAME
  PRINTIFY_API_KEY_SECRET_NAME
  GELATO_API_KEY_SECRET_NAME
  SPOD_ACCESS_TOKEN_SECRET_NAME
  GOOTEN_RECIPE_ID_SECRET_NAME
EOF
}

if [[ $# -lt 1 ]]; then
  usage
  exit 1
fi

environment_name="$1"
shift

if [[ "$environment_name" != "staging" && "$environment_name" != "production" ]]; then
  echo "Environment must be 'staging' or 'production'" >&2
  exit 1
fi

apply_changes="false"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --apply)
      apply_changes="true"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage
      exit 1
      ;;
  esac
done

if ! command -v aws >/dev/null 2>&1; then
  echo "aws CLI is required" >&2
  exit 1
fi

: "${SERVICE_SLUG:=r-image-magic}"
: "${AWS_REGION:=us-east-1}"
: "${AWS_ACCOUNT_ID:=123456789012}"

: "${DATABASE_URL_SECRET_NAME:=${SERVICE_SLUG}/${environment_name}/database-url}"
: "${R2_ACCOUNT_ID_SECRET_NAME:=${SERVICE_SLUG}/${environment_name}/r2-account-id}"
: "${R2_ACCESS_KEY_ID_SECRET_NAME:=${SERVICE_SLUG}/${environment_name}/r2-access-key-id}"
: "${R2_SECRET_ACCESS_KEY_SECRET_NAME:=${SERVICE_SLUG}/${environment_name}/r2-secret-access-key}"
: "${PRINTFUL_ACCESS_TOKEN_SECRET_NAME:=${SERVICE_SLUG}/${environment_name}/printful-access-token}"
: "${PRINTIFY_API_KEY_SECRET_NAME:=${SERVICE_SLUG}/${environment_name}/printify-api-key}"
: "${GELATO_API_KEY_SECRET_NAME:=${SERVICE_SLUG}/${environment_name}/gelato-api-key}"
: "${SPOD_ACCESS_TOKEN_SECRET_NAME:=${SERVICE_SLUG}/${environment_name}/spod-access-token}"
: "${GOOTEN_RECIPE_ID_SECRET_NAME:=${SERVICE_SLUG}/${environment_name}/gooten-recipe-id}"

ensure_secret() {
  local secret_name="$1"
  local placeholder_value="$2"

  if aws secretsmanager describe-secret --secret-id "$secret_name" --region "$AWS_REGION" >/dev/null 2>&1; then
    echo "Exists: $secret_name"
    return 0
  fi

  if [[ "$apply_changes" == "true" ]]; then
    aws secretsmanager create-secret \
      --name "$secret_name" \
      --description "${SERVICE_SLUG} ${environment_name} secret placeholder" \
      --secret-string "$placeholder_value" \
      --region "$AWS_REGION"
  else
    printf '%q ' aws secretsmanager create-secret \
      --name "$secret_name" \
      --description "${SERVICE_SLUG} ${environment_name} secret placeholder" \
      --secret-string "$placeholder_value" \
      --region "$AWS_REGION"
    printf '\n'
  fi
}

ensure_secret "$DATABASE_URL_SECRET_NAME" 'REPLACE_ME_DATABASE_URL'
ensure_secret "$R2_ACCOUNT_ID_SECRET_NAME" 'REPLACE_ME_R2_ACCOUNT_ID'
ensure_secret "$R2_ACCESS_KEY_ID_SECRET_NAME" 'REPLACE_ME_R2_ACCESS_KEY_ID'
ensure_secret "$R2_SECRET_ACCESS_KEY_SECRET_NAME" 'REPLACE_ME_R2_SECRET_ACCESS_KEY'
ensure_secret "$PRINTFUL_ACCESS_TOKEN_SECRET_NAME" 'REPLACE_ME_PRINTFUL_ACCESS_TOKEN'
ensure_secret "$PRINTIFY_API_KEY_SECRET_NAME" 'REPLACE_ME_PRINTIFY_API_KEY'
ensure_secret "$GELATO_API_KEY_SECRET_NAME" 'REPLACE_ME_GELATO_API_KEY'
ensure_secret "$SPOD_ACCESS_TOKEN_SECRET_NAME" 'REPLACE_ME_SPOD_ACCESS_TOKEN'
ensure_secret "$GOOTEN_RECIPE_ID_SECRET_NAME" 'REPLACE_ME_GOOTEN_RECIPE_ID'

if [[ "$apply_changes" == "false" ]]; then
  echo
  echo "Dry run only. Re-run with --apply to create missing secrets."
fi