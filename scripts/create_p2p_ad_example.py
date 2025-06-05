#!/usr/bin/env python3
"""
Пример создания P2P объявления с умной логикой
"""

import json
import sys
sys.path.append('.')
from scripts.bybit_smart_ad_creator import SmartAdCreator

def create_p2p_ad(api_key: str, api_secret: str, amount_rub: float):
    """
    Создает P2P объявление на продажу USDT
    
    Args:
        api_key: API ключ Bybit
        api_secret: API секрет Bybit
        amount_rub: Сумма транзакции в рублях
    """
    # Создаем экземпляр умного создателя
    creator = SmartAdCreator(api_key, api_secret, testnet=False)
    
    # Параметры объявления
    ad_params = {
        "amount": str(amount_rub),  # Сумма в рублях
        "remark": "Быстрая продажа USDT. Перевод сразу после оплаты"
    }
    
    print(f"🚀 Создание P2P объявления на продажу USDT")
    print(f"💵 Сумма: {amount_rub} RUB")
    print("=" * 50)
    
    # Создаем объявление
    result = creator.create_smart_ad(ad_params)
    
    if result.get("ret_code") == 0:
        item_id = result.get("result", {}).get("itemId", "Unknown")
        print(f"\n✅ Объявление успешно создано!")
        print(f"📝 ID объявления: {item_id}")
        
        # Показываем детали созданного объявления
        print("\n📋 Детали объявления:")
        print(f"  - Тип: Продажа USDT")
        print(f"  - Сумма: {amount_rub} RUB")
        print(f"  - Срок оплаты: 15 минут")
        print(f"  - Статус: Активно")
        
        return True
    else:
        print(f"\n❌ Ошибка создания объявления")
        print(f"Код: {result.get('ret_code')}")
        print(f"Сообщение: {result.get('ret_msg')}")
        
        # Подсказки по ошибкам
        error_code = result.get("ret_code")
        if error_code == 912120024:
            print("💡 Подсказка: Недостаточно USDT на Funding аккаунте")
            print("   Переведите USDT со Spot на Funding аккаунт")
        elif error_code == -1:
            print("💡 Подсказка: Проверьте лимиты активных объявлений")
        
        return False

def main():
    """Пример использования"""
    # В реальном использовании загружайте из безопасного места
    with open("test_data/bybit_creditials.json", "r") as f:
        credentials = json.load(f)
    
    # Создаем объявление на 10000 рублей
    create_p2p_ad(
        api_key=credentials["api_key"],
        api_secret=credentials["api_secret"],
        amount_rub=10000
    )

if __name__ == "__main__":
    main()