#!/bin/bash

# Production deployment script for Raffle Platform
# This script handles the complete deployment process including:
# - Environment validation
# - Database migrations
# - Smart contract deployment
# - Application deployment
# - Health checks
# - Rollback capabilities

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
DEPLOYMENT_ENV="${DEPLOYMENT_ENV:-production}"
NAMESPACE="raffle-platform"
DOCKER_REGISTRY="${DOCKER_REGISTRY:-raffleplatform}"
VERSION="${VERSION:-$(git rev-parse --short HEAD)}"
TIMEOUT="${TIMEOUT:-600}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Error handling
cleanup() {
    local exit_code=$?
    if [ $exit_code -ne 0 ]; then
        log_error "Deployment failed with exit code $exit_code"
        log_info "Starting rollback process..."
        rollback_deployment
    fi
    exit $exit_code
}

trap cleanup EXIT

# Validation functions
validate_environment() {
    log_info "Validating deployment environment..."
    
    # Check required tools
    local required_tools=("kubectl" "docker" "helm" "jq" "curl")
    for tool in "${required_tools[@]}"; do
        if ! command -v "$tool" &> /dev/null; then
            log_error "Required tool '$tool' is not installed"
            exit 1
        fi
    done
    
    # Check Kubernetes connection
    if ! kubectl cluster-info &> /dev/null; then
        log_error "Cannot connect to Kubernetes cluster"
        exit 1
    fi
    
    # Check Docker registry access
    if ! docker info &> /dev/null; then
        log_error "Cannot connect to Docker daemon"
        exit 1
    fi
    
    # Validate environment variables
    local required_vars=(
        "DATABASE_URL"
        "REDIS_URL"
        "JWT_SECRET"
        "STRIPE_SECRET_KEY"
        "BLOCKCHAIN_RPC_URL"
        "CONTRACT_ADDRESS"
        "DEPLOYER_PRIVATE_KEY"
    )
    
    for var in "${required_vars[@]}"; do
        if [ -z "${!var:-}" ]; then
            log_error "Required environment variable '$var' is not set"
            exit 1
        fi
    done
    
    log_success "Environment validation completed"
}

# Build and push Docker images
build_and_push_images() {
    log_info "Building and pushing Docker images..."
    
    # Build backend image
    log_info "Building backend image..."
    docker build -f backend/Dockerfile.prod -t "${DOCKER_REGISTRY}/backend:${VERSION}" backend/
    docker tag "${DOCKER_REGISTRY}/backend:${VERSION}" "${DOCKER_REGISTRY}/backend:latest"
    
    # Build frontend image
    log_info "Building frontend image..."
    docker build -f frontend/Dockerfile.prod \
        --build-arg REACT_APP_API_URL="https://api.raffleplatform.com" \
        --build-arg REACT_APP_WS_URL="wss://api.raffleplatform.com" \
        --build-arg REACT_APP_CDN_URL="https://cdn.raffleplatform.com" \
        -t "${DOCKER_REGISTRY}/frontend:${VERSION}" frontend/
    docker tag "${DOCKER_REGISTRY}/frontend:${VERSION}" "${DOCKER_REGISTRY}/frontend:latest"
    
    # Push images
    log_info "Pushing images to registry..."
    docker push "${DOCKER_REGISTRY}/backend:${VERSION}"
    docker push "${DOCKER_REGISTRY}/backend:latest"
    docker push "${DOCKER_REGISTRY}/frontend:${VERSION}"
    docker push "${DOCKER_REGISTRY}/frontend:latest"
    
    log_success "Docker images built and pushed successfully"
}

# Database operations
run_database_migrations() {
    log_info "Running database migrations..."
    
    # Create a temporary pod for running migrations
    kubectl run migration-job-${VERSION} \
        --namespace=${NAMESPACE} \
        --image="${DOCKER_REGISTRY}/backend:${VERSION}" \
        --restart=Never \
        --rm -i --tty \
        --env="DATABASE_URL=${DATABASE_URL}" \
        --command -- /app/raffle-platform-backend migrate
    
    # Wait for migration to complete
    kubectl wait --for=condition=complete --timeout=${TIMEOUT}s job/migration-job-${VERSION} -n ${NAMESPACE}
    
    log_success "Database migrations completed successfully"
}

# Smart contract deployment
deploy_smart_contracts() {
    log_info "Deploying smart contracts..."
    
    cd "${PROJECT_ROOT}/contracts"
    
    # Install dependencies
    npm ci
    
    # Compile contracts
    npx hardhat compile
    
    # Deploy to Polygon mainnet
    npx hardhat run scripts/deploy.js --network polygon
    
    # Verify contracts
    npx hardhat run scripts/validate-deployment.js --network polygon
    
    log_success "Smart contracts deployed and verified successfully"
}

# Kubernetes deployment
deploy_to_kubernetes() {
    log_info "Deploying to Kubernetes..."
    
    # Create namespace if it doesn't exist
    kubectl create namespace ${NAMESPACE} --dry-run=client -o yaml | kubectl apply -f -
    
    # Apply secrets
    log_info "Applying secrets..."
    kubectl create secret generic raffle-secrets \
        --namespace=${NAMESPACE} \
        --from-literal=database-url="${DATABASE_URL}" \
        --from-literal=redis-url="${REDIS_URL}" \
        --from-literal=jwt-secret="${JWT_SECRET}" \
        --from-literal=stripe-secret-key="${STRIPE_SECRET_KEY}" \
        --from-literal=stripe-webhook-secret="${STRIPE_WEBHOOK_SECRET}" \
        --from-literal=blockchain-rpc-url="${BLOCKCHAIN_RPC_URL}" \
        --from-literal=contract-address="${CONTRACT_ADDRESS}" \
        --from-literal=deployer-private-key="${DEPLOYER_PRIVATE_KEY}" \
        --from-literal=sentry-dsn="${SENTRY_DSN}" \
        --from-literal=postgres-password="${DB_PASSWORD}" \
        --dry-run=client -o yaml | kubectl apply -f -
    
    # Apply configurations
    log_info "Applying Kubernetes configurations..."
    kubectl apply -f k8s/production/namespace.yaml
    kubectl apply -f k8s/production/postgres-deployment.yaml
    kubectl apply -f k8s/production/redis-deployment.yaml
    
    # Wait for database to be ready
    log_info "Waiting for database to be ready..."
    kubectl wait --for=condition=ready pod -l app=postgres --timeout=${TIMEOUT}s -n ${NAMESPACE}
    
    # Deploy applications
    log_info "Deploying applications..."
    
    # Update image tags in deployment files
    sed -i "s|image: raffleplatform/backend:latest|image: ${DOCKER_REGISTRY}/backend:${VERSION}|g" k8s/production/backend-deployment.yaml
    sed -i "s|image: raffleplatform/frontend:latest|image: ${DOCKER_REGISTRY}/frontend:${VERSION}|g" k8s/production/frontend-deployment.yaml
    
    kubectl apply -f k8s/production/backend-deployment.yaml
    kubectl apply -f k8s/production/frontend-deployment.yaml
    kubectl apply -f k8s/production/ingress.yaml
    
    log_success "Kubernetes deployment completed"
}

# Health checks
perform_health_checks() {
    log_info "Performing health checks..."
    
    # Wait for deployments to be ready
    log_info "Waiting for backend deployment..."
    kubectl rollout status deployment/raffle-backend --timeout=${TIMEOUT}s -n ${NAMESPACE}
    
    log_info "Waiting for frontend deployment..."
    kubectl rollout status deployment/raffle-frontend --timeout=${TIMEOUT}s -n ${NAMESPACE}
    
    # Test endpoints
    log_info "Testing API endpoints..."
    local api_url="https://api.raffleplatform.com"
    local frontend_url="https://raffleplatform.com"
    
    # Wait for services to be available
    sleep 30
    
    # Test API health endpoint
    if curl -f "${api_url}/health" > /dev/null 2>&1; then
        log_success "API health check passed"
    else
        log_error "API health check failed"
        return 1
    fi
    
    # Test frontend
    if curl -f "${frontend_url}" > /dev/null 2>&1; then
        log_success "Frontend health check passed"
    else
        log_error "Frontend health check failed"
        return 1
    fi
    
    # Test database connectivity
    log_info "Testing database connectivity..."
    kubectl exec -n ${NAMESPACE} deployment/raffle-backend -- /app/raffle-platform-backend health-check
    
    log_success "All health checks passed"
}

# Rollback function
rollback_deployment() {
    log_warning "Rolling back deployment..."
    
    # Get previous revision
    local previous_revision=$(kubectl rollout history deployment/raffle-backend -n ${NAMESPACE} | tail -2 | head -1 | awk '{print $1}')
    
    if [ -n "$previous_revision" ]; then
        log_info "Rolling back to revision $previous_revision"
        kubectl rollout undo deployment/raffle-backend --to-revision=$previous_revision -n ${NAMESPACE}
        kubectl rollout undo deployment/raffle-frontend --to-revision=$previous_revision -n ${NAMESPACE}
        
        # Wait for rollback to complete
        kubectl rollout status deployment/raffle-backend --timeout=${TIMEOUT}s -n ${NAMESPACE}
        kubectl rollout status deployment/raffle-frontend --timeout=${TIMEOUT}s -n ${NAMESPACE}
        
        log_success "Rollback completed"
    else
        log_error "No previous revision found for rollback"
    fi
}

# Monitoring setup
setup_monitoring() {
    log_info "Setting up monitoring..."
    
    # Deploy Prometheus
    helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
    helm repo update
    
    helm upgrade --install prometheus prometheus-community/kube-prometheus-stack \
        --namespace monitoring \
        --create-namespace \
        --values monitoring/prometheus-values.yaml \
        --wait
    
    # Deploy Grafana dashboards
    kubectl apply -f monitoring/grafana-dashboards.yaml
    
    log_success "Monitoring setup completed"
}

# Backup operations
create_backup() {
    log_info "Creating backup before deployment..."
    
    # Database backup
    kubectl exec -n ${NAMESPACE} deployment/postgres -- pg_dump -U raffle_user raffle_platform_prod > "backup-$(date +%Y%m%d-%H%M%S).sql"
    
    # Upload backup to cloud storage (implement based on your cloud provider)
    # aws s3 cp backup-*.sql s3://raffle-platform-backups/
    
    log_success "Backup created successfully"
}

# Main deployment function
main() {
    log_info "Starting production deployment for Raffle Platform"
    log_info "Version: ${VERSION}"
    log_info "Environment: ${DEPLOYMENT_ENV}"
    log_info "Namespace: ${NAMESPACE}"
    
    # Pre-deployment steps
    validate_environment
    create_backup
    
    # Build and deploy
    build_and_push_images
    deploy_smart_contracts
    run_database_migrations
    deploy_to_kubernetes
    
    # Post-deployment steps
    perform_health_checks
    setup_monitoring
    
    log_success "Production deployment completed successfully!"
    log_info "Frontend URL: https://raffleplatform.com"
    log_info "API URL: https://api.raffleplatform.com"
    log_info "Monitoring: https://grafana.raffleplatform.com"
}

# Script execution
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi