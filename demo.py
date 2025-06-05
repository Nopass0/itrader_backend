#!/usr/bin/env python3
import sys
import time
from datetime import datetime

# ANSI color codes
RED = '\033[91m'
GREEN = '\033[92m'
YELLOW = '\033[93m'
BLUE = '\033[94m'
CYAN = '\033[96m'
WHITE = '\033[97m'
BOLD = '\033[1m'
RESET = '\033[0m'
DIM = '\033[2m'

def print_banner():
    print(f"{BLUE}=================================={RESET}")
    print(f"{BLUE}{BOLD}    iTrader Auto-Trader System    {RESET}")
    print(f"{BLUE}=================================={RESET}")
    print()

def confirm_action(action, details):
    print(f"\n{YELLOW}{'='*80}{RESET}")
    print(f"{YELLOW}{BOLD}⚠️  ACTION REQUIRED: {action}{RESET}")
    print(f"{YELLOW}{'='*80}{RESET}")
    
    print(f"\n{CYAN}📋 Details:{RESET}")
    for key, value in details.items():
        print(f"  {WHITE}{BOLD}{key}:{RESET} {GREEN}{value}{RESET}")
    
    print(f"\n{WHITE}❓ Do you want to proceed with this action?{RESET}")
    while True:
        choice = input(f"   {CYAN}Enter your choice (yes/no): {RESET}").lower().strip()
        if choice in ['yes', 'y', 'да']:
            print(f"{GREEN}✅ Action confirmed!{RESET}")
            return True
        elif choice in ['no', 'n', 'нет']:
            print(f"{RED}❌ Action cancelled!{RESET}")
            return False
        else:
            print(f"{YELLOW}⚠️  Invalid input. Please enter 'yes' or 'no'.{RESET}")

def demo_transaction(auto_mode, tx_id):
    print(f"\n{YELLOW}📊 Found new pending transaction!{RESET}")
    
    details = {
        "Transaction ID": tx_id,
        "Amount": "75000.00 RUB",
        "Phone": "+7900******67",
        "Bank": "Tinkoff",
        "Action": "Accept and create Bybit ad"
    }
    
    if not auto_mode:
        if confirm_action("Create Virtual Transaction", details):
            print(f"{GREEN}Transaction {tx_id} processed{RESET}")
            
            # Simulate Bybit ad creation
            time.sleep(1)
            ad_details = {
                "Bybit Account": "bybit_account_1",
                "Amount RUB": "75000.00 RUB",
                "Amount USDT": "932.84 USDT",
                "Rate": "80.45 RUB/USDT",
                "Payment Method": "SBP",
                "Ad Type": "SELL USDT",
                "Duration": "15 minutes"
            }
            
            if confirm_action("Create Bybit P2P Advertisement", ad_details):
                print(f"{GREEN}Advertisement created successfully{RESET}")
            else:
                print(f"{RED}Advertisement creation cancelled{RESET}")
        else:
            print(f"{RED}Transaction {tx_id} skipped{RESET}")
    else:
        print(f"{BLUE}🤖 Auto-processing transaction...{RESET}")
        time.sleep(1)
        print(f"{GREEN}Transaction {tx_id} auto-processed{RESET}")
        print(f"{GREEN}Advertisement created successfully{RESET}")

def main():
    print_banner()
    
    # Check for auto mode
    auto_mode = '--auto' in sys.argv
    
    if auto_mode:
        print(f"{RED}{BOLD}🤖 Starting in AUTOMATIC mode{RESET}")
        print(f"{YELLOW}⚠️  All actions will be auto-confirmed!{RESET}")
        print()
        confirm = input("Are you sure you want to run in AUTOMATIC mode? (yes/no): ")
        if confirm.lower() != 'yes':
            print("Cancelled. Exiting...")
            return
    else:
        print(f"{GREEN}{BOLD}👤 Starting in MANUAL mode{RESET}")
        print(f"{GREEN}✅ Actions require confirmation{RESET}")
        print(f"{CYAN}💡 To run in automatic mode, use: python3 demo.py --auto{RESET}")
        print()
    
    print(f"\n{GREEN}System initialized. Starting demo...{RESET}")
    print(f"{DIM}Press Ctrl+C to exit{RESET}\n")
    
    cycle = 0
    try:
        while True:
            cycle += 1
            print(f"\n{WHITE}{BOLD}=== Cycle {cycle} ==={RESET}")
            print(f"{CYAN}[{datetime.now().strftime('%H:%M:%S')}] Checking for new transactions...{RESET}")
            
            # Simulate finding a transaction every 3 cycles
            if cycle % 3 == 0:
                demo_transaction(auto_mode, f"DEMO-{1000 + cycle}")
            
            print(f"{DIM}💤 Waiting 10 seconds for next check...{RESET}")
            time.sleep(10)
            
    except KeyboardInterrupt:
        print(f"\n\n{YELLOW}System shutdown requested.{RESET}")
        print(f"{GREEN}Goodbye!{RESET}")

if __name__ == "__main__":
    main()