#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
TEMPLATE_PATH="${TASK_TEMPLATE_PATH:-$ROOT_DIR/ecs/task-definition.json}"
OUTPUT_PATH="${1:-}"

: "${IMAGE_REPOSITORY:=ghcr.io/akaich00/r-image-magic}"
: "${IMAGE_TAG:=latest}"
: "${AWS_REGION:=us-east-1}"
: "${ECS_TASK_FAMILY:=r-image-magic}"
: "${SERVICE_NAME:=r-image-magic}"
: "${CONTAINER_NAME:=r-image-magic}"
: "${CONTAINER_PORT:=8080}"
: "${TASK_CPU:=1024}"
: "${TASK_MEMORY:=2048}"
: "${AWS_LOG_GROUP:=/ecs/r-image-magic}"
: "${PRICING_URL:=https://r-image-magic.com/pricing}"
: "${DEFAULT_R2_BUCKET:=r-image-magic-pod-assets}"
: "${GHCR_SECRET_ARN:=__GHCR_SECRET_ARN__}"
: "${ECS_EXECUTION_ROLE_ARN:=__EXECUTION_ROLE_ARN__}"
: "${ECS_TASK_ROLE_ARN:=__TASK_ROLE_ARN__}"
: "${DATABASE_URL_SECRET_ARN:=__DATABASE_URL_SECRET_ARN__}"
: "${R2_ACCOUNT_ID_SECRET_ARN:=__R2_ACCOUNT_ID_SECRET_ARN__}"
: "${R2_ACCESS_KEY_ID_SECRET_ARN:=__R2_ACCESS_KEY_ID_SECRET_ARN__}"
: "${R2_SECRET_ACCESS_KEY_SECRET_ARN:=__R2_SECRET_ACCESS_KEY_SECRET_ARN__}"
: "${PRINTFUL_ACCESS_TOKEN_SECRET_ARN:=}"
: "${PRINTIFY_API_KEY_SECRET_ARN:=}"
: "${GELATO_API_KEY_SECRET_ARN:=}"
: "${SPOD_ACCESS_TOKEN_SECRET_ARN:=}"
: "${GOOTEN_RECIPE_ID_SECRET_ARN:=}"

IMAGE="${IMAGE_REPOSITORY}:${IMAGE_TAG}"

rendered_json="$({
  jq \
    --arg IMAGE "$IMAGE" \
    --arg TASK_FAMILY "$ECS_TASK_FAMILY" \
    --arg CONTAINER_NAME "$CONTAINER_NAME" \
    --arg SERVICE_NAME "$SERVICE_NAME" \
    --arg CONTAINER_PORT "$CONTAINER_PORT" \
    --arg TASK_CPU "$TASK_CPU" \
    --arg TASK_MEMORY "$TASK_MEMORY" \
    --arg EXECUTION_ROLE_ARN "$ECS_EXECUTION_ROLE_ARN" \
    --arg TASK_ROLE_ARN "$ECS_TASK_ROLE_ARN" \
    --arg GHCR_SECRET_ARN "$GHCR_SECRET_ARN" \
    --arg AWS_REGION "$AWS_REGION" \
    --arg AWS_LOG_GROUP "$AWS_LOG_GROUP" \
    --arg PRICING_URL "$PRICING_URL" \
    --arg DEFAULT_R2_BUCKET "$DEFAULT_R2_BUCKET" \
    --arg DATABASE_URL_SECRET_ARN "$DATABASE_URL_SECRET_ARN" \
    --arg R2_ACCOUNT_ID_SECRET_ARN "$R2_ACCOUNT_ID_SECRET_ARN" \
    --arg R2_ACCESS_KEY_ID_SECRET_ARN "$R2_ACCESS_KEY_ID_SECRET_ARN" \
    --arg R2_SECRET_ACCESS_KEY_SECRET_ARN "$R2_SECRET_ACCESS_KEY_SECRET_ARN" \
    --arg PRINTFUL_ACCESS_TOKEN_SECRET_ARN "$PRINTFUL_ACCESS_TOKEN_SECRET_ARN" \
    --arg PRINTIFY_API_KEY_SECRET_ARN "$PRINTIFY_API_KEY_SECRET_ARN" \
    --arg GELATO_API_KEY_SECRET_ARN "$GELATO_API_KEY_SECRET_ARN" \
    --arg SPOD_ACCESS_TOKEN_SECRET_ARN "$SPOD_ACCESS_TOKEN_SECRET_ARN" \
    --arg GOOTEN_RECIPE_ID_SECRET_ARN "$GOOTEN_RECIPE_ID_SECRET_ARN" \
    '
      .family = $TASK_FAMILY |
      .cpu = $TASK_CPU |
      .memory = $TASK_MEMORY |
      .executionRoleArn = $EXECUTION_ROLE_ARN |
      .taskRoleArn = $TASK_ROLE_ARN |
      .containerDefinitions[0].name = $CONTAINER_NAME |
      .containerDefinitions[0].image = $IMAGE |
      .containerDefinitions[0].repositoryCredentials.credentialsParameter = $GHCR_SECRET_ARN |
      .containerDefinitions[0].portMappings[0].containerPort = ($CONTAINER_PORT | tonumber) |
      .containerDefinitions[0].portMappings[0].hostPort = ($CONTAINER_PORT | tonumber) |
      .containerDefinitions[0].healthCheck.command[1] = ("curl -f http://localhost:" + $CONTAINER_PORT + "/health || exit 1") |
      .containerDefinitions[0].logConfiguration.options["awslogs-group"] = $AWS_LOG_GROUP |
      .containerDefinitions[0].logConfiguration.options["awslogs-region"] = $AWS_REGION |
      .containerDefinitions[0].environment = (.containerDefinitions[0].environment | map(
        if .name == "MOCKUP_SERVICE__NAME" then .value = $SERVICE_NAME
        elif .name == "MOCKUP_SERVICE__PRICING_URL" then .value = $PRICING_URL
        elif .name == "MOCKUP_SERVICE__R2_BUCKET_DEFAULT" then .value = $DEFAULT_R2_BUCKET
        else . end
      )) |
      .containerDefinitions[0].secrets = (
        .containerDefinitions[0].secrets
        | map(
            if .name == "DATABASE_URL" or .name == "MOCKUP_DATABASE__URL" then .valueFrom = $DATABASE_URL_SECRET_ARN
            elif .name == "R2_ACCOUNT_ID" then .valueFrom = $R2_ACCOUNT_ID_SECRET_ARN
            elif .name == "R2_ACCESS_KEY_ID" then .valueFrom = $R2_ACCESS_KEY_ID_SECRET_ARN
            elif .name == "R2_SECRET_ACCESS_KEY" then .valueFrom = $R2_SECRET_ACCESS_KEY_SECRET_ARN
            elif .name == "PRINTFUL_ACCESS_TOKEN" then .valueFrom = $PRINTFUL_ACCESS_TOKEN_SECRET_ARN
            elif .name == "PRINTIFY_API_KEY" then .valueFrom = $PRINTIFY_API_KEY_SECRET_ARN
            elif .name == "GELATO_API_KEY" then .valueFrom = $GELATO_API_KEY_SECRET_ARN
            elif .name == "SPOD_ACCESS_TOKEN" then .valueFrom = $SPOD_ACCESS_TOKEN_SECRET_ARN
            elif .name == "GOOTEN_RECIPE_ID" then .valueFrom = $GOOTEN_RECIPE_ID_SECRET_ARN
            else . end
          )
        | map(select(.valueFrom != null and .valueFrom != "" and (.valueFrom | startswith("__") | not)))
      )
    ' \
    "$TEMPLATE_PATH"
} )"

if [[ -n "$OUTPUT_PATH" ]]; then
  printf '%s\n' "$rendered_json" > "$OUTPUT_PATH"
  printf 'Wrote %s\n' "$OUTPUT_PATH"
else
  printf '%s\n' "$rendered_json"
fi
