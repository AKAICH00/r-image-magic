# Kubernetes Deployment for r-image-magic

## Prerequisites

1. K8s cluster (K3s) running on Hetzner
2. Cloudflare R2 bucket: `r-image-magic-pod-assets`
3. POD provider API credentials

## Setup R2 API Credentials

### Step 1: Create R2 API Token

1. Log into Cloudflare Dashboard: https://dash.cloudflare.com
2. Go to **R2 Object Storage** > **Manage R2 API Tokens**
3. Click **Create API token**
4. Configure:
   - **Token name**: `r-image-magic-pod-assets`
   - **Permissions**: Object Read & Write
   - **Specify bucket(s)**: Select `r-image-magic-pod-assets`
5. Click **Create API Token**
6. Copy the **Access Key ID** and **Secret Access Key**

### Step 2: Create K8s Secrets

```bash
# Create R2 storage secret
kubectl create secret generic r2-pod-assets \
  --namespace r-image-magic \
  --from-literal=account-id=30ad2c6207d01d2238d8e62e7030c28d \
  --from-literal=access-key-id=YOUR_R2_ACCESS_KEY_ID \
  --from-literal=secret-access-key=YOUR_R2_SECRET_ACCESS_KEY

# Create POD provider secrets (add as you get tokens)
kubectl create secret generic pod-provider-secrets \
  --namespace r-image-magic \
  --from-literal=printful-token=YOUR_PRINTFUL_ACCESS_TOKEN \
  --from-literal=printify-key=YOUR_PRINTIFY_API_KEY \
  --from-literal=gelato-key= \
  --from-literal=spod-token= \
  --from-literal=gooten-recipe-id=
```

### Step 3: Apply Deployment

```bash
kubectl apply -f k8s/deployment.yaml
```

### Step 4: Verify

```bash
# Check deployment status
kubectl get pods -n r-image-magic

# Check logs
kubectl logs -n r-image-magic deployment/r-image-magic

# Test health endpoint
curl http://100.97.89.1:30880/health
```

## Environment Variables Reference

| Variable | Description | Required |
|----------|-------------|----------|
| `R2_ACCOUNT_ID` | Cloudflare account ID | Yes |
| `R2_ACCESS_KEY_ID` | R2 API access key ID | Yes |
| `R2_SECRET_ACCESS_KEY` | R2 API secret access key | Yes |
| `R2_BUCKET_NAME` | R2 bucket name | Yes |
| `R2_PUBLIC_URL_PREFIX` | Public URL for CDN access | No |
| `PRINTFUL_ACCESS_TOKEN` | Printful OAuth token | No |
| `PRINTIFY_API_KEY` | Printify API key | No |
| `GELATO_API_KEY` | Gelato API key | No |
| `SPOD_ACCESS_TOKEN` | SPOD OAuth token | No |
| `GOOTEN_RECIPE_ID` | Gooten recipe ID | No |

## R2 Folder Structure

```
r-image-magic-pod-assets/
├── printful/
│   ├── products/{product_id}/
│   │   ├── base/              # Base product images
│   │   ├── mockups/           # Mockup templates
│   │   └── thumbnails/        # Thumbnails
│   └── variants/{variant_id}/ # Variant-specific mockups
├── printify/
│   └── ...
├── gelato/
│   └── ...
├── spod/
│   └── ...
└── gooten/
    └── ...
```

## Obtaining Provider API Credentials

### Printful
1. Go to https://www.printful.com/dashboard/developer
2. Create a new API access token
3. Copy the access token

### Printify
1. Go to https://printify.com/app/account/api
2. Generate a new API token
3. Copy the API key

### Gelato
1. Sign up at https://www.gelato.com/
2. Go to Settings > API
3. Generate an API key

### SPOD
1. Sign up at https://www.spod.com/
2. Contact support for API access
3. Receive OAuth credentials

### Gooten
1. Sign up at https://www.gooten.com/
2. Go to Developer settings
3. Get your Recipe ID
