#!/usr/bin/env python3
"""
Example Admin Client for Auto Trader WebSocket API
Shows how to connect and control the trading system
"""

import asyncio
import json
import websockets
from datetime import datetime
import sys

class AdminClient:
    def __init__(self, uri="ws://localhost:8765", token=None):
        self.uri = uri
        self.token = token
        self.running = True
        
    async def connect(self):
        """Connect to admin WebSocket"""
        async with websockets.connect(self.uri) as websocket:
            print(f"Connected to {self.uri}")
            
            # Authenticate
            await self.authenticate(websocket)
            
            # Start listening and command tasks
            await asyncio.gather(
                self.listen_messages(websocket),
                self.send_commands(websocket)
            )
    
    async def authenticate(self, websocket):
        """Authenticate with admin token"""
        auth_msg = {
            "command": "auth",
            "token": self.token
        }
        await websocket.send(json.dumps(auth_msg))
        
    async def listen_messages(self, websocket):
        """Listen for messages from server"""
        try:
            async for message in websocket:
                data = json.loads(message)
                self.handle_message(data)
        except websockets.exceptions.ConnectionClosed:
            print("Connection closed")
            self.running = False
            
    def handle_message(self, data):
        """Handle incoming message"""
        msg_type = data.get("type")
        timestamp = datetime.now().strftime("%H:%M:%S")
        
        if msg_type == "welcome":
            print(f"[{timestamp}] Connected: {data.get('message')}")
            
        elif msg_type == "auth_success":
            print(f"[{timestamp}] ✅ Authenticated successfully")
            
        elif msg_type == "auth_failed":
            print(f"[{timestamp}] ❌ Authentication failed: {data.get('message')}")
            self.running = False
            
        elif msg_type == "response":
            print(f"[{timestamp}] Response: {json.dumps(data.get('data'), indent=2)}")
            
        elif msg_type == "event":
            event = data.get("event")
            event_data = data.get("data", {})
            print(f"[{timestamp}] 📢 Event '{event}': {json.dumps(event_data, indent=2)}")
            
        elif msg_type == "error":
            print(f"[{timestamp}] ❌ Error: {data.get('message')}")
            
        else:
            print(f"[{timestamp}] Unknown message: {json.dumps(data, indent=2)}")
            
    async def send_commands(self, websocket):
        """Send commands to server"""
        print("\nAvailable commands:")
        print("  accounts    - Show all accounts")
        print("  stats       - Show system statistics")
        print("  txs         - Show recent transactions")
        print("  add_gate    - Add Gate.io account")
        print("  add_bybit   - Add Bybit account")
        print("  help        - Show this help")
        print("  quit        - Exit\n")
        
        while self.running:
            try:
                # Get user input
                command = await asyncio.get_event_loop().run_in_executor(
                    None, input, "admin> "
                )
                
                if command == "quit":
                    self.running = False
                    break
                    
                # Process command
                msg = self.process_command(command)
                if msg:
                    await websocket.send(json.dumps(msg))
                    
            except EOFError:
                break
                
    def process_command(self, command):
        """Process user command"""
        parts = command.strip().split()
        if not parts:
            return None
            
        cmd = parts[0].lower()
        
        if cmd == "accounts":
            return {"command": "get_accounts"}
            
        elif cmd == "stats":
            return {"command": "get_stats"}
            
        elif cmd == "txs":
            return {"command": "get_transactions", "limit": 10}
            
        elif cmd == "add_gate":
            login = input("Gate.io login: ")
            password = input("Gate.io password: ")
            return {
                "command": "add_account",
                "platform": "gate",
                "credentials": {
                    "login": login,
                    "password": password
                }
            }
            
        elif cmd == "add_bybit":
            api_key = input("Bybit API key: ")
            api_secret = input("Bybit API secret: ")
            return {
                "command": "add_account",
                "platform": "bybit",
                "credentials": {
                    "api_key": api_key,
                    "api_secret": api_secret
                }
            }
            
        elif cmd == "help":
            print("\nAvailable commands:")
            print("  accounts    - Show all accounts")
            print("  stats       - Show system statistics")
            print("  txs         - Show recent transactions")
            print("  add_gate    - Add Gate.io account")
            print("  add_bybit   - Add Bybit account")
            print("  help        - Show this help")
            print("  quit        - Exit\n")
            return None
            
        else:
            print(f"Unknown command: {cmd}")
            return None


async def main():
    """Main function"""
    print("=== Auto Trader Admin Client ===\n")
    
    # Get admin token
    if len(sys.argv) > 1:
        token = sys.argv[1]
    else:
        # Try to read from settings
        try:
            with open("db/settings.json", "r") as f:
                settings = json.load(f)
                token = settings.get("admin_token")
                print(f"Using token from settings: {token}")
        except:
            token = input("Enter admin token: ")
    
    # Create and run client
    client = AdminClient(token=token)
    
    try:
        await client.connect()
    except KeyboardInterrupt:
        print("\nExiting...")
    except Exception as e:
        print(f"Error: {e}")


if __name__ == "__main__":
    asyncio.run(main())