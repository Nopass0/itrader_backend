#!/usr/bin/env python3
"""
P2P Trading System - Main Launcher
Convenient launcher with multiple operating modes
"""

import os
import sys
import time
import psycopg2
from datetime import datetime
from rich.console import Console
from rich.panel import Panel
from rich.prompt import Prompt, Confirm
from rich.table import Table

# Add project root to path
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

# Import our modules
from src.gmail.auth import GmailAuthManager
from src.core.account_menu import AccountMenu
from src.core.monitoring import MonitoringSystem
from src.core.transaction_processor import TransactionProcessor
from src.gate.client import GateClient
from scripts.bybit_smart_ad_creator import SmartAdCreator

console = Console()

class P2PSystemLauncher:
    def __init__(self):
        # Database connection
        self.db_url = self.get_db_url()
        
        # Components
        self.gmail_manager = GmailAuthManager(self.db_url)
        self.account_menu = AccountMenu(self.db_url)
        
    def get_db_url(self) -> str:
        """Get database URL from environment or config"""
        db_url = os.getenv('DATABASE_URL')
        if not db_url:
            # Default local database
            db_url = "postgresql://postgres:postgres@localhost:5432/p2p_trading"
        return db_url
    
    def clear_screen(self):
        """Clear terminal screen"""
        os.system('clear' if os.name == 'posix' else 'cls')
    
    def check_gmail_setup(self) -> bool:
        """Check if Gmail is configured"""
        try:
            service = self.gmail_manager.get_gmail_service()
            return service is not None
        except:
            return False
    
    def check_accounts_exist(self) -> tuple:
        """Check if accounts exist in database"""
        conn = psycopg2.connect(self.db_url)
        try:
            cur = conn.cursor()
            
            # Check Gate accounts
            cur.execute("SELECT COUNT(*) FROM gate_accounts WHERE is_active = true")
            gate_count = cur.fetchone()[0]
            
            # Check Bybit accounts
            cur.execute("SELECT COUNT(*) FROM bybit_accounts WHERE is_active = true")
            bybit_count = cur.fetchone()[0]
            
            return gate_count > 0, bybit_count > 0
            
        finally:
            conn.close()
    
    def show_system_status(self):
        """Display system status"""
        gmail_ok = self.check_gmail_setup()
        gate_ok, bybit_ok = self.check_accounts_exist()
        
        table = Table(show_header=True, header_style="bold cyan")
        table.add_column("Component", width=20)
        table.add_column("Status", width=15)
        table.add_column("Details", width=40)
        
        # Gmail status
        gmail_status = "✅ Configured" if gmail_ok else "❌ Not configured"
        gmail_details = "Ready for receipt monitoring" if gmail_ok else "Run 'Setup Gmail' first"
        table.add_row("Gmail", gmail_status, gmail_details)
        
        # Gate.io status
        gate_status = "✅ Active accounts" if gate_ok else "⚠️  No accounts"
        gate_details = "Ready for transaction monitoring" if gate_ok else "Add Gate.io accounts first"
        table.add_row("Gate.io", gate_status, gate_details)
        
        # Bybit status
        bybit_status = "✅ Active accounts" if bybit_ok else "⚠️  No accounts"
        bybit_details = "Ready for P2P trading" if bybit_ok else "Add Bybit accounts first"
        table.add_row("Bybit", bybit_status, bybit_details)
        
        console.print(table)
    
    def show_main_menu(self):
        """Show main launcher menu"""
        self.clear_screen()
        
        # Title
        console.print(Panel(
            "[bold blue]P2P Trading Automation System[/bold blue]\n"
            "[dim]Intelligent Gate.io → Bybit P2P Trading[/dim]",
            expand=False
        ))
        
        # System status
        console.print("\n[bold]System Status:[/bold]")
        self.show_system_status()
        
        # Menu options
        console.print("\n[bold cyan]Main Menu:[/bold cyan]")
        console.print("1. 🔧 Setup Gmail authentication")
        console.print("2. 👥 Manage accounts (Gate.io / Bybit)")
        console.print("3. ▶️  Start monitoring (Manual mode)")
        console.print("4. 🚀 Start monitoring (Automatic mode)")
        console.print("5. 🧪 Test mode (Single transaction)")
        console.print("6. 📊 View transaction history")
        console.print("0. Exit")
        
        return Prompt.ask("\n[bold]Select option[/bold]", choices=["0","1","2","3","4","5","6"])
    
    def setup_gmail(self):
        """Setup Gmail authentication"""
        self.clear_screen()
        console.print(Panel("[bold cyan]Gmail Setup[/bold cyan]", expand=False))
        
        console.print("\n[yellow]To setup Gmail authentication:[/yellow]")
        console.print("1. Go to Google Cloud Console")
        console.print("2. Create a new project or select existing")
        console.print("3. Enable Gmail API")
        console.print("4. Create OAuth 2.0 credentials")
        console.print("5. Download credentials.json file")
        
        creds_file = Prompt.ask("\nPath to credentials.json file")
        
        if os.path.exists(creds_file):
            try:
                self.gmail_manager.setup_gmail_account(creds_file)
                console.print("[green]✅ Gmail setup successful![/green]")
            except Exception as e:
                console.print(f"[red]❌ Setup failed: {e}[/red]")
        else:
            console.print("[red]❌ Credentials file not found[/red]")
        
        Prompt.ask("\nPress Enter to continue")
    
    def start_monitoring(self, auto_mode: bool = False):
        """Start the monitoring system"""
        self.clear_screen()
        
        mode_text = "Automatic" if auto_mode else "Manual"
        console.print(Panel(f"[bold cyan]Starting {mode_text} Mode[/bold cyan]", expand=False))
        
        # Check prerequisites
        gmail_ok = self.check_gmail_setup()
        gate_ok, bybit_ok = self.check_accounts_exist()
        
        if not gmail_ok:
            console.print("[red]❌ Gmail not configured. Please setup Gmail first.[/red]")
            Prompt.ask("\nPress Enter to continue")
            return
        
        if not gate_ok or not bybit_ok:
            console.print("[yellow]⚠️  Warning: No active accounts found[/yellow]")
            if not Confirm.ask("Continue anyway?"):
                return
        
        # Show mode description
        console.print(f"\n[bold]Running in {mode_text} mode:[/bold]")
        if auto_mode:
            console.print("• Ads will be created automatically")
            console.print("• Chat messages sent without confirmation")
            console.print("• Receipts processed automatically")
            console.print("• Funds released after validation")
        else:
            console.print("• All actions require confirmation")
            console.print("• You can review each step")
            console.print("• Safe for testing and learning")
        
        console.print("\n[dim]Press Ctrl+C to stop monitoring[/dim]\n")
        
        # Start monitoring system
        monitoring = MonitoringSystem(self.db_url, auto_mode)
        
        try:
            monitoring.start()
        except KeyboardInterrupt:
            monitoring.stop()
    
    def test_mode(self):
        """Run test mode with single transaction"""
        self.clear_screen()
        console.print(Panel("[bold cyan]Test Mode - Single Transaction[/bold cyan]", expand=False))
        
        console.print("\n[yellow]This mode will:[/yellow]")
        console.print("• Check Gate.io for ONE pending transaction")
        console.print("• Create P2P ad on Bybit")
        console.print("• Monitor for buyer response")
        console.print("• Process through complete workflow")
        console.print("• Stop after one transaction")
        
        if not Confirm.ask("\nProceed with test mode?"):
            return
        
        # Create test monitoring system
        monitoring = MonitoringSystem(self.db_url, auto_mode=False)
        
        try:
            # Get one transaction
            console.print("\n[cyan]Checking Gate.io for pending transactions...[/cyan]")
            
            conn = psycopg2.connect(self.db_url)
            cur = conn.cursor()
            
            # Get first active Gate account
            cur.execute("""
                SELECT id, login, password 
                FROM gate_accounts 
                WHERE is_active = true 
                LIMIT 1
            """)
            
            account = cur.fetchone()
            if not account:
                console.print("[red]No active Gate.io accounts found[/red]")
                return
            
            acc_id, login, password = account
            
            # Get Gate client and check transactions
            gate_client = GateClient(login, password)
            pending_txs = gate_client.get_pending_transactions()
            
            if not pending_txs:
                console.print("[yellow]No pending transactions found[/yellow]")
                
                # Ask if user wants to create test transaction
                if Confirm.ask("\nCreate a test transaction for demo?"):
                    self.create_test_transaction()
                return
            
            # Process first transaction
            tx = pending_txs[0]
            console.print(f"\n[green]Found transaction:[/green]")
            console.print(f"Amount: {tx.get('amount', {}).get('trader', {}).get('643', 0)} RUB")
            console.print(f"Wallet: {tx.get('wallet', 'Unknown')}")
            
            if Confirm.ask("\nProcess this transaction?"):
                # Create transaction record
                monitoring.process_gate_transaction(acc_id, tx)
                
                # Start monitoring (limited time)
                console.print("\n[cyan]Starting test monitoring for 10 minutes...[/cyan]")
                monitoring.transaction_processor.chat_bot.start()
                
                start_time = time.time()
                while time.time() - start_time < 600:  # 10 minutes
                    time.sleep(1)
                    
                    # Check if transaction completed
                    cur.execute("""
                        SELECT status FROM transactions 
                        WHERE gate_transaction_id = %s
                    """, (tx.get('id'),))
                    
                    result = cur.fetchone()
                    if result and result[0] in ['released', 'fool_pool', 'error']:
                        console.print(f"\n[green]Transaction completed with status: {result[0]}[/green]")
                        break
                
                monitoring.stop()
            
            conn.close()
            
        except Exception as e:
            console.print(f"[red]Test mode error: {e}[/red]")
        
        Prompt.ask("\nPress Enter to continue")
    
    def create_test_transaction(self):
        """Create a test transaction for demo purposes"""
        console.print("\n[cyan]Creating test transaction...[/cyan]")
        
        amount = Prompt.ask("Enter test amount in RUB", default="5000")
        wallet = Prompt.ask("Enter test wallet (phone)", default="+7 900 123-45-67")
        
        conn = psycopg2.connect(self.db_url)
        try:
            cur = conn.cursor()
            
            # Get first Gate account
            cur.execute("SELECT id FROM gate_accounts WHERE is_active = true LIMIT 1")
            gate_acc_id = cur.fetchone()[0]
            
            # Create test transaction
            cur.execute("""
                INSERT INTO transactions (
                    gate_transaction_id, status, gate_account_id,
                    amount_rub, wallet, bank_label, bank_code
                ) VALUES (%s, %s, %s, %s, %s, %s, %s)
                RETURNING id
            """, (
                f"TEST_{int(time.time())}", 'pending', gate_acc_id,
                float(amount), wallet, 'Т-Банк', 'tinkoff'
            ))
            
            tx_id = cur.fetchone()[0]
            conn.commit()
            
            console.print(f"[green]✅ Test transaction created (ID: {tx_id})[/green]")
            
        except Exception as e:
            conn.rollback()
            console.print(f"[red]Error creating test transaction: {e}[/red]")
        finally:
            conn.close()
    
    def view_history(self):
        """View transaction history"""
        self.clear_screen()
        console.print(Panel("[bold cyan]Transaction History[/bold cyan]", expand=False))
        
        conn = psycopg2.connect(self.db_url)
        try:
            cur = conn.cursor()
            
            # Get recent transactions
            cur.execute("""
                SELECT t.id, t.created_at, t.status, t.amount_rub,
                       t.wallet, g.login as gate_login,
                       b.name as bybit_name
                FROM transactions t
                LEFT JOIN gate_accounts g ON t.gate_account_id = g.id
                LEFT JOIN bybit_accounts b ON t.bybit_account_id = b.id
                ORDER BY t.created_at DESC
                LIMIT 20
            """)
            
            transactions = cur.fetchall()
            
            if transactions:
                table = Table(show_header=True, header_style="bold magenta")
                table.add_column("ID", width=6)
                table.add_column("Date/Time", width=20)
                table.add_column("Status", width=15)
                table.add_column("Amount RUB", width=12)
                table.add_column("Gate Account", width=15)
                table.add_column("Bybit Account", width=15)
                
                status_colors = {
                    'pending': 'yellow',
                    'processing': 'cyan',
                    'waiting_response': 'blue',
                    'waiting_payment': 'magenta',
                    'approved': 'green',
                    'released': 'bold green',
                    'fool_pool': 'red',
                    'error': 'bold red'
                }
                
                for tx in transactions:
                    tx_id, created_at, status, amount, wallet, gate_login, bybit_name = tx
                    
                    color = status_colors.get(status, 'white')
                    status_display = f"[{color}]{status}[/{color}]"
                    
                    table.add_row(
                        str(tx_id),
                        created_at.strftime("%Y-%m-%d %H:%M"),
                        status_display,
                        f"{amount:,.0f}",
                        gate_login or "N/A",
                        bybit_name or "N/A"
                    )
                
                console.print(table)
                
                # Show statistics
                cur.execute("""
                    SELECT 
                        COUNT(*) as total,
                        COUNT(CASE WHEN status = 'released' THEN 1 END) as completed,
                        COUNT(CASE WHEN status = 'fool_pool' THEN 1 END) as fool_pool,
                        COUNT(CASE WHEN status = 'error' THEN 1 END) as errors,
                        SUM(CASE WHEN status = 'released' THEN amount_rub ELSE 0 END) as total_volume
                    FROM transactions
                """)
                
                stats = cur.fetchone()
                
                console.print(f"\n[bold]Statistics:[/bold]")
                console.print(f"Total transactions: {stats[0]}")
                console.print(f"Completed: [green]{stats[1]}[/green]")
                console.print(f"Fool pool: [red]{stats[2]}[/red]")
                console.print(f"Errors: [red]{stats[3]}[/red]")
                console.print(f"Total volume: [bold]{stats[4]:,.0f} RUB[/bold]")
                
            else:
                console.print("[yellow]No transactions found[/yellow]")
            
        except Exception as e:
            console.print(f"[red]Error loading history: {e}[/red]")
        finally:
            conn.close()
        
        Prompt.ask("\nPress Enter to continue")
    
    def run(self):
        """Run the main launcher"""
        while True:
            choice = self.show_main_menu()
            
            if choice == "0":
                if Confirm.ask("Exit P2P Trading System?"):
                    console.print("[green]👋 Goodbye![/green]")
                    break
            elif choice == "1":
                self.setup_gmail()
            elif choice == "2":
                self.account_menu.run()
            elif choice == "3":
                self.start_monitoring(auto_mode=False)
            elif choice == "4":
                self.start_monitoring(auto_mode=True)
            elif choice == "5":
                self.test_mode()
            elif choice == "6":
                self.view_history()


def main():
    """Entry point"""
    try:
        launcher = P2PSystemLauncher()
        launcher.run()
    except KeyboardInterrupt:
        console.print("\n[yellow]Interrupted by user[/yellow]")
    except Exception as e:
        console.print(f"\n[red]Fatal error: {e}[/red]")
        sys.exit(1)


if __name__ == "__main__":
    main()