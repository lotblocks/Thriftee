# Deployment Guide - Raffle Shopping Platform

This comprehensive guide covers all deployment scenarios for the Raffle Shopping Platform, from development to production environments.

## 🏗️ Infrastructure Overview

The platform is designed with a modern, scalable architecture supporting multiple deployment strategies:

- **Development**: Docker Compose for local development
- **Staging**: Kubernetes cluster with staging configurations
- **Production**: Auto-scaling Kubernetes deployment with enterprise features

## 📋 Prerequisites

### Required Tools
- Docker & Docker Compose
- Kubernetes CLI (kubectl)
- Helm 3.x
- Node.js 18+
- Rust 1.70+
- Git

### Required Services
- PostgreSQL 15+
- Redis 7+
- Container Registry (Docker Hub, AWS ECR, etc.)
- Kubernetes Cluster (EKS, GKE, AKS, or self-managed)
- Domain with DNS management
- SSL Certificate provider (Let's Encrypt recommended)

## 🔧 Environment Configuration

### Environment Variables

Create environment-specific configuration files:

#### Production (.env.prod)
```bash
# Database
DATABASE_URL=postgresql://raffle_user:${DB_PASSWORD}@postgres:5432/raffle_platform_prod
REDIS_URL=redis://redis:6379

# Authentication
JWT_SECRET=${JWT_SECRET}
JWT_REFRESH_SECRET=${JWT_REFRESH_SECRET}

# Payment Processing
STRIPE_SECRET_KEY=${STRIPE_SECRET_KEY}
STRIPE_WEBHOOK_SECRET=${STRIPE_WEBHOOK_SECRET}

# Blockchain
BLOCKCHAIN_RPC_URL=${POLYGON_RPC_URL}
BLOCKCHAIN_WS_URL=${POLYGON_WS_URL}
CONTRACT_ADDRESS=${CONTRACT_ADDRESS}
DEPLOYER_PRIVATE_KEY=${DEPLOYER_PRIVATE_KEY}

# External Services
CLOUDINARY_CLOUD_NAME=${CLOUDINARY_CLOUD_NAME}
CLOUDINARY_API_KEY=${CLOUDINARY_API_KEY}
CLOUDINARY_API_SECRET=${CLOUDINARY_API_SECRET}
SENTRY_DSN=${SENTRY_DSN}

# Email
SMTP_HOST=${SMTP_HOST}
SMTP_USER=${SMTP_USER}
SMTP_PASSWORD=${SMTP_PASSWORD}
SMTP_FROM=${SMTP_FROM}

# Monitoring
GRAFANA_PASSWORD=${GRAFANA_PASSWORD}
PROMETHEUS_RETENTION=30d
```

#### Staging (.env.staging)
```bash
# Similar to production but with staging-specific values
DATABASE_URL=postgresql://raffle_user:${DB_PASSWORD}@postgres:5432/raffle_platform_staging
BLOCKCHAIN_RPC_URL=${MUMBAI_RPC_URL}  # Mumbai testnet
# ... other staging-specific configurations
```

## 🐳 Docker Deployment

### Development Environment

1. **Start Services**
   ```bash
   # Start all services
   docker-compose up -d
   
   # View logs
   docker-compose logs -f
   
   # Stop services
   docker-compose down
   ```

2. **Database Setup**
   ```bash
   # Run migrations
   docker-compose exec backend /app/raffle-platform-backend migrate
   
   # Seed development data (optional)
   docker-compose exec backend /app/raffle-platform-backend seed
   ```

### Production Docker Compose

1. **Deploy Production Stack**
   ```bash
   # Deploy with production configuration
   docker-compose -f docker-compose.prod.yml up -d
   
   # Monitor deployment
   docker-compose -f docker-compose.prod.yml ps
   docker-compose -f docker-compose.prod.yml logs -f
   ```

2. **SSL Setup**
   ```bash
   # Traefik will automatically obtain SSL certificates via Let's Encrypt
   # Ensure your domain DNS points to your server
   
   # Check certificate status
   docker-compose -f docker-compose.prod.yml logs traefik | grep -i cert
   ```

3. **Health Checks**
   ```bash
   # Check application health
   curl https://your-domain.com/health
   curl https://api.your-domain.com/health
   
   # Check monitoring
   curl https://grafana.your-domain.com
   ```

## ☸️ Kubernetes Deployment

### Cluster Setup

1. **Create Namespace**
   ```bash
   kubectl create namespace raffle-platform
   kubectl config set-context --current --namespace=raffle-platform
   ```

2. **Setup Secrets**
   ```bash
   # Create secrets from environment file
   kubectl create secret generic raffle-secrets \
     --from-env-file=.env.prod \
     --namespace=raffle-platform
   
   # Verify secrets
   kubectl get secrets -n raffle-platform
   ```

### Database Deployment

1. **Deploy PostgreSQL**
   ```bash
   # Apply PostgreSQL configuration
   kubectl apply -f k8s/production/postgres-deployment.yaml
   
   # Wait for database to be ready
   kubectl wait --for=condition=ready pod -l app=postgres --timeout=300s
   
   # Run migrations
   kubectl run migration-job \
     --image=raffleplatform/backend:latest \
     --restart=Never \
     --rm -i --tty \
     --env="DATABASE_URL=${DATABASE_URL}" \
     --command -- /app/raffle-platform-backend migrate
   ```

2. **Setup Database Backup**
   ```bash
   # Deploy backup service
   kubectl apply -f k8s/production/postgres-backup.yaml
   
   # Verify backup schedule
   kubectl get cronjobs -n raffle-platform
   ```

### Application Deployment

1. **Build and Push Images**
   ```bash
   # Build backend image
   docker build -f backend/Dockerfile.prod -t raffleplatform/backend:v1.0.0 backend/
   docker push raffleplatform/backend:v1.0.0
   
   # Build frontend image
   docker build -f frontend/Dockerfile.prod -t raffleplatform/frontend:v1.0.0 frontend/
   docker push raffleplatform/frontend:v1.0.0
   ```

2. **Deploy Applications**
   ```bash
   # Deploy backend
   kubectl apply -f k8s/production/backend-deployment.yaml
   
   # Deploy frontend
   kubectl apply -f k8s/production/frontend-deployment.yaml
   
   # Deploy ingress
   kubectl apply -f k8s/production/ingress.yaml
   ```

3. **Verify Deployment**
   ```bash
   # Check pod status
   kubectl get pods -n raffle-platform
   
   # Check services
   kubectl get services -n raffle-platform
   
   # Check ingress
   kubectl get ingress -n raffle-platform
   
   # View logs
   kubectl logs -f deployment/raffle-backend -n raffle-platform
   ```

### Auto-scaling Setup

1. **Horizontal Pod Autoscaler**
   ```bash
   # HPA is included in ingress.yaml, verify it's working
   kubectl get hpa -n raffle-platform
   
   # Check scaling metrics
   kubectl describe hpa raffle-backend-hpa -n raffle-platform
   ```

2. **Cluster Autoscaler** (Cloud-specific)
   ```bash
   # AWS EKS
   kubectl apply -f https://raw.githubusercontent.com/kubernetes/autoscaler/master/cluster-autoscaler/cloudprovider/aws/examples/cluster-autoscaler-autodiscover.yaml
   
   # GKE (enabled by default)
   # AKS
   az aks update --resource-group myResourceGroup --name myAKSCluster --enable-cluster-autoscaler --min-count 1 --max-count 10
   ```

## 📊 Monitoring Setup

### Prometheus and Grafana

1. **Deploy Monitoring Stack**
   ```bash
   # Add Prometheus Helm repository
   helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
   helm repo update
   
   # Install Prometheus stack
   helm install prometheus prometheus-community/kube-prometheus-stack \
     --namespace monitoring \
     --create-namespace \
     --values monitoring/prometheus-values.yaml
   ```

2. **Configure Dashboards**
   ```bash
   # Apply custom dashboards
   kubectl apply -f monitoring/grafana-dashboards.yaml
   
   # Access Grafana
   kubectl port-forward svc/prometheus-grafana 3000:80 -n monitoring
   # Open http://localhost:3000 (admin/prom-operator)
   ```

### Log Aggregation

1. **Deploy Loki Stack**
   ```bash
   # Add Grafana Helm repository
   helm repo add grafana https://grafana.github.io/helm-charts
   
   # Install Loki
   helm install loki grafana/loki-stack \
     --namespace monitoring \
     --set grafana.enabled=false \
     --set prometheus.enabled=false
   ```

## 🔒 Security Configuration

### SSL/TLS Setup

1. **Install cert-manager**
   ```bash
   # Install cert-manager
   kubectl apply -f https://github.com/jetstack/cert-manager/releases/download/v1.13.0/cert-manager.yaml
   
   # Create Let's Encrypt issuer
   kubectl apply -f k8s/production/cert-issuer.yaml
   ```

2. **Network Policies**
   ```bash
   # Apply network policies for security
   kubectl apply -f k8s/production/network-policies.yaml
   ```

### Security Scanning

1. **Container Scanning**
   ```bash
   # Scan images for vulnerabilities
   docker run --rm -v /var/run/docker.sock:/var/run/docker.sock \
     aquasec/trivy image raffleplatform/backend:latest
   ```

2. **Kubernetes Security**
   ```bash
   # Install and run kube-bench
   kubectl apply -f https://raw.githubusercontent.com/aquasecurity/kube-bench/main/job.yaml
   kubectl logs job/kube-bench
   ```

## 🚀 Automated Deployment

### Using the Deployment Script

1. **Production Deployment**
   ```bash
   # Set environment variables
   export DEPLOYMENT_ENV=production
   export VERSION=v1.0.0
   export DOCKER_REGISTRY=raffleplatform
   
   # Run deployment script
   ./scripts/deploy-production.sh
   ```

2. **Staging Deployment**
   ```bash
   # Deploy to staging
   export DEPLOYMENT_ENV=staging
   ./scripts/deploy-production.sh
   ```

### CI/CD Pipeline

The platform includes GitHub Actions workflows for automated deployment:

1. **Continuous Integration** (`.github/workflows/ci.yml`)
   - Runs on every pull request
   - Executes all tests (unit, integration, e2e)
   - Performs security scanning
   - Builds and validates Docker images

2. **Continuous Deployment** (`.github/workflows/deploy.yml`)
   - Triggers on main branch pushes
   - Deploys to staging automatically
   - Requires manual approval for production
   - Includes rollback capabilities

## 🔄 Database Management

### Migrations

1. **Run Migrations**
   ```bash
   # Development
   cd backend && cargo run --bin migrate
   
   # Production (Kubernetes)
   kubectl run migration-job \
     --image=raffleplatform/backend:latest \
     --restart=Never \
     --rm -i --tty \
     --env="DATABASE_URL=${DATABASE_URL}" \
     --command -- /app/raffle-platform-backend migrate
   ```

2. **Rollback Migrations**
   ```bash
   # Rollback last migration
   kubectl run rollback-job \
     --image=raffleplatform/backend:latest \
     --restart=Never \
     --rm -i --tty \
     --env="DATABASE_URL=${DATABASE_URL}" \
     --command -- /app/raffle-platform-backend migrate rollback
   ```

### Backup and Recovery

1. **Automated Backups**
   ```bash
   # Backups are configured in postgres-deployment.yaml
   # Verify backup job
   kubectl get cronjobs -n raffle-platform
   
   # Manual backup
   kubectl exec -it postgres-0 -n raffle-platform -- \
     pg_dump -U raffle_user raffle_platform_prod > backup-$(date +%Y%m%d).sql
   ```

2. **Restore from Backup**
   ```bash
   # Restore database
   kubectl exec -i postgres-0 -n raffle-platform -- \
     psql -U raffle_user raffle_platform_prod < backup-20240101.sql
   ```

## 🔍 Health Checks and Monitoring

### Application Health

1. **Health Endpoints**
   ```bash
   # Backend health
   curl https://api.your-domain.com/health
   
   # Frontend health
   curl https://your-domain.com/health
   
   # Database health
   kubectl exec -it postgres-0 -n raffle-platform -- \
     pg_isready -U raffle_user -d raffle_platform_prod
   ```

2. **Performance Monitoring**
   ```bash
   # Check performance metrics
   curl https://api.your-domain.com/metrics
   
   # View Grafana dashboards
   open https://grafana.your-domain.com
   ```

### Troubleshooting

1. **Common Issues**
   ```bash
   # Check pod logs
   kubectl logs -f deployment/raffle-backend -n raffle-platform
   
   # Check events
   kubectl get events -n raffle-platform --sort-by='.lastTimestamp'
   
   # Check resource usage
   kubectl top pods -n raffle-platform
   kubectl top nodes
   ```

2. **Debug Commands**
   ```bash
   # Access pod shell
   kubectl exec -it deployment/raffle-backend -n raffle-platform -- /bin/sh
   
   # Port forward for debugging
   kubectl port-forward svc/raffle-backend-service 8080:80 -n raffle-platform
   ```

## 🔄 Rollback Procedures

### Application Rollback

1. **Kubernetes Rollback**
   ```bash
   # Check rollout history
   kubectl rollout history deployment/raffle-backend -n raffle-platform
   
   # Rollback to previous version
   kubectl rollout undo deployment/raffle-backend -n raffle-platform
   
   # Rollback to specific revision
   kubectl rollout undo deployment/raffle-backend --to-revision=2 -n raffle-platform
   ```

2. **Database Rollback**
   ```bash
   # Rollback database migrations
   kubectl run rollback-job \
     --image=raffleplatform/backend:latest \
     --restart=Never \
     --rm -i --tty \
     --env="DATABASE_URL=${DATABASE_URL}" \
     --command -- /app/raffle-platform-backend migrate rollback
   ```

## 📈 Scaling Guidelines

### Horizontal Scaling

1. **Manual Scaling**
   ```bash
   # Scale backend
   kubectl scale deployment raffle-backend --replicas=5 -n raffle-platform
   
   # Scale frontend
   kubectl scale deployment raffle-frontend --replicas=3 -n raffle-platform
   ```

2. **Auto-scaling Configuration**
   ```bash
   # Update HPA settings
   kubectl patch hpa raffle-backend-hpa -n raffle-platform -p '{"spec":{"maxReplicas":15}}'
   ```

### Vertical Scaling

1. **Resource Limits**
   ```bash
   # Update resource limits in deployment YAML
   # Then apply changes
   kubectl apply -f k8s/production/backend-deployment.yaml
   ```

## 🛡️ Security Best Practices

### Production Security Checklist

- [ ] All secrets stored in Kubernetes secrets or external secret management
- [ ] Network policies implemented to restrict pod-to-pod communication
- [ ] RBAC configured with least privilege principle
- [ ] Container images scanned for vulnerabilities
- [ ] SSL/TLS certificates properly configured and auto-renewed
- [ ] Database connections encrypted
- [ ] Regular security updates applied
- [ ] Audit logging enabled and monitored
- [ ] Backup and disaster recovery procedures tested

### Compliance

- **GDPR**: User data protection and right to be forgotten
- **PCI DSS**: Payment card data security (handled by Stripe)
- **SOC 2**: Security and availability controls
- **OWASP**: Following OWASP Top 10 security guidelines

## 📞 Support and Maintenance

### Regular Maintenance Tasks

1. **Weekly**
   - Review monitoring alerts and performance metrics
   - Check backup integrity
   - Update dependencies with security patches

2. **Monthly**
   - Review and rotate secrets
   - Analyze performance trends
   - Update documentation

3. **Quarterly**
   - Conduct security audits
   - Review and test disaster recovery procedures
   - Capacity planning and scaling review

### Emergency Procedures

1. **Incident Response**
   - Monitor alerts in Grafana/AlertManager
   - Check application logs for errors
   - Scale resources if needed
   - Implement rollback if necessary

2. **Contact Information**
   - DevOps Team: devops@raffleplatform.com
   - Security Team: security@raffleplatform.com
   - On-call Engineer: +1-XXX-XXX-XXXX

## 📚 Additional Resources

- [Kubernetes Documentation](https://kubernetes.io/docs/)
- [Docker Documentation](https://docs.docker.com/)
- [Prometheus Monitoring](https://prometheus.io/docs/)
- [Grafana Dashboards](https://grafana.com/docs/)
- [Let's Encrypt](https://letsencrypt.org/docs/)

---

This deployment guide provides comprehensive instructions for deploying the Raffle Shopping Platform in various environments. For specific questions or issues, please refer to the troubleshooting section or contact the development team.