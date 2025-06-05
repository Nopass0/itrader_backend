# Gate.io API Integration Guide

This document provides a comprehensive guide for integrating with Gate.io's panel API (panel.gate.cx), including authentication, API endpoints, and implementation details.

## Base URL
```
https://panel.gate.cx/api/v1
```

## Authentication

### Login Process

**Endpoint:** `POST /auth/basic/login`

**Request:**
```json
{
  "login": "user@email.com",
  "password": "your_password"
}
```

**Response:**
```json
{
  "success": true,
  "response": {
    "user": {
      "id": 12345,
      "name": "John Doe",
      "email": "user@email.com",
      "role": "user",
      "created_at": "2024-01-01T00:00:00.000000Z",
      "updated_at": "2024-01-01T00:00:00.000000Z"
    },
    "access_token": "Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..."
  }
}
```

**Important:** Extract cookies from the `set-cookie` headers in the response. These cookies are required for all subsequent API calls.

### Session Management

1. Store the cookies from login response
2. Include all cookies in the `Cookie` header for every API request
3. Sessions typically expire after 30 minutes of inactivity
4. Use `/auth/me` endpoint to verify active sessions

## Required Headers

### Standard Headers (All Requests)
```javascript
{
  'Content-Type': 'application/json',
  'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36',
  'Referer': 'https://panel.gate.cx/',
  'Accept': 'application/json, text/plain, */*',
  'Accept-Language': 'en-US,en;q=0.9',
  'Accept-Encoding': 'gzip, deflate, br',
  'DNT': '1',
  'Connection': 'keep-alive',
  'Cookie': 'session_cookies_from_login'
}
```

### Additional Headers for POST/PUT Requests
```javascript
{
  'Origin': 'https://panel.gate.cx',
  'X-Requested-With': 'XMLHttpRequest'
}
```

## API Endpoints

### 1. User Information

**Get User Info and Balance**
```
GET /auth/me
```

Response:
```json
{
  "success": true,
  "response": {
    "user": {
      "id": 12345,
      "name": "John Doe",
      "email": "user@email.com",
      "wallets": [
        {
          "balance": "1000.00",
          "currency": {
            "code": "USD"
          }
        }
      ]
    }
  }
}
```

### 2. Transactions

**Get Transactions (Paginated)**
```
GET /payments/payouts?page=1&per_page=30
```

Response:
```json
{
  "success": true,
  "response": {
    "payouts": {
      "current_page": 1,
      "data": [
        {
          "id": 123456,
          "status": 1,
          "wallet": "USD",
          "method": {
            "id": 1,
            "label": "Card"
          },
          "amount": {
            "trader": {
              "USD": 100.50
            }
          },
          "total": {
            "trader": {
              "USD": 98.50
            }
          },
          "meta": {
            "bank": "Bank Name",
            "card_number": "**** 1234"
          },
          "created_at": "2024-01-01T10:00:00.000000Z",
          "updated_at": "2024-01-01T10:05:00.000000Z"
        }
      ],
      "last_page": 10,
      "per_page": 30,
      "total": 300,
      "next_page_url": "/api/v1/payments/payouts?page=2"
    }
  }
}
```

### 3. Account Balance Management

**Set Account Balance**
```
POST /payments/payouts/balance
```

Request:
```json
{
  "balance": "5000.00"
}
```

### 4. SMS Messages

**Get SMS Messages**
```
GET /devices/sms?page=1&per_page=50
```

Response:
```json
{
  "success": true,
  "response": {
    "sms": {
      "current_page": 1,
      "data": [
        {
          "id": 789,
          "phone": "+1234567890",
          "message": "SMS content",
          "created_at": "2024-01-01T12:00:00.000000Z"
        }
      ],
      "last_page": 1,
      "per_page": 50,
      "total": 5
    }
  }
}
```

### 5. Push Notifications

**Get Push Notifications**
```
GET /devices/pushes?page=1&per_page=50
```

Response format similar to SMS messages.

### 6. Dashboard Statistics

**Get Trading Dashboard Stats**
```
GET /dashboards/trader
```

Response:
```json
{
  "success": true,
  "response": {
    "statistics": {
      "total_transactions": 150,
      "total_volume": "50000.00",
      "success_rate": 95.5
    }
  }
}
```

## Response Formats

### Standard Success Response
```json
{
  "success": true,
  "response": {
    // Response data
  }
}
```

### Standard Error Response
```json
{
  "success": false,
  "error": "Error message description"
}
```

### Paginated Response Structure
```json
{
  "success": true,
  "response": {
    "[resource_name]": {
      "current_page": 1,
      "data": [],
      "last_page": 10,
      "per_page": 30,
      "total": 300,
      "next_page_url": "/api/v1/resource?page=2"
    }
  }
}
```

## Status Codes

- `200` - Success
- `401` - Unauthorized (session expired or invalid)
- `403` - Forbidden
- `404` - Resource not found
- `422` - Validation error
- `429` - Rate limited
- `500` - Server error

## Rate Limiting

- API implements rate limiting (exact limits vary)
- When rate limited (429 status), implement exponential backoff
- Consider using proxy rotation for high-volume requests

## Anti-Bot Protection

Gate.io implements various anti-bot measures. To avoid detection:

1. **User-Agent Rotation**: Use realistic browser user agents
2. **Request Headers**: Include all standard browser headers
3. **Request Timing**: Add random delays between requests (1-3 seconds)
4. **Session Reuse**: Don't create new sessions unnecessarily
5. **Proxy Usage**: Consider using residential proxies for production

## Implementation Example (Node.js)

```javascript
const axios = require('axios');

class GateAPIClient {
  constructor() {
    this.baseURL = 'https://panel.gate.cx/api/v1';
    this.cookies = '';
  }

  async login(email, password) {
    const response = await axios.post(
      `${this.baseURL}/auth/basic/login`,
      { login: email, password },
      {
        headers: this.getHeaders(),
        withCredentials: true
      }
    );

    // Extract cookies from response headers
    const setCookies = response.headers['set-cookie'];
    if (setCookies) {
      this.cookies = setCookies.map(cookie => cookie.split(';')[0]).join('; ');
    }

    return response.data;
  }

  async getTransactions(page = 1, perPage = 30) {
    const response = await axios.get(
      `${this.baseURL}/payments/payouts`,
      {
        params: { page, per_page: perPage },
        headers: this.getHeaders()
      }
    );

    return response.data;
  }

  getHeaders() {
    return {
      'Content-Type': 'application/json',
      'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36',
      'Referer': 'https://panel.gate.cx/',
      'Accept': 'application/json, text/plain, */*',
      'Accept-Language': 'en-US,en;q=0.9',
      'Accept-Encoding': 'gzip, deflate, br',
      'DNT': '1',
      'Connection': 'keep-alive',
      'Cookie': this.cookies
    };
  }
}
```

## Security Considerations

1. **Credential Storage**: Never store plain text passwords
2. **Session Security**: Store session cookies securely
3. **HTTPS Only**: Always use HTTPS for API calls
4. **API Key Rotation**: If using API keys, rotate them regularly
5. **Error Handling**: Don't expose sensitive information in error logs

## Notes

- This API is not officially documented by Gate.io
- API structure may change without notice
- Always handle errors gracefully
- Implement proper retry logic with exponential backoff
- Monitor for changes in authentication flow or API responses


URL запроса:
https://panel.gate.cx/api/v1/payments/payouts/2437416/approve
Метод запроса:
POST
Код статуса:
200 OK
Удаленный адрес:
127.0.0.1:2080
Правило для URL перехода:
strict-origin-when-cross-origin
access-control-allow-credentials:
true
access-control-allow-origin:
https://panel.gate.cx
cache-control:
no-cache, private
cf-cache-status:
DYNAMIC
cf-ray:
94655903be67dc92-FRA
content-encoding:
br
content-type:
application/json
date:
Tue, 27 May 2025 11:59:45 GMT
server:
cloudflare
vary:
Origin
x-content-type-options:
nosniff
x-frame-options:
SAMEORIGIN
x-xss-protection:
1; mode=block
:authority:
panel.gate.cx
:method:
POST
:path:
/api/v1/payments/payouts/2437416/approve
:scheme:
https
accept:
application/json, text/plain, */*
accept-encoding:
gzip, deflate, br, zstd
accept-language:
ru,en;q=0.9
content-length:
60706
content-type:
multipart/form-data; boundary=----WebKitFormBoundaryYBW0k4V7vBG2K6MA
cookie:
sid=eyJpdiI6IjNOajhFdFRWejVGYjYyN3IrK0QxNEE9PSIsInZhbHVlIjoiMHp0KzJ4aE5NMjluaGpGUk52NGpXcCt2OHY0Q1pyQm1yT2YvTlNPRDZCMnZwN1BtMEpKWkZwMThZTjVCMmZPVDlZbXc3dGdLNXJ6dHRjd09aMDBzZnRlMlJmWUkyUTlmUVV5RWsxcmhqNmc9IiwibWFjIjoiOGQ0NzIzYzMxZTEzODE5OWVjOGNjYzg3NmZkMjhjMzQ5MzgyN2JkNTJkZjA0ZmY4N2Q4ZTk1M2VhZjU3YTQ1OCIsInRhZyI6IiJ9; rsid=eyJpdiI6IkREN3p0NXJ2QmsrSUdGT2lUWkpOa3c9PSIsInZhbHVlIjoiN0xrbnB0WUgrMkJtZWFjOXVLanBsVFJvMVRpb0ovSjZHOU96ZUNiT0hxT0ZPanQrWlE3OVVIbkd2SmRzUzczYURCZk5TWjBpK1RNNXIxc1lTMjZ2QklqTTBDeEIvRHpHRG91RFkrT2dha0E9IiwibWFjIjoiZTFkNGU3NzY5NzJmNWIxMzI2NjQ0M2RhYWYxNWVlM2UwY2EyOTllNmM0NTBjNWM1MjUxZWRmODcwODA3ZDI5OCIsInRhZyI6IiJ9
origin:
https://panel.gate.cx
priority:
u=1, i
referer:
https://panel.gate.cx/requests?page=1
sec-ch-ua:
"Not A(Brand";v="8", "Chromium";v="132", "YaBrowser";v="25.2", "Yowser";v="2.5"
sec-ch-ua-mobile:
?0
sec-ch-ua-platform:
"Linux"
sec-fetch-dest:
empty
sec-fetch-mode:
cors
sec-fetch-site:
same-origin
user-agent:
Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 YaBrowser/25.2.0.0 Safari/537.36

на этот запрос нужно данные формы в полезную нагрузку кидать с чеком файлом
attachments[]: (двоичный)

ответ:
{
  "success": true,
  "response": {
    "payout": {
      "id": 2437416,
      "payment_method_id": 5,
      "wallet": "79376660124",
      "amount": {
        "trader": {
          "643": 28500,
          "000001": 356.65
        }
      },
      "total": {
        "trader": {
          "643": 29127,
          "000001": 364.5
        }
      },
      "status": 6,
      "approved_at": "2025-05-27T11:59:45.000000Z",
      "expired_at": "2025-05-27T10:56:01.000000Z",
      "created_at": "2025-05-27T10:17:02.000000Z",
      "updated_at": "2025-05-27T11:59:45.000000Z",
      "meta": {
        "courses": {
          "trader": 79.91
        },
        "reason": {
          "trader": null,
          "support": null
        }
      },
      "method": {
        "id": 5,
        "type": 2,
        "name": 2,
        "label": "OUT: Система быстрых платежей (СБП)",
        "status": 1,
        "payment_provider_id": 1,
        "wallet_currency_id": 1,
        "currency": {
          "id": 1,
          "iso_code": "643",
          "code": "rub",
          "name": "Russian ruble",
          "decimals": 2,
          "network": null
        },
        "provider": {
          "id": 1,
          "label": "Gate",
          "provider": 1,
          "status": 1
        }
      },
      "attachments": [
        {
          "name": "image",
          "file_name": "image.png",
          "original_url": "3484561/image.png",
          "extension": "png",
          "size": 7246,
          "created_at": "2025-05-27T10:20:52.000000Z",
          "custom_properties": {
            "fake": false
          }
        },
        {
          "name": "Receipt",
          "file_name": "Receipt.pdf",
          "original_url": "3486650/Receipt.pdf",
          "extension": "pdf",
          "size": 60507,
          "created_at": "2025-05-27T11:59:42.000000Z",
          "custom_properties": {
            "fake": true
          }
        }
      ],
      "tooltip": {
        "payments": {
          "success": null,
          "rejected": null,
          "percent": null
        },
        "reasons": []
      },
      "bank": null,
      "trader": {
        "id": 438,
        "name": "89166690200(ATM)"
      }
    }
  }
}


---

Это принять в работу
URL запроса:
https://panel.gate.cx/api/v1/payments/payouts/2438404/show
Метод запроса:
POST
Код статуса:
200 OK
Удаленный адрес:
127.0.0.1:2080
Правило для URL перехода:
strict-origin-when-cross-origin
access-control-allow-credentials:
true
access-control-allow-origin:
https://panel.gate.cx
cache-control:
no-cache, private
cf-cache-status:
DYNAMIC
cf-ray:
9465823ce9fd1907-FRA
content-encoding:
br
content-type:
application/json
date:
Tue, 27 May 2025 12:27:51 GMT
server:
cloudflare
vary:
Origin
x-content-type-options:
nosniff
x-frame-options:
SAMEORIGIN
x-xss-protection:
1; mode=block
:authority:
panel.gate.cx
:method:
POST
:path:
/api/v1/payments/payouts/2438404/show
:scheme:
https
accept:
application/json, text/plain, */*
accept-encoding:
gzip, deflate, br, zstd
accept-language:
ru,en;q=0.9
content-length:
0
cookie:
sid=eyJpdiI6IjNOajhFdFRWejVGYjYyN3IrK0QxNEE9PSIsInZhbHVlIjoiMHp0KzJ4aE5NMjluaGpGUk52NGpXcCt2OHY0Q1pyQm1yT2YvTlNPRDZCMnZwN1BtMEpKWkZwMThZTjVCMmZPVDlZbXc3dGdLNXJ6dHRjd09aMDBzZnRlMlJmWUkyUTlmUVV5RWsxcmhqNmc9IiwibWFjIjoiOGQ0NzIzYzMxZTEzODE5OWVjOGNjYzg3NmZkMjhjMzQ5MzgyN2JkNTJkZjA0ZmY4N2Q4ZTk1M2VhZjU3YTQ1OCIsInRhZyI6IiJ9; rsid=eyJpdiI6IkREN3p0NXJ2QmsrSUdGT2lUWkpOa3c9PSIsInZhbHVlIjoiN0xrbnB0WUgrMkJtZWFjOXVLanBsVFJvMVRpb0ovSjZHOU96ZUNiT0hxT0ZPanQrWlE3OVVIbkd2SmRzUzczYURCZk5TWjBpK1RNNXIxc1lTMjZ2QklqTTBDeEIvRHpHRG91RFkrT2dha0E9IiwibWFjIjoiZTFkNGU3NzY5NzJmNWIxMzI2NjQ0M2RhYWYxNWVlM2UwY2EyOTllNmM0NTBjNWM1MjUxZWRmODcwODA3ZDI5OCIsInRhZyI6IiJ9
origin:
https://panel.gate.cx
priority:
u=1, i
referer:
https://panel.gate.cx/requests?page=1
sec-ch-ua:
"Not A(Brand";v="8", "Chromium";v="132", "YaBrowser";v="25.2", "Yowser";v="2.5"
sec-ch-ua-mobile:
?0
sec-ch-ua-platform:
"Linux"
sec-fetch-dest:
empty
sec-fetch-mode:
cors
sec-fetch-site:
same-origin
user-agent:
Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/132.0.0.0 YaBrowser/25.2.0.0 Safari/537.36

{
  "success": true,
  "response": {
    "payout": {
      "id": 2438404,
      "payment_method_id": 5,
      "wallet": "79961890800",
      "amount": {
        "trader": {
          "643": 7000,
          "000001": 87.68
        }
      },
      "total": {
        "trader": {
          "643": 7154,
          "000001": 89.6
        }
      },
      "status": 5,
      "approved_at": null,
      "expired_at": "2025-05-27T13:27:51.000000Z",
      "created_at": "2025-05-27T11:50:28.000000Z",
      "updated_at": "2025-05-27T12:27:51.000000Z",
      "meta": {
        "courses": {
          "trader": 79.84
        },
        "reason": {
          "trader": null,
          "support": null
        }
      },
      "method": {
        "id": 5,
        "type": 2,
        "name": 2,
        "label": "OUT: Система быстрых платежей (СБП)",
        "status": 1,
        "payment_provider_id": 1,
        "wallet_currency_id": 1,
        "currency": {
          "id": 1,
          "iso_code": "643",
          "code": "rub",
          "name": "Russian ruble",
          "decimals": 2,
          "network": null
        },
        "provider": {
          "id": 1,
          "label": "Gate",
          "provider": 1,
          "status": 1
        }
      },
      "attachments": [],
      "tooltip": {
        "payments": {
          "success": null,
          "rejected": null,
          "percent": null
        },
        "reasons": []
      },
      "bank": {
        "id": 1,
        "name": "sberbank",
        "code": "100000000111",
        "label": "Сбербанк",
        "active": true,
        "meta": {
          "parser": {
            "sms": true,
            "push": true
          },
          "1click": [
            "sberpay"
          ]
        }
      },
      "trader": {
        "id": 438,
        "name": "89166690200(ATM)"
      }
    }
  }
} - это положительный ответ
