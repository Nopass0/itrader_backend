# Gate.io API Status

## Current Status (June 2, 2025)

The Gate.io API endpoints are returning `410 Gone` status, indicating they have been removed or deprecated.

### Tested Endpoints

All the following endpoints return 410 Gone:
- `/api/v1/payments/payouts`
- `/api/v1/payments/payouts?page=1`
- `/api/v1/payments/payouts?filters%5Bstatus%5D%5B%5D=4&filters%5Bstatus%5D%5B%5D=5&page=1`
- `/api/v1/payments/payouts?search%5Bid%5D=2518434&page=1`

### Authentication

We are using the correct cookies (`sid` and `rsid`) from successful login, but the API still returns 410.

### Possible Solutions

1. **Use a local proxy**: The user's examples show requests going through `127.0.0.1:2080`, which might be modifying requests or routing them differently.

2. **API Migration**: The API might have moved to a different endpoint or version.

3. **Alternative Authentication**: Additional authentication headers or tokens might be required.

### Working Features

The following features are confirmed to work:
- Login and cookie saving (`gate-login`)
- Transaction approval endpoint (`/api/v1/payments/payouts/{id}/approve`) - based on user's example

### Next Steps

1. Contact Gate.io support or check their documentation for the current API endpoints
2. Investigate if a local proxy is required for API access
3. Check if there's a new API version or different base URL