#!/bin/bash

export DATABASE_URL="postgresql://postgres:root@localhost/itrader"

echo "Checking database tables..."
psql $DATABASE_URL -c "\dt"

echo -e "\nChecking order_pools columns..."
psql $DATABASE_URL -c "\d order_pools" 2>&1 || echo "Table order_pools does not exist"

echo -e "\nRunning migrations manually..."
sqlx migrate run