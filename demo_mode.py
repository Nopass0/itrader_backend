#!/usr/bin/env python3
"""
Demo Mode - Run P2P System without PostgreSQL
Uses in-memory storage for demonstration
"""

import os
import sys
from datetime import datetime
from typing import Dict, List, Optional
from rich.console import Console
from rich.panel import Panel
from rich.prompt import Prompt, Confirm
from rich.table import Table

# Add project root to path
project_root = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, project_root)

console = Console()

# Mock data storage
class DemoStorage:
    def __init__(self):
        self.gate_accounts = []
        self.bybit_accounts = []
        self.transactions = []
        
    def add_gate_account(self, login: str, password: str):
        account = {
            "id": len(self.gate_accounts) + 1,
            "login": login,
            "password": password,
            "is_active": True,
            "balance_rub": 0.0
        }
        self.gate_accounts.append(account)
        return account
    
    def add_bybit_account(self, name: str, api_key: str, api_secret: str):
        account = {
            "id": len(self.bybit_accounts) + 1,
            "name": name,
            "api_key": api_key,
            "api_secret": api_secret,
            "is_active": True
        }
        self.bybit_accounts.append(account)
        return account
    
    def add_transaction(self, amount: float, wallet: str):
        tx = {
            "id": len(self.transactions) + 1,
            "gate_transaction_id": f"DEMO_{len(self.transactions) + 1}",
            "amount_rub": amount,
            "wallet": wallet,
            "status": "pending",
            "created_at": datetime.now()
        }
        self.transactions.append(tx)
        return tx

# Global storage
storage = DemoStorage()

class DemoLauncher:
    def __init__(self):
        self.storage = storage
    
    def clear_screen(self):
        os.system('clear' if os.name == 'posix' else 'cls')
    
    def show_main_menu(self):
        self.clear_screen()
        
        console.print(Panel(
            "[bold blue]P2P Trading System - DEMO MODE[/bold blue]\n"
            "[dim]Running without database - for demonstration only[/dim]",
            expand=False
        ))
        
        # Show accounts
        console.print("\n[bold]Gate.io Accounts:[/bold]")
        if self.storage.gate_accounts:
            for acc in self.storage.gate_accounts:
                console.print(f"  • {acc['login']} {'✅' if acc['is_active'] else '❌'}")
        else:
            console.print("  [yellow]No accounts added[/yellow]")
        
        console.print("\n[bold]Bybit Accounts:[/bold]")
        if self.storage.bybit_accounts:
            for acc in self.storage.bybit_accounts:
                console.print(f"  • {acc['name']} (API: {acc['api_key'][:10]}...)")
        else:
            console.print("  [yellow]No accounts added[/yellow]")
        
        console.print("\n[bold cyan]Demo Menu:[/bold cyan]")
        console.print("1. Add demo Gate.io account")
        console.print("2. Add demo Bybit account")
        console.print("3. Create test transaction")
        console.print("4. View transactions")
        console.print("5. Simulate P2P workflow")
        console.print("0. Exit")
        
        return Prompt.ask("\n[bold]Select option[/bold]", choices=["0","1","2","3","4","5"])
    
    def add_gate_account(self):
        console.print("\n[bold cyan]Add Demo Gate.io Account[/bold cyan]")
        
        login = Prompt.ask("Enter email/login", default="demo@gate.io")
        password = Prompt.ask("Enter password", password=True, default="demo123")
        
        account = self.storage.add_gate_account(login, password)
        console.print(f"[green]✅ Added Gate.io account: {login}[/green]")
        
        Prompt.ask("\nPress Enter to continue")
    
    def add_bybit_account(self):
        console.print("\n[bold cyan]Add Demo Bybit Account[/bold cyan]")
        
        name = Prompt.ask("Enter account name", default="Demo Bybit")
        api_key = Prompt.ask("Enter API key", default="DEMO_API_KEY_123")
        api_secret = Prompt.ask("Enter API secret", default="DEMO_SECRET_456")
        
        account = self.storage.add_bybit_account(name, api_key, api_secret)
        console.print(f"[green]✅ Added Bybit account: {name}[/green]")
        
        Prompt.ask("\nPress Enter to continue")
    
    def create_transaction(self):
        console.print("\n[bold cyan]Create Test Transaction[/bold cyan]")
        
        amount = float(Prompt.ask("Enter amount in RUB", default="5000"))
        wallet = Prompt.ask("Enter wallet/phone", default="+7 900 123-45-67")
        
        tx = self.storage.add_transaction(amount, wallet)
        console.print(f"[green]✅ Created transaction: {amount} RUB from {wallet}[/green]")
        
        Prompt.ask("\nPress Enter to continue")
    
    def view_transactions(self):
        self.clear_screen()
        console.print(Panel("[bold cyan]Transaction History[/bold cyan]", expand=False))
        
        if not self.storage.transactions:
            console.print("[yellow]No transactions yet[/yellow]")
        else:
            table = Table(show_header=True, header_style="bold magenta")
            table.add_column("ID", width=6)
            table.add_column("Amount RUB", width=12)
            table.add_column("Wallet", width=20)
            table.add_column("Status", width=15)
            table.add_column("Created", width=20)
            
            for tx in self.storage.transactions:
                table.add_row(
                    str(tx['id']),
                    f"{tx['amount_rub']:,.0f}",
                    tx['wallet'],
                    tx['status'],
                    tx['created_at'].strftime("%Y-%m-%d %H:%M")
                )
            
            console.print(table)
        
        Prompt.ask("\nPress Enter to continue")
    
    def simulate_workflow(self):
        self.clear_screen()
        console.print(Panel("[bold cyan]P2P Workflow Simulation[/bold cyan]", expand=False))
        
        if not self.storage.gate_accounts or not self.storage.bybit_accounts:
            console.print("[red]❌ Please add at least one Gate and Bybit account first[/red]")
            Prompt.ask("\nPress Enter to continue")
            return
        
        console.print("\n[yellow]This will simulate the complete P2P workflow:[/yellow]")
        console.print("1. Check Gate.io for pending transaction")
        console.print("2. Create P2P ad on Bybit")
        console.print("3. Wait for buyer response")
        console.print("4. Handle chat interaction")
        console.print("5. Process receipt")
        console.print("6. Release funds")
        
        if not Confirm.ask("\nProceed with simulation?"):
            return
        
        # Step 1: Check for transaction
        console.print("\n[cyan]Step 1: Checking Gate.io...[/cyan]")
        if not self.storage.transactions or all(tx['status'] != 'pending' for tx in self.storage.transactions):
            console.print("[yellow]No pending transactions. Creating one...[/yellow]")
            tx = self.storage.add_transaction(5000, "+7 900 123-45-67")
        else:
            tx = next(t for t in self.storage.transactions if t['status'] == 'pending')
        
        console.print(f"[green]✅ Found transaction: {tx['amount_rub']} RUB[/green]")
        
        # Step 2: Create ad
        console.print("\n[cyan]Step 2: Creating Bybit P2P ad...[/cyan]")
        console.print(f"Amount: {tx['amount_rub']} RUB")
        console.print("Payment method: Tinkoff/SBP")
        console.print("[green]✅ Ad created with ID: DEMO_AD_001[/green]")
        
        # Step 3: Wait for buyer
        console.print("\n[cyan]Step 3: Waiting for buyer...[/cyan]")
        console.print("[yellow]⏳ Monitoring for new orders...[/yellow]")
        console.print("[green]✅ New order received! Order ID: DEMO_ORDER_001[/green]")
        
        # Step 4: Chat simulation
        console.print("\n[cyan]Step 4: Chat interaction:[/cyan]")
        console.print("Bot: Здравствуйте! Оплата будет с Т банка?")
        console.print("Buyer: Да")
        console.print("Bot: Чек в формате PDF сможете отправить?")
        console.print("Buyer: Да")
        console.print("Bot: При СБП, если оплата будет на неверный банк, деньги потеряны.")
        console.print("Buyer: Подтверждаю")
        console.print(f"Bot: Реквизиты: {tx['wallet']}, {tx['amount_rub']} RUB")
        
        # Step 5: Receipt
        console.print("\n[cyan]Step 5: Processing receipt...[/cyan]")
        console.print("[yellow]📧 New email from T-Bank[/yellow]")
        console.print("[green]✅ Receipt validated: Payment successful[/green]")
        
        # Step 6: Release
        console.print("\n[cyan]Step 6: Releasing funds...[/cyan]")
        console.print("[green]✅ Funds released to buyer[/green]")
        console.print("[green]✅ Transaction completed![/green]")
        
        # Update transaction status
        tx['status'] = 'completed'
        
        Prompt.ask("\nPress Enter to continue")
    
    def run(self):
        while True:
            choice = self.show_main_menu()
            
            if choice == "0":
                if Confirm.ask("Exit demo?"):
                    console.print("[green]👋 Goodbye![/green]")
                    break
            elif choice == "1":
                self.add_gate_account()
            elif choice == "2":
                self.add_bybit_account()
            elif choice == "3":
                self.create_transaction()
            elif choice == "4":
                self.view_transactions()
            elif choice == "5":
                self.simulate_workflow()


def main():
    """Main entry point"""
    try:
        console.print(Panel(
            "[bold yellow]⚠️  DEMO MODE[/bold yellow]\n"
            "This is a demonstration version without database.\n"
            "For full functionality, please setup PostgreSQL.",
            expand=False
        ))
        
        launcher = DemoLauncher()
        launcher.run()
    except KeyboardInterrupt:
        console.print("\n[yellow]Demo interrupted[/yellow]")
    except Exception as e:
        console.print(f"\n[red]Demo error: {e}[/red]")


if __name__ == "__main__":
    main()