# How to Get Valid Gate.io Cookies

## Steps to Extract Cookies from Browser

1. **Login to Gate.io**
   - Open your browser and log in to https://panel.gate.cx
   - Make sure you're fully authenticated

2. **Open Developer Tools**
   - Press F12 or right-click and select "Inspect"
   - Go to the "Application" or "Storage" tab

3. **Find Cookies**
   - In the left sidebar, find "Cookies" section
   - Click on "https://panel.gate.cx" or "https://gate.cx"

4. **Export Required Cookies**
   Look for these important cookies:
   - `PHPSESSID` or `session_id`
   - `gate_token` or similar authentication token
   - Any cookie that looks like it's for authentication

5. **Format for .gate_cookies.json**
   Create a JSON file with this format:
   ```json
   [
     {
       "domain": ".gate.cx",
       "expirationDate": 1764446400.0,
       "hostOnly": false,
       "httpOnly": true,
       "name": "PHPSESSID",
       "path": "/",
       "sameSite": "Lax",
       "secure": true,
       "session": false,
       "storeId": null,
       "value": "YOUR_SESSION_ID_HERE"
     },
     {
       "domain": ".gate.cx",
       "expirationDate": 1764446400.0,
       "hostOnly": false,
       "httpOnly": true,
       "name": "gate_token",
       "path": "/",
       "sameSite": "Lax",
       "secure": true,
       "session": false,
       "storeId": null,
       "value": "YOUR_AUTH_TOKEN_HERE"
     }
   ]
   ```

6. **Save the file**
   Save as `.gate_cookies.json` in the project root

## Alternative: Use Browser Extension

You can use a cookie export extension like:
- EditThisCookie (Chrome)
- Cookie Quick Manager (Firefox)

These can export cookies in JSON format directly.

## Testing the Cookies

After saving the cookies, test them:
```bash
./test.sh gate-test
```

This will verify if the cookies are working properly.