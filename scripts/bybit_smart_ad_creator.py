#!/usr/bin/env python3
"""
Smart P2P Ad Creator for Bybit
Создает объявления с учетом активных объявлений и методов оплаты

Логика:
1. Получаем список активных объявлений (максимум 2)
2. Смотрим методы оплаты в активных объявлениях
3. Если есть SPB - используем Tinkoff, если есть Tinkoff - используем SPB
4. Если на объявление кто-то откликнулся (frozenQuantity > 0), можем использовать тот же метод снова
5. Два активных не откликнувшихся объявления не могут иметь один метод оплаты
"""

import sys
import json
import time
import hmac
import hashlib
import requests
from typing import Dict, List, Optional, Tuple

# Мапинг ID методов оплаты к их названиям (нужно уточнить реальные ID)
PAYMENT_METHOD_NAMES = {
    "582": "SPB",
    "583": "Tinkoff",
    "14": "Bank Transfer",
    "377": "Balance",
    # Добавьте реальные ID для SPB и Tinkoff
}

# Обратный мапинг
PAYMENT_NAME_TO_ID = {v: k for k, v in PAYMENT_METHOD_NAMES.items()}

class SmartAdCreator:
    def __init__(self, api_key: str, api_secret: str, testnet: bool = False):
        self.api_key = api_key
        self.api_secret = api_secret
        self.base_url = "https://api-testnet.bybit.com" if testnet else "https://api.bybit.com"
        
    def _make_request(self, endpoint: str, params: Dict = None, method: str = "POST") -> Dict:
        """Выполняет подписанный запрос к API"""
        url = self.base_url + endpoint
        
        # Генерация подписи
        timestamp = str(int(time.time() * 1000))
        recv_window = "5000"
        
        if method == "POST" and params:
            param_str = json.dumps(params, separators=(',', ':'), sort_keys=True)
            sign_str = timestamp + self.api_key + recv_window + param_str
        else:
            param_str = ""
            sign_str = timestamp + self.api_key + recv_window
            
        signature = hmac.new(
            self.api_secret.encode('utf-8'),
            sign_str.encode('utf-8'),
            hashlib.sha256
        ).hexdigest()
        
        headers = {
            "X-BAPI-API-KEY": self.api_key,
            "X-BAPI-TIMESTAMP": timestamp,
            "X-BAPI-SIGN": signature,
            "X-BAPI-RECV-WINDOW": recv_window,
            "Content-Type": "application/json"
        }
        
        try:
            if method == "POST":
                response = requests.post(url, data=param_str, headers=headers, timeout=10)
            else:
                response = requests.get(url, headers=headers, timeout=10)
                
            if response.status_code == 200:
                return response.json()
            else:
                return {
                    "ret_code": -1,
                    "ret_msg": f"HTTP {response.status_code}: {response.text}"
                }
        except Exception as e:
            return {
                "ret_code": -1,
                "ret_msg": f"Request error: {str(e)}"
            }
    
    def get_active_ads(self) -> List[Dict]:
        """Получает список активных объявлений"""
        params = {
            "status": "2",  # Available (активные)
            "page": "1",
            "size": "10"
        }
        
        result = self._make_request("/v5/p2p/item/personal/list", params)
        
        if result.get("ret_code") == 0:
            items = result.get("result", {}).get("items", [])
            # Фильтруем только активные (status=10 означает online)
            active_items = [item for item in items if item.get("status") == 10]
            return active_items
        else:
            print(f"❌ Ошибка получения объявлений: {result.get('ret_msg')}")
            return []
    
    def get_user_payment_methods(self) -> List[Dict]:
        """Получает список методов оплаты пользователя"""
        result = self._make_request("/v5/p2p/user/payment/list", {})
        
        if result.get("ret_code") == 0:
            return result.get("result", [])
        else:
            print(f"❌ Ошибка получения методов оплаты: {result.get('ret_msg')}")
            return []
    
    def analyze_active_ads(self, active_ads: List[Dict]) -> Tuple[List[str], List[str]]:
        """
        Анализирует активные объявления и возвращает:
        - blocked_payment_types: типы платежей которые нельзя использовать
        - available_payment_types: типы платежей которые можно использовать
        """
        blocked_payment_types = []
        available_payment_types = []
        
        for ad in active_ads:
            # Получаем типы платежей из paymentTerms
            payment_types = []
            for term in ad.get("paymentTerms", []):
                payment_type = str(term.get("paymentType", ""))
                if payment_type:
                    payment_types.append(payment_type)
            
            frozen_qty = float(ad.get("frozenQuantity", "0"))
            
            # Если есть активные сделки (кто-то откликнулся), можем использовать этот метод снова
            if frozen_qty > 0:
                available_payment_types.extend(payment_types)
            else:
                # Если нет откликов, блокируем этот метод для новых объявлений
                blocked_payment_types.extend(payment_types)
        
        return blocked_payment_types, available_payment_types
    
    def select_payment_method(self, user_methods: List[Dict], blocked_payment_types: List[str], 
                            active_ads: List[Dict]) -> Optional[str]:
        """Выбирает подходящий метод оплаты с учетом чередования SBP/Tinkoff"""
        
        # Находим какие методы уже используются в активных объявлениях
        active_payment_types = []
        active_payment_names = []
        for ad in active_ads:
            for term in ad.get("paymentTerms", []):
                payment_type = str(term.get("paymentType", ""))
                payment_config = term.get("paymentConfig", {})
                payment_name = payment_config.get("paymentName", "")
                if payment_type:
                    active_payment_types.append(payment_type)
                    active_payment_names.append(payment_name.lower())
        
        # Мапинг типов платежей к именам для удобства
        user_payment_map = {}
        for user_method in user_methods:
            payment_type = str(user_method.get("paymentType", ""))
            payment_id = user_method.get("id", "")
            payment_config = user_method.get("paymentConfigVo", {})
            payment_name = payment_config.get("paymentName", "")
            if payment_id != "-1":  # Пропускаем Balance
                user_payment_map[payment_name.lower()] = {
                    "id": payment_id,
                    "type": payment_type,
                    "name": payment_name
                }
        
        # Логика выбора: если есть SBP - выбираем Tinkoff, если есть Tinkoff - выбираем SBP
        if "sbp" in active_payment_names:
            # Если активно SBP, пробуем выбрать Tinkoff
            if "tinkoff" in user_payment_map:
                tinkoff_type = user_payment_map["tinkoff"]["type"]
                if tinkoff_type not in blocked_payment_types:
                    return user_payment_map["tinkoff"]["id"]
        elif "tinkoff" in active_payment_names:
            # Если активен Tinkoff, пробуем выбрать SBP
            if "sbp" in user_payment_map:
                sbp_type = user_payment_map["sbp"]["type"]
                if sbp_type not in blocked_payment_types:
                    return user_payment_map["sbp"]["id"]
        
        # Если чередование невозможно, выбираем приоритетный доступный метод
        priority_methods = ["sbp", "tinkoff", "bank transfer"]
        
        for method_name in priority_methods:
            if method_name in user_payment_map:
                payment_info = user_payment_map[method_name]
                if payment_info["type"] not in blocked_payment_types:
                    return payment_info["id"]
        
        # Если приоритетные недоступны, берем любой незаблокированный
        for payment_info in user_payment_map.values():
            if payment_info["type"] not in blocked_payment_types:
                return payment_info["id"]
        
        return None
    
    def get_current_rates(self) -> Tuple[float, float]:
        """Получает текущие курсы P2P"""
        params = {
            "tokenId": "USDT",
            "currencyId": "RUB",
            "side": "0",  # Buy USDT
            "page": "1",
            "size": "5"
        }
        
        result = self._make_request("/v5/p2p/item/online", params)
        
        if result.get("ret_code") == 0:
            items = result.get("result", {}).get("items", [])
            if items:
                # Берем среднюю цену из топ-3 объявлений
                prices = [float(item.get("price", 90)) for item in items[:3]]
                avg_price = sum(prices) / len(prices) if prices else 90.0
                return avg_price, avg_price - 1.0  # buy_rate, sell_rate
        
        # Fallback rates
        return 90.0, 89.0
    
    def create_smart_ad(self, ad_params: Dict) -> Dict:
        """Создает объявление с умной логикой выбора метода оплаты"""
        print("🧠 Анализ активных объявлений...")
        
        # 1. Получаем активные объявления
        active_ads = self.get_active_ads()
        print(f"📊 Найдено активных объявлений: {len(active_ads)}")
        
        # Проверяем лимит в 2 объявления
        if len(active_ads) >= 2:
            # Проверяем есть ли объявления с откликами
            ads_with_responses = [ad for ad in active_ads if float(ad.get("frozenQuantity", "0")) > 0]
            if len(ads_with_responses) == 0:
                return {
                    "ret_code": -1,
                    "ret_msg": "Достигнут лимит в 2 активных объявления без откликов"
                }
        
        # 2. Анализируем методы оплаты
        blocked_payment_types, available_payment_types = self.analyze_active_ads(active_ads)
        print(f"🚫 Заблокированные типы платежей: {blocked_payment_types}")
        print(f"✅ Доступные типы платежей: {available_payment_types}")
        
        # 3. Получаем методы оплаты пользователя
        user_methods = self.get_user_payment_methods()
        print(f"💳 Методов оплаты пользователя: {len(user_methods)}")
        
        # 4. Выбираем подходящий метод
        selected_method = self.select_payment_method(user_methods, blocked_payment_types, active_ads)
        
        if not selected_method:
            return {
                "ret_code": -1,
                "ret_msg": "Нет доступных методов оплаты для создания объявления"
            }
        
        print(f"✅ Выбран метод оплаты: {selected_method}")
        
        # 5. Получаем актуальные курсы
        buy_rate, sell_rate = self.get_current_rates()
        
        # 6. Подготавливаем параметры объявления
        # ВСЕГДА создаем объявления на продажу USDT
        side = "1"  # Sell USDT
        
        # Используем курс продажи
        price = sell_rate
        
        # Получаем сумму транзакции в рублях (minAmount = maxAmount)
        transaction_amount_rub = float(ad_params.get("amount", ad_params.get("minAmount", ad_params.get("maxAmount", "10000"))))
        
        # Рассчитываем количество USDT: (сумма в рублях / курс) + 5 USDT
        quantity_usdt = (transaction_amount_rub / price) + 5.0
        
        # Округляем количество до 2 знаков после запятой
        quantity_usdt = round(quantity_usdt, 2)
        
        print(f"💵 Сумма транзакции: {transaction_amount_rub} RUB")
        print(f"💱 Курс продажи: {price} RUB/USDT")
        print(f"🪙 Количество USDT: {quantity_usdt} (включая +5 USDT резерв)")
        
        create_params = {
            "tokenId": ad_params.get("tokenId", "USDT"),
            "currencyId": ad_params.get("currencyId", "RUB"),
            "side": side,  # Всегда продажа
            "priceType": "0",  # Fixed rate
            "premium": "",
            "price": str(round(price, 2)),
            "minAmount": str(transaction_amount_rub),  # minAmount = maxAmount
            "maxAmount": str(transaction_amount_rub),  # minAmount = maxAmount
            "remark": ad_params.get("remark", f"Быстрая продажа USDT. Оплата {selected_method}"),
            "tradingPreferenceSet": ad_params.get("tradingPreferenceSet", {
                "hasUnPostAd": "0",
                "isKyc": "0",
                "isEmail": "0",
                "isMobile": "0",
                "hasRegisterTime": "0",
                "registerTimeThreshold": "0",
                "orderFinishNumberDay30": "0",
                "completeRateDay30": "0",
                "nationalLimit": "",
                "hasOrderFinishNumberDay30": "0",
                "hasCompleteRateDay30": "0",
                "hasNationalLimit": "0"
            }),
            "paymentIds": [selected_method],
            "quantity": str(quantity_usdt),
            "paymentPeriod": "15",  # Всегда 15 минут
            "itemType": ad_params.get("itemType", "ORIGIN")
        }
        
        print(f"💰 Создание объявления с ценой {price} RUB/USDT...")
        
        # 7. Создаем объявление
        result = self._make_request("/v5/p2p/item/create", create_params)
        
        if result.get("ret_code") == 0:
            print("✅ Объявление успешно создано!")
        else:
            print(f"❌ Ошибка создания: {result.get('ret_msg')}")
        
        return result

def main():
    try:
        # Читаем входные данные
        input_data = sys.stdin.read()
        data = json.loads(input_data)
        
        # Извлекаем параметры
        api_key = data.get("api_key", "")
        api_secret = data.get("api_secret", "")
        ad_params = data.get("ad_params", {})
        testnet = data.get("testnet", False)
        
        if not api_key or not api_secret:
            result = {
                "ret_code": -1,
                "ret_msg": "API credentials required"
            }
        else:
            # Создаем объявление с умной логикой
            creator = SmartAdCreator(api_key, api_secret, testnet)
            result = creator.create_smart_ad(ad_params)
        
        # Выводим результат
        print(json.dumps(result))
        
    except Exception as e:
        error_result = {
            "ret_code": -1,
            "ret_msg": f"Script error: {str(e)}"
        }
        print(json.dumps(error_result))

if __name__ == "__main__":
    main()