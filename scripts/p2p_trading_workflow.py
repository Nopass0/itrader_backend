#!/usr/bin/env python3
"""
P2P Trading Workflow
Полный цикл P2P торговли: создание объявления → мониторинг ордеров → чат → отпускание средств
"""

import sys
import json
import time
from datetime import datetime
sys.path.append('.')

from scripts.bybit_smart_ad_creator import SmartAdCreator
from scripts.bybit_p2p_order_manager import P2POrderManager

class P2PTradingWorkflow:
    def __init__(self, api_key: str, api_secret: str):
        self.api_key = api_key
        self.api_secret = api_secret
        self.ad_creator = SmartAdCreator(api_key, api_secret)
        self.order_manager = P2POrderManager(api_key, api_secret)
        
    def create_ad_and_monitor(self, amount_rub: float):
        """
        Создает объявление и мониторит входящие ордера
        """
        print(f"🚀 P2P Trading Workflow")
        print(f"💵 Сумма: {amount_rub} RUB")
        print("=" * 60)
        
        # 1. Создаем объявление
        print("\n📝 Создание объявления...")
        ad_result = self.ad_creator.create_smart_ad({
            "amount": str(amount_rub),
            "remark": "Быстрая продажа USDT. Отпускаю сразу после подтверждения оплаты."
        })
        
        if ad_result.get("ret_code") != 0:
            print(f"❌ Ошибка создания объявления: {ad_result.get('ret_msg')}")
            return False
        
        ad_id = ad_result.get("result", {}).get("itemId")
        print(f"✅ Объявление создано: {ad_id}")
        
        # 2. Мониторим новые ордера
        print("\n👀 Мониторинг новых ордеров...")
        print("Нажмите Ctrl+C для остановки")
        
        processed_orders = set()
        
        try:
            while True:
                # Получаем ожидающие ордера
                orders = self.order_manager.get_orders(status=20)  # Ожидают отпускания
                
                for order in orders:
                    order_id = order.get("id")
                    
                    # Пропускаем уже обработанные
                    if order_id in processed_orders:
                        continue
                    
                    # Проверяем что это наш ордер (продажа)
                    if order.get("side") == 1:  # Мы продавец
                        print(f"\n🔔 Новый ордер: {order_id}")
                        self.handle_order(order)
                        processed_orders.add(order_id)
                
                # Ждем 30 секунд перед следующей проверкой
                time.sleep(30)
                
        except KeyboardInterrupt:
            print("\n⏹️  Мониторинг остановлен")
            
        return True
    
    def handle_order(self, order: dict):
        """
        Обрабатывает входящий ордер
        """
        order_id = order.get("id")
        amount = order.get("amount", "0")
        currency = order.get("currencyId", "RUB")
        buyer = order.get("targetNickName", "Unknown")
        
        print(f"\n💼 Обработка ордера {order_id}")
        print(f"   Покупатель: {buyer}")
        print(f"   Сумма: {amount} {currency}")
        
        # 1. Отправляем приветствие
        self.order_manager.send_message(
            order_id,
            f"Здравствуйте! Жду подтверждения оплаты {amount} {currency}. После получения сразу отпущу USDT."
        )
        
        # 2. Проверяем сообщения
        print("   📨 Проверка сообщений...")
        messages = self.order_manager.get_chat_messages(order_id)
        
        # Ищем подтверждение оплаты
        payment_confirmed = self.check_payment_confirmation(messages)
        
        if payment_confirmed:
            print("   ✅ Найдено подтверждение оплаты")
            
            # 3. Отпускаем средства
            print("   💸 Отпускаем средства...")
            success = self.order_manager.release_assets(order_id)
            
            if success:
                # 4. Отправляем подтверждение
                self.order_manager.send_message(
                    order_id,
                    "✅ USDT отправлены! Спасибо за сделку!"
                )
                print("   ✅ Сделка завершена успешно")
            else:
                print("   ❌ Ошибка отпускания средств")
        else:
            print("   ⏳ Ожидаем подтверждения оплаты")
            self.order_manager.send_message(
                order_id,
                "Пожалуйста, отправьте скриншот оплаты после перевода."
            )
    
    def check_payment_confirmation(self, messages: list) -> bool:
        """
        Проверяет наличие подтверждения оплаты в сообщениях
        """
        for msg in messages:
            content_type = msg.get("contentType", "")
            msg_type = msg.get("msgType", 0)
            
            # Проверяем изображения от пользователя (скриншоты)
            if content_type == "pic" and msg_type == 2:
                return True
                
            # Проверяем текстовые подтверждения
            message_text = msg.get("message", "").lower()
            if any(word in message_text for word in ["оплатил", "отправил", "перевел", "paid", "sent"]):
                return True
                
        return False
    
    def monitor_active_ads(self):
        """
        Показывает статус активных объявлений
        """
        print("\n📊 Активные объявления:")
        print("=" * 60)
        
        # Получаем активные объявления
        active_ads = self.ad_creator.get_active_ads()
        
        for ad in active_ads:
            ad_id = ad.get("id")
            price = ad.get("price")
            quantity = ad.get("quantity")
            frozen = ad.get("frozenQuantity", "0")
            last_qty = ad.get("lastQuantity", "0")
            payment_methods = []
            
            for term in ad.get("paymentTerms", []):
                payment_config = term.get("paymentConfig", {})
                payment_name = payment_config.get("paymentName", "Unknown")
                payment_methods.append(payment_name)
            
            print(f"\n📌 ID: {ad_id}")
            print(f"   Цена: {price} RUB/USDT")
            print(f"   Всего: {quantity} USDT")
            print(f"   Доступно: {last_qty} USDT")
            print(f"   В сделках: {frozen} USDT")
            print(f"   Методы: {', '.join(payment_methods)}")
        
        # Показываем активные ордера
        print("\n📋 Активные ордера:")
        active_orders = []
        
        for status in [10, 20]:  # Ожидают оплаты или отпускания
            orders = self.order_manager.get_orders(status=status)
            active_orders.extend(orders)
        
        if active_orders:
            for order in active_orders:
                print(self.order_manager.format_order_info(order))
        else:
            print("   Нет активных ордеров")

def main():
    """Пример использования"""
    # Загружаем учетные данные
    with open("test_data/bybit_creditials.json", "r") as f:
        credentials = json.load(f)
    
    workflow = P2PTradingWorkflow(
        api_key=credentials["api_key"],
        api_secret=credentials["api_secret"]
    )
    
    # Показываем меню
    print("🤖 P2P Trading Bot")
    print("=" * 60)
    print("1. Создать объявление и мониторить")
    print("2. Показать активные объявления и ордера")
    print("3. Выход")
    
    choice = input("\nВыберите действие (1-3): ")
    
    if choice == "1":
        amount = input("Введите сумму в RUB: ")
        try:
            amount_rub = float(amount)
            workflow.create_ad_and_monitor(amount_rub)
        except ValueError:
            print("❌ Неверная сумма")
            
    elif choice == "2":
        workflow.monitor_active_ads()
        
    else:
        print("👋 До свидания!")

if __name__ == "__main__":
    main()