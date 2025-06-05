#!/usr/bin/env python3
"""
Interactive Account Management Menu
"""

import os
import sys
import json
import psycopg2
from typing import List, Dict, Optional
from datetime import datetime
from rich.console import Console
from rich.table import Table
from rich.panel import Panel
from rich.prompt import Prompt, Confirm

console = Console()

class AccountMenu:
    def __init__(self, db_url: str):
        self.db_url = db_url
        
    def get_db_connection(self):
        """Get database connection"""
        return psycopg2.connect(self.db_url)
    
    def clear_screen(self):
        """Clear terminal screen"""
        os.system('clear' if os.name == 'posix' else 'cls')
    
    def show_main_menu(self):
        """Show main menu"""
        self.clear_screen()
        console.print(Panel("[bold blue]P2P Trading System - Account Management[/bold blue]", 
                           expand=False))
        
        console.print("\n[bold]Gate.io Accounts:[/bold]")
        self.show_gate_accounts()
        
        console.print("\n[bold]Bybit Accounts:[/bold]")
        self.show_bybit_accounts()
        
        console.print("\n[bold cyan]Menu Options:[/bold cyan]")
        console.print("1. Add Gate.io account")
        console.print("2. Add Bybit account")
        console.print("3. Edit Gate.io account")
        console.print("4. Edit Bybit account")
        console.print("5. Delete Gate.io account")
        console.print("6. Delete Bybit account")
        console.print("7. Test account connections")
        console.print("8. Back to main menu")
        console.print("0. Exit")
        
        return Prompt.ask("\n[bold]Select option[/bold]", choices=["0","1","2","3","4","5","6","7","8"])
    
    def show_gate_accounts(self):
        """Display Gate.io accounts"""
        conn = self.get_db_connection()
        try:
            cur = conn.cursor()
            cur.execute("""
                SELECT id, login, uid, is_active, balance_rub, 
                       last_check_time, created_at
                FROM gate_accounts
                ORDER BY id
            """)
            
            accounts = cur.fetchall()
            
            if accounts:
                table = Table(show_header=True, header_style="bold magenta")
                table.add_column("ID", style="dim", width=6)
                table.add_column("Login", width=20)
                table.add_column("UID", width=15)
                table.add_column("Active", width=8)
                table.add_column("Balance RUB", width=12)
                table.add_column("Last Check", width=20)
                
                for acc in accounts:
                    active_status = "✅ Yes" if acc[3] else "❌ No"
                    balance = f"{acc[4]:,.2f}" if acc[4] else "0.00"
                    last_check = acc[5].strftime("%Y-%m-%d %H:%M") if acc[5] else "Never"
                    
                    table.add_row(
                        str(acc[0]), acc[1], str(acc[2]), 
                        active_status, balance, last_check
                    )
                
                console.print(table)
            else:
                console.print("[yellow]No Gate.io accounts found[/yellow]")
                
        except Exception as e:
            console.print(f"[red]Error loading accounts: {e}[/red]")
        finally:
            conn.close()
    
    def show_bybit_accounts(self):
        """Display Bybit accounts"""
        conn = self.get_db_connection()
        try:
            cur = conn.cursor()
            cur.execute("""
                SELECT id, name, api_key, is_active, 
                       balance_usdt, created_at
                FROM bybit_accounts
                ORDER BY id
            """)
            
            accounts = cur.fetchall()
            
            if accounts:
                table = Table(show_header=True, header_style="bold magenta")
                table.add_column("ID", style="dim", width=6)
                table.add_column("Name", width=20)
                table.add_column("API Key", width=20)
                table.add_column("Active", width=8)
                table.add_column("Balance USDT", width=12)
                table.add_column("Created", width=20)
                
                for acc in accounts:
                    active_status = "✅ Yes" if acc[3] else "❌ No"
                    balance = f"{acc[4]:,.2f}" if acc[4] else "0.00"
                    created = acc[5].strftime("%Y-%m-%d %H:%M") if acc[5] else "Unknown"
                    api_key_short = acc[2][:10] + "..." if acc[2] else "N/A"
                    
                    table.add_row(
                        str(acc[0]), acc[1], api_key_short,
                        active_status, balance, created
                    )
                
                console.print(table)
            else:
                console.print("[yellow]No Bybit accounts found[/yellow]")
                
        except Exception as e:
            console.print(f"[red]Error loading accounts: {e}[/red]")
        finally:
            conn.close()
    
    def add_gate_account(self):
        """Add new Gate.io account"""
        console.print("\n[bold cyan]Add Gate.io Account[/bold cyan]")
        
        login = Prompt.ask("Enter login (email/phone)")
        password = Prompt.ask("Enter password", password=True)
        uid = Prompt.ask("Enter UID", default="0")
        
        # Optional: import cookies from file
        cookies_file = Prompt.ask("Path to cookies file (optional, press Enter to skip)", default="")
        cookies = None
        
        if cookies_file and os.path.exists(cookies_file):
            try:
                with open(cookies_file, 'r') as f:
                    cookies = f.read()
                console.print("[green]✅ Cookies loaded from file[/green]")
            except Exception as e:
                console.print(f"[yellow]⚠️  Failed to load cookies: {e}[/yellow]")
        
        conn = self.get_db_connection()
        try:
            cur = conn.cursor()
            
            # Insert account
            cur.execute("""
                INSERT INTO gate_accounts (login, password, uid, is_active)
                VALUES (%s, %s, %s, true)
                RETURNING id
            """, (login, password, int(uid)))
            
            account_id = cur.fetchone()[0]
            
            # Insert cookies if provided
            if cookies:
                cur.execute("""
                    INSERT INTO gate_cookies (account_id, cookies, is_valid)
                    VALUES (%s, %s, true)
                """, (account_id, cookies))
            
            conn.commit()
            console.print(f"[green]✅ Gate.io account added successfully (ID: {account_id})[/green]")
            
        except Exception as e:
            conn.rollback()
            console.print(f"[red]❌ Error adding account: {e}[/red]")
        finally:
            conn.close()
        
        Prompt.ask("\nPress Enter to continue")
    
    def add_bybit_account(self):
        """Add new Bybit account"""
        console.print("\n[bold cyan]Add Bybit Account[/bold cyan]")
        
        name = Prompt.ask("Enter account name/description")
        api_key = Prompt.ask("Enter API key")
        api_secret = Prompt.ask("Enter API secret", password=True)
        
        conn = self.get_db_connection()
        try:
            cur = conn.cursor()
            cur.execute("""
                INSERT INTO bybit_accounts (name, api_key, api_secret, is_active)
                VALUES (%s, %s, %s, true)
                RETURNING id
            """, (name, api_key, api_secret))
            
            account_id = cur.fetchone()[0]
            conn.commit()
            
            console.print(f"[green]✅ Bybit account added successfully (ID: {account_id})[/green]")
            
            # Test connection
            if Confirm.ask("Test connection now?"):
                self.test_bybit_account(account_id)
            
        except Exception as e:
            conn.rollback()
            console.print(f"[red]❌ Error adding account: {e}[/red]")
        finally:
            conn.close()
        
        Prompt.ask("\nPress Enter to continue")
    
    def edit_gate_account(self):
        """Edit Gate.io account"""
        account_id = Prompt.ask("Enter Gate.io account ID to edit")
        
        conn = self.get_db_connection()
        try:
            cur = conn.cursor()
            cur.execute("""
                SELECT login, password, uid, is_active
                FROM gate_accounts
                WHERE id = %s
            """, (account_id,))
            
            account = cur.fetchone()
            if not account:
                console.print("[red]Account not found[/red]")
                return
            
            console.print(f"\n[cyan]Current values:[/cyan]")
            console.print(f"Login: {account[0]}")
            console.print(f"UID: {account[2]}")
            console.print(f"Active: {'Yes' if account[3] else 'No'}")
            
            # Get new values
            login = Prompt.ask("New login", default=account[0])
            password = Prompt.ask("New password (press Enter to keep current)", 
                                password=True, default="")
            uid = Prompt.ask("New UID", default=str(account[2]))
            is_active = Confirm.ask("Active?", default=account[3])
            
            # Update
            if password:
                cur.execute("""
                    UPDATE gate_accounts 
                    SET login = %s, password = %s, uid = %s, 
                        is_active = %s, updated_at = CURRENT_TIMESTAMP
                    WHERE id = %s
                """, (login, password, int(uid), is_active, account_id))
            else:
                cur.execute("""
                    UPDATE gate_accounts 
                    SET login = %s, uid = %s, 
                        is_active = %s, updated_at = CURRENT_TIMESTAMP
                    WHERE id = %s
                """, (login, int(uid), is_active, account_id))
            
            conn.commit()
            console.print("[green]✅ Account updated successfully[/green]")
            
        except Exception as e:
            conn.rollback()
            console.print(f"[red]❌ Error updating account: {e}[/red]")
        finally:
            conn.close()
        
        Prompt.ask("\nPress Enter to continue")
    
    def edit_bybit_account(self):
        """Edit Bybit account"""
        account_id = Prompt.ask("Enter Bybit account ID to edit")
        
        conn = self.get_db_connection()
        try:
            cur = conn.cursor()
            cur.execute("""
                SELECT name, api_key, is_active
                FROM bybit_accounts
                WHERE id = %s
            """, (account_id,))
            
            account = cur.fetchone()
            if not account:
                console.print("[red]Account not found[/red]")
                return
            
            console.print(f"\n[cyan]Current values:[/cyan]")
            console.print(f"Name: {account[0]}")
            console.print(f"API Key: {account[1][:10]}...")
            console.print(f"Active: {'Yes' if account[2] else 'No'}")
            
            # Get new values
            name = Prompt.ask("New name", default=account[0])
            api_key = Prompt.ask("New API key (press Enter to keep current)", 
                               default="")
            api_secret = Prompt.ask("New API secret (press Enter to keep current)", 
                                  password=True, default="")
            is_active = Confirm.ask("Active?", default=account[2])
            
            # Update
            if api_key and api_secret:
                cur.execute("""
                    UPDATE bybit_accounts 
                    SET name = %s, api_key = %s, api_secret = %s,
                        is_active = %s, updated_at = CURRENT_TIMESTAMP
                    WHERE id = %s
                """, (name, api_key, api_secret, is_active, account_id))
            else:
                cur.execute("""
                    UPDATE bybit_accounts 
                    SET name = %s, is_active = %s, 
                        updated_at = CURRENT_TIMESTAMP
                    WHERE id = %s
                """, (name, is_active, account_id))
            
            conn.commit()
            console.print("[green]✅ Account updated successfully[/green]")
            
        except Exception as e:
            conn.rollback()
            console.print(f"[red]❌ Error updating account: {e}[/red]")
        finally:
            conn.close()
        
        Prompt.ask("\nPress Enter to continue")
    
    def delete_gate_account(self):
        """Delete Gate.io account"""
        account_id = Prompt.ask("Enter Gate.io account ID to delete")
        
        if Confirm.ask(f"[red]Are you sure you want to delete account {account_id}?[/red]"):
            conn = self.get_db_connection()
            try:
                cur = conn.cursor()
                cur.execute("DELETE FROM gate_accounts WHERE id = %s", (account_id,))
                conn.commit()
                console.print("[green]✅ Account deleted successfully[/green]")
            except Exception as e:
                conn.rollback()
                console.print(f"[red]❌ Error deleting account: {e}[/red]")
            finally:
                conn.close()
        
        Prompt.ask("\nPress Enter to continue")
    
    def delete_bybit_account(self):
        """Delete Bybit account"""
        account_id = Prompt.ask("Enter Bybit account ID to delete")
        
        if Confirm.ask(f"[red]Are you sure you want to delete account {account_id}?[/red]"):
            conn = self.get_db_connection()
            try:
                cur = conn.cursor()
                cur.execute("DELETE FROM bybit_accounts WHERE id = %s", (account_id,))
                conn.commit()
                console.print("[green]✅ Account deleted successfully[/green]")
            except Exception as e:
                conn.rollback()
                console.print(f"[red]❌ Error deleting account: {e}[/red]")
            finally:
                conn.close()
        
        Prompt.ask("\nPress Enter to continue")
    
    def test_bybit_account(self, account_id: int):
        """Test Bybit account connection"""
        # Import here to avoid circular imports
        sys.path.append('.')
        from scripts.bybit_get_rates import get_rates
        
        conn = self.get_db_connection()
        try:
            cur = conn.cursor()
            cur.execute("""
                SELECT api_key, api_secret
                FROM bybit_accounts
                WHERE id = %s
            """, (account_id,))
            
            account = cur.fetchone()
            if not account:
                console.print("[red]Account not found[/red]")
                return
            
            console.print("\n[cyan]Testing Bybit connection...[/cyan]")
            
            # Test rate fetching
            result = get_rates(account[0], account[1], 10000, testnet=False)
            
            if result.get("success"):
                buy_rate = result.get("buy_rate", 0)
                sell_rate = result.get("sell_rate", 0)
                console.print(f"[green]✅ Connection successful![/green]")
                console.print(f"Buy rate: {buy_rate} RUB/USDT")
                console.print(f"Sell rate: {sell_rate} RUB/USDT")
            else:
                console.print(f"[red]❌ Connection failed: {result.get('error')}[/red]")
                
        except Exception as e:
            console.print(f"[red]❌ Test failed: {e}[/red]")
        finally:
            conn.close()
    
    def test_connections(self):
        """Test all account connections"""
        console.print("\n[bold cyan]Testing Account Connections[/bold cyan]")
        
        # Test Bybit accounts
        conn = self.get_db_connection()
        try:
            cur = conn.cursor()
            cur.execute("""
                SELECT id, name FROM bybit_accounts WHERE is_active = true
            """)
            
            bybit_accounts = cur.fetchall()
            
            if bybit_accounts:
                console.print("\n[bold]Testing Bybit accounts:[/bold]")
                for acc_id, name in bybit_accounts:
                    console.print(f"\nTesting {name} (ID: {acc_id})...")
                    self.test_bybit_account(acc_id)
            
            # TODO: Add Gate.io connection tests
            
        except Exception as e:
            console.print(f"[red]Error testing connections: {e}[/red]")
        finally:
            conn.close()
        
        Prompt.ask("\nPress Enter to continue")
    
    def run(self):
        """Run the account management menu"""
        while True:
            choice = self.show_main_menu()
            
            if choice == "0":
                if Confirm.ask("Exit program?"):
                    break
            elif choice == "1":
                self.add_gate_account()
            elif choice == "2":
                self.add_bybit_account()
            elif choice == "3":
                self.edit_gate_account()
            elif choice == "4":
                self.edit_bybit_account()
            elif choice == "5":
                self.delete_gate_account()
            elif choice == "6":
                self.delete_bybit_account()
            elif choice == "7":
                self.test_connections()
            elif choice == "8":
                return  # Back to main menu