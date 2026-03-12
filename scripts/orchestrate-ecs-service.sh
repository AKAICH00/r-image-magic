#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  ./scripts/orchestrate-ecs-service.sh <staging|production> [options]

Options:
  --service-slug <name>        Service slug to bootstrap (default: r-image-magic)
  --repo <owner/repo>          GitHub repository for environment variable setup
  --base-domain <domain>       Base domain used by GitHub env bootstrap
  --aws-account-id <id>        AWS account ID used by bootstrap scripts
  --render-output <file>       Write rendered task definition JSON to this path
  --compare-with <service>     Compare the target ECS service against another service after setup
  --cluster <name>             ECS cluster for comparison (default from env/bootstrap)
  --region <aws-region>        AWS region override
  --apply                      Apply GitHub/AWS bootstrap changes instead of dry-run
  --skip-secrets               Skip Secrets Manager bootstrap step
  --skip-github                Skip GitHub Environment bootstrap step
  --skip-render                Skip task-definition render step
  --skip-compare               Skip ECS comparison step

Notes:
  - Comparison only runs when --compare-with is provided.
  - Without --apply, bootstrap steps print commands instead of mutating AWS/GitHub.
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

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
SCRIPTS_DIR="$ROOT_DIR/scripts"

service_slug="r-image-magic"
repo=""
base_domain="rimagemagic.com"
aws_account_id="123456789012"
render_output=""
compare_with=""
cluster=""
region="${AWS_REGION:-us-east-1}"
apply_changes="false"
skip_secrets="false"
skip_github="false"
skip_render="false"
skip_compare="false"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --service-slug)
      service_slug="$2"
      shift 2
      ;;
    --repo)
      repo="$2"
      shift 2
      ;;
    --base-domain)
      base_domain="$2"
      shift 2
      ;;
    --aws-account-id)
      aws_account_id="$2"
      shift 2
      ;;
    --render-output)
      render_output="$2"
      shift 2
      ;;
    --compare-with)
      compare_with="$2"
      shift 2
      ;;
    --cluster)
      cluster="$2"
      shift 2
      ;;
    --region)
      region="$2"
      shift 2
      ;;
    --apply)
      apply_changes="true"
      shift
      ;;
    --skip-secrets)
      skip_secrets="true"
      shift
      ;;
    --skip-github)
      skip_github="true"
      shift
      ;;
    --skip-render)
      skip_render="true"
      shift
      ;;
    --skip-compare)
      skip_compare="true"
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

export SERVICE_SLUG="$service_slug"
export BASE_DOMAIN="$base_domain"
export AWS_ACCOUNT_ID="$aws_account_id"
export AWS_REGION="$region"

if [[ "$environment_name" == "staging" ]]; then
  : "${ECS_SERVICE:=${service_slug}-staging}"
  : "${ECS_TASK_FAMILY:=${service_slug}-staging}"
  : "${SERVICE_NAME:=${service_slug}-staging}"
  : "${AWS_LOG_GROUP:=/ecs/${service_slug}-staging}"
else
  : "${ECS_SERVICE:=${service_slug}}"
  : "${ECS_TASK_FAMILY:=${service_slug}}"
  : "${SERVICE_NAME:=${service_slug}}"
  : "${AWS_LOG_GROUP:=/ecs/${service_slug}}"
fi

if [[ -z "$cluster" ]]; then
  cluster="${ECS_CLUSTER:-akaich00-cluster}"
fi
export ECS_CLUSTER="$cluster"
export ECS_SERVICE
export ECS_TASK_FAMILY
export SERVICE_NAME
export AWS_LOG_GROUP

run_step() {
  local description="$1"
  shift
  echo
  echo ">>> $description"
  "$@"
}

if [[ "$skip_secrets" != "true" ]]; then
  secrets_args=("$environment_name")
  if [[ "$apply_changes" == "true" ]]; then
    secrets_args+=(--apply)
  fi
  run_step "Bootstrap AWS Secrets Manager placeholders" "$SCRIPTS_DIR/bootstrap-aws-secrets.sh" "${secrets_args[@]}"
fi

if [[ "$skip_github" != "true" ]]; then
  github_args=("$environment_name")
  if [[ -n "$repo" ]]; then
    github_args+=(--repo "$repo")
  fi
  if [[ "$apply_changes" == "true" ]]; then
    github_args+=(--apply)
  fi
  run_step "Bootstrap GitHub Environment variables" "$SCRIPTS_DIR/bootstrap-github-environment.sh" "${github_args[@]}"
fi

if [[ "$skip_render" != "true" ]]; then
  render_args=()
  if [[ -n "$render_output" ]]; then
    render_args+=("$render_output")
  fi
  export IMAGE_TAG="${IMAGE_TAG:-latest}"
  run_step "Render ECS task definition" "$SCRIPTS_DIR/render-ecs-task-def.sh" "${render_args[@]}"
fi

if [[ "$skip_compare" != "true" && -n "$compare_with" ]]; then
  run_step "Compare ECS services for isolation" "$SCRIPTS_DIR/compare-ecs-services.sh" "$cluster" "$compare_with" "$ECS_SERVICE" --region "$region"
fi

echo
echo "Completed orchestration for service '$service_slug' in environment '$environment_name'."