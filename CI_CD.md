# CI/CD Setup Guide for iTrader Backend

This guide provides comprehensive instructions for setting up GitHub Actions CI/CD pipeline for the iTrader Backend project.

## Table of Contents
- [Prerequisites](#prerequisites)
- [GitHub Secrets Configuration](#github-secrets-configuration)
- [Environment Variables](#environment-variables)
- [GitHub Actions Workflows](#github-actions-workflows)
- [Deployment Best Practices](#deployment-best-practices)
- [Security Considerations](#security-considerations)
- [Troubleshooting](#troubleshooting)

## Prerequisites

Before setting up the CI/CD pipeline, ensure you have:
- A GitHub repository for the project
- Access to repository settings to configure secrets
- Docker Hub account (for container registry)
- Production server with SSH access
- All required API keys and credentials

## GitHub Secrets Configuration

Navigate to your repository → Settings → Secrets and variables → Actions, then add the following secrets:

### Required Secrets

#### 1. API and Service Credentials
- `OPENAI_API_KEY`: Your OpenAI API key for AI chat functionality
- `BYBIT_API_KEY`: Bybit exchange API key
- `BYBIT_API_SECRET`: Bybit exchange API secret
- `GATE_API_KEY`: Gate.io API key (if using Gate.io integration)
- `GATE_API_SECRET`: Gate.io API secret (if using Gate.io integration)

#### 2. Database Configuration
- `DATABASE_URL`: PostgreSQL connection string
  ```
  postgresql://username:password@host:port/database
  ```
- `DATABASE_HOST`: Database host (e.g., `localhost` or IP address)
- `DATABASE_PORT`: Database port (default: `5432`)
- `DATABASE_NAME`: Database name
- `DATABASE_USER`: Database username
- `DATABASE_PASSWORD`: Database password

#### 3. Gmail/Email Configuration
- `GMAIL_CLIENT_ID`: OAuth2 client ID for Gmail API
- `GMAIL_CLIENT_SECRET`: OAuth2 client secret for Gmail API
- `GMAIL_REFRESH_TOKEN`: OAuth2 refresh token (obtained after initial auth)
- `GMAIL_ACCESS_TOKEN`: OAuth2 access token (optional, can be refreshed)

#### 4. Docker Registry
- `DOCKER_REGISTRY_USERNAME`: Docker Hub username
- `DOCKER_REGISTRY_PASSWORD`: Docker Hub password or access token
- `DOCKER_IMAGE_NAME`: Docker image name (e.g., `yourusername/itrader-backend`)

#### 5. Deployment Secrets
- `DEPLOY_HOST`: Production server hostname or IP
- `DEPLOY_PORT`: SSH port (default: `22`)
- `DEPLOY_USER`: SSH username for deployment
- `DEPLOY_SSH_KEY`: Private SSH key for server access (base64 encoded)
- `DEPLOY_PATH`: Deployment directory path on server

#### 6. Application Secrets
- `JWT_SECRET`: Secret key for JWT token generation (generate a strong random string)
- `ADMIN_TOKEN`: Admin API token for protected endpoints
- `ENCRYPTION_KEY`: Key for encrypting sensitive data (32-byte hex string)

### Optional Secrets
- `SENTRY_DSN`: Sentry error tracking DSN (if using Sentry)
- `SLACK_WEBHOOK_URL`: Slack webhook for deployment notifications
- `TELEGRAM_BOT_TOKEN`: Telegram bot token for notifications
- `TELEGRAM_CHAT_ID`: Telegram chat ID for notifications

## Environment Variables

These environment variables should be set in your workflow files:

```yaml
env:
  RUST_VERSION: "1.75"
  CARGO_TERM_COLOR: always
  SQLX_OFFLINE: true
  RUST_BACKTRACE: 1
  RUST_LOG: info
```

## GitHub Actions Workflows

### 1. CI Pipeline (`.github/workflows/ci.yml`)

```yaml
name: CI

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

env:
  RUST_VERSION: "1.75"
  CARGO_TERM_COLOR: always

jobs:
  test:
    name: Test
    runs-on: ubuntu-latest
    
    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_USER: test_user
          POSTGRES_PASSWORD: test_password
          POSTGRES_DB: itrader_test
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 5432:5432
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Install Rust
      uses: dtolnay/rust-toolchain@stable
      with:
        toolchain: ${{ env.RUST_VERSION }}
        components: rustfmt, clippy
    
    - name: Cache cargo dependencies
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/bin/
          ~/.cargo/registry/index/
          ~/.cargo/registry/cache/
          ~/.cargo/git/db/
          target/
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
    
    - name: Install system dependencies
      run: |
        sudo apt-get update
        sudo apt-get install -y pkg-config libssl-dev python3-dev
    
    - name: Check formatting
      run: cargo fmt -- --check
    
    - name: Run clippy
      run: cargo clippy -- -D warnings
    
    - name: Run tests
      env:
        DATABASE_URL: postgresql://test_user:test_password@localhost:5432/itrader_test
        OPENAI_API_KEY: ${{ secrets.OPENAI_API_KEY }}
        BYBIT_API_KEY: ${{ secrets.BYBIT_API_KEY }}
        BYBIT_API_SECRET: ${{ secrets.BYBIT_API_SECRET }}
      run: |
        cargo test --all-features
    
    - name: Build release
      run: cargo build --release

  security:
    name: Security Audit
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - uses: actions-rs/audit-check@v1
      with:
        token: ${{ secrets.GITHUB_TOKEN }}

  lint-python:
    name: Lint Python
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - name: Set up Python
      uses: actions/setup-python@v4
      with:
        python-version: '3.11'
    - name: Install dependencies
      run: |
        python -m pip install --upgrade pip
        pip install flake8 black mypy
    - name: Lint with flake8
      run: flake8 . --count --select=E9,F63,F7,F82 --show-source --statistics
    - name: Check formatting with black
      run: black --check .
```

### 2. CD Pipeline (`.github/workflows/cd.yml`)

```yaml
name: CD

on:
  push:
    branches: [ main ]
    tags: [ 'v*' ]

env:
  DOCKER_IMAGE: ${{ secrets.DOCKER_REGISTRY_USERNAME }}/itrader-backend

jobs:
  build-and-push:
    name: Build and Push Docker Image
    runs-on: ubuntu-latest
    outputs:
      version: ${{ steps.version.outputs.version }}
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Generate version
      id: version
      run: |
        if [[ $GITHUB_REF == refs/tags/* ]]; then
          VERSION=${GITHUB_REF#refs/tags/}
        else
          VERSION=latest
        fi
        echo "version=$VERSION" >> $GITHUB_OUTPUT
    
    - name: Set up Docker Buildx
      uses: docker/setup-buildx-action@v3
    
    - name: Log in to Docker Hub
      uses: docker/login-action@v3
      with:
        username: ${{ secrets.DOCKER_REGISTRY_USERNAME }}
        password: ${{ secrets.DOCKER_REGISTRY_PASSWORD }}
    
    - name: Build and push Docker image
      uses: docker/build-push-action@v5
      with:
        context: .
        push: true
        tags: |
          ${{ env.DOCKER_IMAGE }}:${{ steps.version.outputs.version }}
          ${{ env.DOCKER_IMAGE }}:latest
        cache-from: type=gha
        cache-to: type=gha,mode=max
        build-args: |
          RUST_VERSION=${{ env.RUST_VERSION }}

  deploy:
    name: Deploy to Production
    needs: build-and-push
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main' || startsWith(github.ref, 'refs/tags/')
    
    steps:
    - uses: actions/checkout@v4
    
    - name: Prepare SSH key
      run: |
        echo "${{ secrets.DEPLOY_SSH_KEY }}" | base64 -d > deploy_key
        chmod 600 deploy_key
    
    - name: Deploy to server
      env:
        VERSION: ${{ needs.build-and-push.outputs.version }}
      run: |
        ssh -o StrictHostKeyChecking=no -i deploy_key \
          ${{ secrets.DEPLOY_USER }}@${{ secrets.DEPLOY_HOST }} -p ${{ secrets.DEPLOY_PORT }} << 'EOF'
          
          # Navigate to deployment directory
          cd ${{ secrets.DEPLOY_PATH }}
          
          # Backup current configuration
          cp -f .env .env.backup.$(date +%Y%m%d_%H%M%S)
          
          # Pull latest image
          docker pull ${{ env.DOCKER_IMAGE }}:$VERSION
          
          # Stop current container
          docker-compose down
          
          # Update docker-compose.yml with new image version
          sed -i "s|image:.*itrader-backend.*|image: ${{ env.DOCKER_IMAGE }}:$VERSION|g" docker-compose.yml
          
          # Start new container
          docker-compose up -d
          
          # Health check
          sleep 10
          if ! docker-compose ps | grep -q "Up"; then
            echo "Deployment failed! Rolling back..."
            docker-compose down
            docker-compose up -d
            exit 1
          fi
          
          # Clean up old images
          docker image prune -f
        EOF
    
    - name: Notify deployment
      if: always()
      uses: 8398a7/action-slack@v3
      with:
        status: ${{ job.status }}
        text: |
          Deployment ${{ job.status }}!
          Version: ${{ needs.build-and-push.outputs.version }}
          Actor: ${{ github.actor }}
          Ref: ${{ github.ref }}
      env:
        SLACK_WEBHOOK_URL: ${{ secrets.SLACK_WEBHOOK_URL }}
```

### 3. Release Workflow (`.github/workflows/release.yml`)

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  create-release:
    name: Create Release
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v4
      with:
        fetch-depth: 0
    
    - name: Generate changelog
      id: changelog
      run: |
        PREVIOUS_TAG=$(git describe --tags --abbrev=0 HEAD^ 2>/dev/null || echo "")
        if [ -z "$PREVIOUS_TAG" ]; then
          CHANGELOG=$(git log --pretty=format:"- %s" HEAD)
        else
          CHANGELOG=$(git log --pretty=format:"- %s" $PREVIOUS_TAG..HEAD)
        fi
        echo "changelog<<EOF" >> $GITHUB_OUTPUT
        echo "$CHANGELOG" >> $GITHUB_OUTPUT
        echo "EOF" >> $GITHUB_OUTPUT
    
    - name: Create Release
      uses: actions/create-release@v1
      env:
        GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
      with:
        tag_name: ${{ github.ref }}
        release_name: Release ${{ github.ref }}
        body: |
          ## Changes in this Release
          ${{ steps.changelog.outputs.changelog }}
          
          ## Docker Image
          ```
          docker pull ${{ secrets.DOCKER_REGISTRY_USERNAME }}/itrader-backend:${{ github.ref_name }}
          ```
        draft: false
        prerelease: false
```

## Deployment Best Practices

### 1. Environment Configuration

Create a `.env.production` template:

```bash
# Application
RUST_LOG=info
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
ADMIN_TOKEN=${ADMIN_TOKEN}

# Database
DATABASE_URL=${DATABASE_URL}
DATABASE_MAX_CONNECTIONS=20
DATABASE_MIN_CONNECTIONS=5

# APIs
OPENAI_API_KEY=${OPENAI_API_KEY}
BYBIT_API_KEY=${BYBIT_API_KEY}
BYBIT_API_SECRET=${BYBIT_API_SECRET}

# Gmail
GMAIL_CLIENT_ID=${GMAIL_CLIENT_ID}
GMAIL_CLIENT_SECRET=${GMAIL_CLIENT_SECRET}
GMAIL_REFRESH_TOKEN=${GMAIL_REFRESH_TOKEN}

# Security
JWT_SECRET=${JWT_SECRET}
ENCRYPTION_KEY=${ENCRYPTION_KEY}

# Features
AUTO_CONFIRM=false
ENABLE_WEBSOCKET=true
```

### 2. Docker Compose Production Setup

```yaml
version: '3.8'

services:
  itrader-backend:
    image: ${DOCKER_IMAGE}:latest
    container_name: itrader-backend
    restart: unless-stopped
    ports:
      - "8080:8080"
    environment:
      - RUST_LOG=info
      - DATABASE_URL=${DATABASE_URL}
    env_file:
      - .env.production
    volumes:
      - ./data:/app/data
      - ./logs:/app/logs
      - ./db:/app/db
    depends_on:
      - postgres
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s

  postgres:
    image: postgres:15-alpine
    container_name: itrader-postgres
    restart: unless-stopped
    ports:
      - "5432:5432"
    environment:
      POSTGRES_USER: ${DATABASE_USER}
      POSTGRES_PASSWORD: ${DATABASE_PASSWORD}
      POSTGRES_DB: ${DATABASE_NAME}
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U ${DATABASE_USER}"]
      interval: 10s
      timeout: 5s
      retries: 5

volumes:
  postgres_data:
```

### 3. Rollback Strategy

Create a rollback script (`scripts/rollback.sh`):

```bash
#!/bin/bash

# Rollback to previous version
PREVIOUS_VERSION=$(docker images --format "table {{.Repository}}:{{.Tag}}" | grep itrader-backend | head -2 | tail -1 | cut -d' ' -f1)

if [ -z "$PREVIOUS_VERSION" ]; then
  echo "No previous version found!"
  exit 1
fi

echo "Rolling back to $PREVIOUS_VERSION..."

# Update docker-compose.yml
sed -i "s|image:.*itrader-backend.*|image: $PREVIOUS_VERSION|g" docker-compose.yml

# Restart with previous version
docker-compose down
docker-compose up -d

echo "Rollback complete!"
```

## Security Considerations

### 1. Secret Management Best Practices

- **Never commit secrets** to version control
- Use **strong, unique passwords** for all services
- **Rotate secrets regularly** (at least every 90 days)
- Use **environment-specific secrets** (dev, staging, prod)
- Enable **2FA** on all service accounts

### 2. SSH Key Setup

Generate deployment SSH key:
```bash
ssh-keygen -t ed25519 -f deploy_key -C "github-actions@itrader"
```

Add public key to server:
```bash
ssh-copy-id -i deploy_key.pub user@server
```

Encode private key for GitHub secret:
```bash
base64 -w 0 < deploy_key
```

### 3. Network Security

- Use **HTTPS** for all external communications
- Configure **firewall rules** on production server
- Use **VPN** or **private networks** when possible
- Implement **rate limiting** on API endpoints

### 4. Database Security

- Use **strong passwords** for database users
- Enable **SSL/TLS** for database connections
- Implement **regular backups**
- Use **read replicas** for reporting

## Troubleshooting

### Common Issues and Solutions

#### 1. Docker Build Failures
```yaml
# Add build timeout and retry
- name: Build and push Docker image
  uses: docker/build-push-action@v5
  with:
    context: .
    push: true
    tags: ${{ env.DOCKER_IMAGE }}:latest
    build-args: |
      BUILDKIT_STEP_LOG_MAX_SIZE=50000000
    timeout-minutes: 30
  retry-on: error
  retry-max-attempts: 3
```

#### 2. Database Connection Issues
- Verify DATABASE_URL format
- Check network connectivity
- Ensure database server is running
- Verify firewall rules

#### 3. SSH Deployment Failures
- Check SSH key permissions (600)
- Verify server connectivity
- Ensure deployment user has necessary permissions
- Check disk space on server

#### 4. Health Check Failures
```bash
# Debug health check
docker exec itrader-backend curl -v http://localhost:8080/health
docker logs itrader-backend --tail 100
```

### Monitoring and Alerts

Set up monitoring with:
- **Prometheus** + **Grafana** for metrics
- **ELK Stack** for log aggregation
- **Sentry** for error tracking
- **Uptime monitoring** services

### Backup Strategy

1. **Database Backups**
   ```bash
   # Daily backup script
   #!/bin/bash
   BACKUP_DIR="/backups/postgres"
   TIMESTAMP=$(date +%Y%m%d_%H%M%S)
   
   pg_dump $DATABASE_URL > "$BACKUP_DIR/backup_$TIMESTAMP.sql"
   
   # Keep only last 7 days
   find $BACKUP_DIR -name "*.sql" -mtime +7 -delete
   ```

2. **Application Data Backup**
   ```bash
   # Backup application data
   tar -czf "/backups/app/app_data_$TIMESTAMP.tar.gz" /app/data /app/db
   ```

## Maintenance

### Regular Tasks

1. **Weekly**
   - Review error logs
   - Check disk usage
   - Monitor performance metrics

2. **Monthly**
   - Update dependencies
   - Review and rotate secrets
   - Test backup restoration

3. **Quarterly**
   - Security audit
   - Performance optimization
   - Infrastructure review

### Update Dependencies

```yaml
# Automated dependency updates
name: Update Dependencies

on:
  schedule:
    - cron: '0 0 * * 1' # Weekly on Mondays

jobs:
  update:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - name: Update Rust dependencies
      run: |
        cargo update
        cargo upgrade --workspace
    - name: Create Pull Request
      uses: peter-evans/create-pull-request@v5
      with:
        commit-message: "chore: update dependencies"
        title: "Weekly dependency updates"
        body: |
          Automated dependency updates
          - Updated Cargo.lock
          - Upgraded workspace dependencies
        branch: deps/update-$(date +%Y%m%d)
```

## Conclusion

This CI/CD setup provides:
- Automated testing and quality checks
- Secure secret management
- Reliable deployment process
- Easy rollback capability
- Comprehensive monitoring

Remember to:
- Test the pipeline in a staging environment first
- Document any custom configurations
- Keep secrets secure and rotate regularly
- Monitor deployments and system health
- Have a disaster recovery plan

For additional support or questions, refer to the project documentation or create an issue in the repository.