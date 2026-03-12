#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  ./scripts/compare-ecs-services.sh <cluster> <service-a> <service-b> [--region us-east-1]

Purpose:
  Compare two ECS services and flag shared or isolated resources so sibling apps
  can reuse the same cluster safely without breaking each other.

Checks:
  - ECS service names
  - task definition families
  - task roles and execution roles
  - load balancer target groups
  - CloudWatch log groups
  - Secrets Manager references
  - security groups and subnets

Exit codes:
  0 = no isolation failures detected
  1 = one or more isolation failures detected
EOF
}

if [[ $# -lt 3 ]]; then
  usage
  exit 1
fi

cluster="$1"
service_a="$2"
service_b="$3"
shift 3

region="${AWS_REGION:-us-east-1}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --region)
      region="$2"
      shift 2
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

for required in aws jq; do
  if ! command -v "$required" >/dev/null 2>&1; then
    echo "$required is required" >&2
    exit 1
  fi
done

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

service_file="$tmp_dir/services.json"
aws ecs describe-services \
  --cluster "$cluster" \
  --services "$service_a" "$service_b" \
  --region "$region" \
  > "$service_file"

failures=0

status_line() {
  local level="$1"
  local message="$2"
  printf '[%s] %s\n' "$level" "$message"
}

unique_values() {
  jq -r '.[] | select(. != null and . != "")' | sort -u
}

extract_service_json() {
  local name="$1"
  jq --arg name "$name" '.services[] | select(.serviceName == $name)' "$service_file"
}

svc_a_json="$(extract_service_json "$service_a")"
svc_b_json="$(extract_service_json "$service_b")"

if [[ -z "$svc_a_json" || -z "$svc_b_json" ]]; then
  status_line FAIL "Could not load both ECS services from cluster $cluster"
  exit 1
fi

task_def_a_arn="$(printf '%s' "$svc_a_json" | jq -r '.taskDefinition')"
task_def_b_arn="$(printf '%s' "$svc_b_json" | jq -r '.taskDefinition')"

task_def_a_file="$tmp_dir/task-a.json"
task_def_b_file="$tmp_dir/task-b.json"

aws ecs describe-task-definition --task-definition "$task_def_a_arn" --region "$region" > "$task_def_a_file"
aws ecs describe-task-definition --task-definition "$task_def_b_arn" --region "$region" > "$task_def_b_file"

family_a="$(jq -r '.taskDefinition.family' "$task_def_a_file")"
family_b="$(jq -r '.taskDefinition.family' "$task_def_b_file")"

task_role_a="$(jq -r '.taskDefinition.taskRoleArn // ""' "$task_def_a_file")"
task_role_b="$(jq -r '.taskDefinition.taskRoleArn // ""' "$task_def_b_file")"
exec_role_a="$(jq -r '.taskDefinition.executionRoleArn // ""' "$task_def_a_file")"
exec_role_b="$(jq -r '.taskDefinition.executionRoleArn // ""' "$task_def_b_file")"

target_groups_a="$(printf '%s' "$svc_a_json" | jq -r '.loadBalancers[].targetGroupArn? // empty' | sort -u)"
target_groups_b="$(printf '%s' "$svc_b_json" | jq -r '.loadBalancers[].targetGroupArn? // empty' | sort -u)"

log_groups_a="$(jq -r '.taskDefinition.containerDefinitions[].logConfiguration.options["awslogs-group"]? // empty' "$task_def_a_file" | sort -u)"
log_groups_b="$(jq -r '.taskDefinition.containerDefinitions[].logConfiguration.options["awslogs-group"]? // empty' "$task_def_b_file" | sort -u)"

secret_refs_a="$(jq -r '.taskDefinition.containerDefinitions[].secrets[].valueFrom? // empty' "$task_def_a_file" | sort -u)"
secret_refs_b="$(jq -r '.taskDefinition.containerDefinitions[].secrets[].valueFrom? // empty' "$task_def_b_file" | sort -u)"

security_groups_a="$(printf '%s' "$svc_a_json" | jq -r '.networkConfiguration.awsvpcConfiguration.securityGroups[]? // empty' | sort -u)"
security_groups_b="$(printf '%s' "$svc_b_json" | jq -r '.networkConfiguration.awsvpcConfiguration.securityGroups[]? // empty' | sort -u)"

subnets_a="$(printf '%s' "$svc_a_json" | jq -r '.networkConfiguration.awsvpcConfiguration.subnets[]? // empty' | sort -u)"
subnets_b="$(printf '%s' "$svc_b_json" | jq -r '.networkConfiguration.awsvpcConfiguration.subnets[]? // empty' | sort -u)"

intersection() {
  local left="$1"
  local right="$2"
  comm -12 <(printf '%s\n' "$left" | sed '/^$/d' | sort -u) <(printf '%s\n' "$right" | sed '/^$/d' | sort -u)
}

shared_target_groups="$(intersection "$target_groups_a" "$target_groups_b")"
shared_log_groups="$(intersection "$log_groups_a" "$log_groups_b")"
shared_secret_refs="$(intersection "$secret_refs_a" "$secret_refs_b")"
shared_security_groups="$(intersection "$security_groups_a" "$security_groups_b")"
shared_subnets="$(intersection "$subnets_a" "$subnets_b")"

echo "Comparing ECS services in cluster: $cluster"
echo "- Service A: $service_a"
echo "- Service B: $service_b"
echo

if [[ "$service_a" == "$service_b" ]]; then
  status_line FAIL "Both names point to the same ECS service"
  failures=$((failures + 1))
else
  status_line PASS "ECS services are distinct"
fi

if [[ "$family_a" == "$family_b" ]]; then
  status_line FAIL "Task definition family is shared: $family_a"
  failures=$((failures + 1))
else
  status_line PASS "Task definition families are isolated: $family_a vs $family_b"
fi

if [[ -n "$shared_target_groups" ]]; then
  status_line FAIL "Shared target groups detected"
  printf '%s\n' "$shared_target_groups" | sed 's/^/  - /'
  failures=$((failures + 1))
else
  status_line PASS "Target groups are isolated"
fi

if [[ -n "$shared_log_groups" ]]; then
  status_line FAIL "Shared CloudWatch log groups detected"
  printf '%s\n' "$shared_log_groups" | sed 's/^/  - /'
  failures=$((failures + 1))
else
  status_line PASS "CloudWatch log groups are isolated"
fi

if [[ -n "$shared_secret_refs" ]]; then
  status_line WARN "Shared secret references detected"
  printf '%s\n' "$shared_secret_refs" | sed 's/^/  - /'
else
  status_line PASS "Secrets Manager references are isolated"
fi

if [[ "$task_role_a" == "$task_role_b" && -n "$task_role_a" ]]; then
  status_line WARN "Task role is shared: $task_role_a"
else
  status_line PASS "Task roles are isolated"
fi

if [[ "$exec_role_a" == "$exec_role_b" && -n "$exec_role_a" ]]; then
  status_line PASS "Execution role is shared, which is acceptable for baseline ECS execution"
else
  status_line PASS "Execution roles are isolated"
fi

if [[ -n "$shared_security_groups" ]]; then
  status_line INFO "Shared security groups detected (acceptable if ingress is scoped by target group/ALB rule)"
  printf '%s\n' "$shared_security_groups" | sed 's/^/  - /'
else
  status_line PASS "Security groups are isolated"
fi

if [[ -n "$shared_subnets" ]]; then
  status_line INFO "Shared subnets detected (expected in shared VPC/cluster setups)"
  printf '%s\n' "$shared_subnets" | sed 's/^/  - /'
else
  status_line PASS "Subnets are isolated"
fi

echo
echo "Detailed values"
echo "- $service_a family: $family_a"
echo "- $service_b family: $family_b"
echo "- $service_a task role: ${task_role_a:-<none>}"
echo "- $service_b task role: ${task_role_b:-<none>}"
echo "- $service_a execution role: ${exec_role_a:-<none>}"
echo "- $service_b execution role: ${exec_role_b:-<none>}"

if [[ $failures -gt 0 ]]; then
  echo
  status_line FAIL "Comparison found $failures isolation failure(s)"
  exit 1
fi

echo
status_line PASS "No hard isolation failures detected"