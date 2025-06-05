#\!/bin/bash
# Set required environment variables
export APP__admin_token="${APP__admin_token:-your-secure-token}"
export DATABASE_URL="${DATABASE_URL:-postgres://postgres:password@localhost/itrader}"
export REDIS_URL="${REDIS_URL:-redis://localhost:6379}"

# Show current configuration
echo "Running iTrader Backend with:"
echo "  Admin Token: ${APP__admin_token}"
echo "  Database: ${DATABASE_URL}"
echo "  Redis: ${REDIS_URL}"
echo ""

# Run the application
./run.sh "$@"
