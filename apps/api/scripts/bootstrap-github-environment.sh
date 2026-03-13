#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  ./scripts/bootstrap-github-environment.sh <staging|production> [--repo owner/repo] [--apply]

Behavior:
  - Without --apply, prints the gh commands it would run.
  - With --apply, creates the GitHub Environment if needed and sets environment-scoped variables.

Optional environment overrides:
  SERVICE_SLUG
  BASE_DOMAIN
  AWS_ACCOUNT_ID
  AWS_REGION
  ECS_CLUSTER
  ECS_SERVICE
  ECS_TASK_FAMILY
  SERVICE_NAME
  CONTAINER_NAME
  CONTAINER_PORT
  TASK_CPU
  TASK_MEMORY
  AWS_LOG_GROUP
  PRICING_URL
  DEFAULT_R2_BUCKET
  GHCR_SECRET_NAME
  ECS_EXECUTION_ROLE_ARN
  ECS_TASK_ROLE_ARN
  MOCKUP_DATABASE_URL_SECRET_ARN
  R2_ACCOUNT_ID_SECRET_ARN
  R2_ACCESS_KEY_ID_SECRET_ARN
  R2_SECRET_ACCESS_KEY_SECRET_ARN
  PRINTFUL_ACCESS_TOKEN_SECRET_ARN
  PRINTIFY_API_KEY_SECRET_ARN
  GELATO_API_KEY_SECRET_ARN
  SPOD_ACCESS_TOKEN_SECRET_ARN
  GOOTEN_RECIPE_ID_SECRET_ARN
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

repo=""
apply_changes="false"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --repo)
      repo="$2"
      shift 2
      ;;
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

if ! command -v gh >/dev/null 2>&1; then
  echo "gh CLI is required" >&2
  exit 1
fi

if [[ -z "$repo" ]]; then
  repo="$(gh repo view --json nameWithOwner -q .nameWithOwner 2>/dev/null || true)"
fi

if [[ -z "$repo" ]]; then
  echo "Could not determine repository. Pass --repo owner/repo." >&2
  exit 1
fi

: "${SERVICE_SLUG:=r-image-magic}"
: "${BASE_DOMAIN:=rimagemagic.com}"
: "${AWS_ACCOUNT_ID:=123456789012}"

if [[ "$environment_name" == "staging" ]]; then
  : "${AWS_REGION:=us-east-1}"
  : "${ECS_CLUSTER:=akaich00-cluster}"
  : "${ECS_SERVICE:=${SERVICE_SLUG}-staging}"
  : "${ECS_TASK_FAMILY:=${SERVICE_SLUG}-staging}"
  : "${SERVICE_NAME:=${SERVICE_SLUG}-staging}"
  : "${CONTAINER_NAME:=${SERVICE_SLUG}}"
  : "${CONTAINER_PORT:=8080}"
  : "${TASK_CPU:=1024}"
  : "${TASK_MEMORY:=2048}"
  : "${AWS_LOG_GROUP:=/ecs/${SERVICE_SLUG}-staging}"
  : "${PRICING_URL:=https://staging.${BASE_DOMAIN}/pricing}"
  : "${DEFAULT_R2_BUCKET:=${SERVICE_SLUG}-staging-assets}"
  : "${GHCR_SECRET_NAME:=${SERVICE_SLUG}/staging/ghcr-credentials}"
  : "${ECS_EXECUTION_ROLE_ARN:=arn:aws:iam::${AWS_ACCOUNT_ID}:role/ecsTaskExecutionRole}"
  : "${ECS_TASK_ROLE_ARN:=arn:aws:iam::${AWS_ACCOUNT_ID}:role/${SERVICE_SLUG}StagingTaskRole}"
  : "${MOCKUP_DATABASE_URL_SECRET_ARN:=arn:aws:secretsmanager:${AWS_REGION}:${AWS_ACCOUNT_ID}:secret:${SERVICE_SLUG}/staging/database-url}"
  : "${R2_ACCOUNT_ID_SECRET_ARN:=arn:aws:secretsmanager:${AWS_REGION}:${AWS_ACCOUNT_ID}:secret:${SERVICE_SLUG}/staging/r2-account-id}"
  : "${R2_ACCESS_KEY_ID_SECRET_ARN:=arn:aws:secretsmanager:${AWS_REGION}:${AWS_ACCOUNT_ID}:secret:${SERVICE_SLUG}/staging/r2-access-key-id}"
  : "${R2_SECRET_ACCESS_KEY_SECRET_ARN:=arn:aws:secretsmanager:${AWS_REGION}:${AWS_ACCOUNT_ID}:secret:${SERVICE_SLUG}/staging/r2-secret-access-key}"
else
  : "${AWS_REGION:=us-east-1}"
  : "${ECS_CLUSTER:=akaich00-cluster}"
  : "${ECS_SERVICE:=${SERVICE_SLUG}}"
  : "${ECS_TASK_FAMILY:=${SERVICE_SLUG}}"
  : "${SERVICE_NAME:=${SERVICE_SLUG}}"
  : "${CONTAINER_NAME:=${SERVICE_SLUG}}"
  : "${CONTAINER_PORT:=8080}"
  : "${TASK_CPU:=2048}"
  : "${TASK_MEMORY:=4096}"
  : "${AWS_LOG_GROUP:=/ecs/${SERVICE_SLUG}}"
  : "${PRICING_URL:=https://${BASE_DOMAIN}/pricing}"
  : "${DEFAULT_R2_BUCKET:=${SERVICE_SLUG}-pod-assets}"
  : "${GHCR_SECRET_NAME:=${SERVICE_SLUG}/production/ghcr-credentials}"
  : "${ECS_EXECUTION_ROLE_ARN:=arn:aws:iam::${AWS_ACCOUNT_ID}:role/ecsTaskExecutionRole}"
  : "${ECS_TASK_ROLE_ARN:=arn:aws:iam::${AWS_ACCOUNT_ID}:role/${SERVICE_SLUG}TaskRole}"
  : "${MOCKUP_DATABASE_URL_SECRET_ARN:=arn:aws:secretsmanager:${AWS_REGION}:${AWS_ACCOUNT_ID}:secret:${SERVICE_SLUG}/production/database-url}"
  : "${R2_ACCOUNT_ID_SECRET_ARN:=arn:aws:secretsmanager:${AWS_REGION}:${AWS_ACCOUNT_ID}:secret:${SERVICE_SLUG}/production/r2-account-id}"
  : "${R2_ACCESS_KEY_ID_SECRET_ARN:=arn:aws:secretsmanager:${AWS_REGION}:${AWS_ACCOUNT_ID}:secret:${SERVICE_SLUG}/production/r2-access-key-id}"
  : "${R2_SECRET_ACCESS_KEY_SECRET_ARN:=arn:aws:secretsmanager:${AWS_REGION}:${AWS_ACCOUNT_ID}:secret:${SERVICE_SLUG}/production/r2-secret-access-key}"
fi

: "${PRINTFUL_ACCESS_TOKEN_SECRET_ARN:=}"
: "${PRINTIFY_API_KEY_SECRET_ARN:=}"
: "${GELATO_API_KEY_SECRET_ARN:=}"
: "${SPOD_ACCESS_TOKEN_SECRET_ARN:=}"
: "${GOOTEN_RECIPE_ID_SECRET_ARN:=}"

variables=(
  AWS_REGION
  ECS_CLUSTER
  ECS_SERVICE
  ECS_TASK_FAMILY
  SERVICE_NAME
  CONTAINER_NAME
  CONTAINER_PORT
  TASK_CPU
  TASK_MEMORY
  AWS_LOG_GROUP
  PRICING_URL
  DEFAULT_R2_BUCKET
  GHCR_SECRET_NAME
  ECS_EXECUTION_ROLE_ARN
  ECS_TASK_ROLE_ARN
  MOCKUP_DATABASE_URL_SECRET_ARN
  R2_ACCOUNT_ID_SECRET_ARN
  R2_ACCESS_KEY_ID_SECRET_ARN
  R2_SECRET_ACCESS_KEY_SECRET_ARN
  PRINTFUL_ACCESS_TOKEN_SECRET_ARN
  PRINTIFY_API_KEY_SECRET_ARN
  GELATO_API_KEY_SECRET_ARN
  SPOD_ACCESS_TOKEN_SECRET_ARN
  GOOTEN_RECIPE_ID_SECRET_ARN
)

run_or_print() {
  if [[ "$apply_changes" == "true" ]]; then
    "$@"
  else
    printf '%q ' "$@"
    printf '\n'
  fi
}

run_or_print gh api --method PUT "repos/${repo}/environments/${environment_name}"

for variable_name in "${variables[@]}"; do
  value="${!variable_name}"
  if [[ -z "$value" ]]; then
    echo "Skipping $variable_name (empty)"
    continue
  fi
  run_or_print gh variable set "$variable_name" --env "$environment_name" --repo "$repo" --body "$value"
done

if [[ "$apply_changes" == "false" ]]; then
  echo
  echo "Dry run only. Re-run with --apply to create/update the environment variables."
fi
