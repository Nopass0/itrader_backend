#!/usr/bin/env python3
"""
Bybit P2P Order Manager
Управление P2P ордерами: чаты, сообщения, отпускание средств
"""

import sys
import json
import time
import hmac
import hashlib
import requests
import uuid
from typing import Dict, List, Optional, Tuple
from datetime import datetime

class P2POrderManager:
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
    
    def get_orders(self, status: Optional[int] = None, page: int = 1, size: int = 10) -> List[Dict]:
        """
        Получает список P2P ордеров
        
        Статусы:
        - 10: ожидает оплаты от покупателя
        - 20: ожидает отпускания средств продавцом
        - 30: апелляция
        - 40: отменен
        - 50: завершен
        """
        params = {
            "page": page,
            "size": size
        }
        
        if status is not None:
            params["status"] = status
            
        result = self._make_request("/v5/p2p/order/simplifyList", params)
        
        if result.get("ret_code") == 0:
            return result.get("result", {}).get("items", [])
        else:
            print(f"❌ Ошибка получения ордеров: {result.get('ret_msg')}")
            return []
    
    def get_pending_orders(self) -> List[Dict]:
        """Получает список ожидающих ордеров (требующих действий)"""
        params = {
            "page": 1,
            "size": 100
        }
        
        result = self._make_request("/v5/p2p/order/pending/simplifyList", params)
        
        if result.get("ret_code") == 0:
            items = result.get("result", {}).get("items", [])
            return items
        else:
            print(f"❌ Ошибка получения ожидающих ордеров: {result.get('ret_msg')}")
            return []
    
    def get_chat_messages(self, order_id: str, page: int = 1, size: int = 50) -> List[Dict]:
        """
        Получает сообщения чата для ордера
        
        Типы сообщений:
        - 0: системное сообщение
        - 1: текст (пользователь)
        - 2: изображение (пользователь)
        - 5: текст (администратор)
        - 6: изображение (администратор)
        - 7: pdf (пользователь)
        - 8: видео (пользователь)
        """
        params = {
            "orderId": order_id,
            "currentPage": str(page),
            "size": str(size)
        }
        
        result = self._make_request("/v5/p2p/order/message/listpage", params)
        
        if result.get("ret_code") == 0:
            return result.get("result", [])
        else:
            print(f"❌ Ошибка получения сообщений: {result.get('ret_msg')}")
            return []
    
    def send_message(self, order_id: str, message: str, content_type: str = "str", 
                    file_name: Optional[str] = None) -> bool:
        """
        Отправляет сообщение в чат ордера
        
        Args:
            order_id: ID ордера
            message: Текст сообщения или URL файла
            content_type: Тип контента (str, pic, pdf, video)
            file_name: Имя файла (для медиа)
        """
        params = {
            "orderId": order_id,
            "message": message,
            "contentType": content_type,
            "msgUuid": str(uuid.uuid4().hex)
        }
        
        if file_name:
            params["fileName"] = file_name
            
        result = self._make_request("/v5/p2p/order/message/send", params)
        
        if result.get("ret_code") == 0:
            return True
        else:
            print(f"❌ Ошибка отправки сообщения: {result.get('ret_msg')}")
            return False
    
    def release_assets(self, order_id: str) -> bool:
        """
        Отпускает средства по ордеру (для продавца)
        
        Args:
            order_id: ID ордера
        """
        params = {
            "orderId": order_id
        }
        
        result = self._make_request("/v5/p2p/order/finish", params)
        
        if result.get("ret_code") == 0:
            print(f"✅ Средства по ордеру {order_id} успешно отпущены")
            return True
        else:
            print(f"❌ Ошибка отпускания средств: {result.get('ret_msg')}")
            return False
    
    def mark_as_paid(self, order_id: str) -> bool:
        """
        Отмечает ордер как оплаченный (для покупателя)
        
        Args:
            order_id: ID ордера
        """
        params = {
            "orderId": order_id
        }
        
        result = self._make_request("/v5/p2p/order/pay", params)
        
        if result.get("ret_code") == 0:
            print(f"✅ Ордер {order_id} отмечен как оплаченный")
            return True
        else:
            print(f"❌ Ошибка отметки оплаты: {result.get('ret_msg')}")
            return False
    
    def format_order_info(self, order: Dict) -> str:
        """Форматирует информацию об ордере для вывода"""
        order_id = order.get("id", "Unknown")
        side = "Продажа" if order.get("side") == 1 else "Покупка"
        amount = order.get("amount", "0")
        currency = order.get("currencyId", "")
        token = order.get("tokenId", "USDT")
        price = order.get("price", "0")
        status = order.get("status", 0)
        
        status_text = {
            10: "Ожидает оплаты",
            20: "Ожидает отпускания средств",
            30: "Апелляция",
            40: "Отменен",
            50: "Завершен",
            60: "Оплачивается",
            70: "Ошибка оплаты"
        }.get(status, f"Статус {status}")
        
        counterparty = order.get("targetNickName", "Unknown")
        created = order.get("createDate", "")
        
        if created:
            created_dt = datetime.fromtimestamp(int(created) / 1000)
            created_str = created_dt.strftime("%Y-%m-%d %H:%M:%S")
        else:
            created_str = "Unknown"
        
        info = f"""
📋 Ордер: {order_id}
├─ Тип: {side} {token}
├─ Сумма: {amount} {currency}
├─ Курс: {price} {currency}/{token}
├─ Статус: {status_text}
├─ Контрагент: {counterparty}
└─ Создан: {created_str}
"""
        return info
    
    def format_message(self, msg: Dict) -> str:
        """Форматирует сообщение чата для вывода"""
        msg_type = msg.get("msgType", 0)
        content = msg.get("message", "")
        sender = msg.get("nickName", "Unknown")
        timestamp = msg.get("createDate", "")
        
        if timestamp:
            msg_dt = datetime.fromtimestamp(int(timestamp) / 1000)
            time_str = msg_dt.strftime("%H:%M:%S")
        else:
            time_str = "??:??:??"
        
        # Определяем тип сообщения
        if msg_type == 0:
            sender = "🤖 Система"
        elif msg_type in [5, 6]:
            sender = "👮 Администратор"
        else:
            sender = f"👤 {sender}"
        
        # Форматируем контент
        content_type = msg.get("contentType", "str")
        if content_type == "pic":
            content = f"📷 [Изображение: {content}]"
        elif content_type == "pdf":
            content = f"📄 [PDF: {msg.get('fileName', 'document.pdf')}]"
        elif content_type == "video":
            content = f"🎥 [Видео: {msg.get('fileName', 'video.mp4')}]"
        
        return f"[{time_str}] {sender}: {content}"

def main():
    """Пример использования"""
    try:
        # Читаем входные данные
        input_data = sys.stdin.read()
        data = json.loads(input_data)
        
        # Извлекаем параметры
        api_key = data.get("api_key", "")
        api_secret = data.get("api_secret", "")
        action = data.get("action", "list_orders")
        testnet = data.get("testnet", False)
        
        if not api_key or not api_secret:
            result = {
                "ret_code": -1,
                "ret_msg": "API credentials required"
            }
            print(json.dumps(result))
            return
        
        # Создаем менеджер
        manager = P2POrderManager(api_key, api_secret, testnet)
        
        # Выполняем действие
        if action == "list_orders":
            # Список ордеров
            status = data.get("status")
            orders = manager.get_orders(status=status)
            
            result = {
                "ret_code": 0,
                "ret_msg": "SUCCESS",
                "result": {
                    "orders": orders,
                    "count": len(orders)
                }
            }
            
        elif action == "get_pending":
            # Ожидающие ордера
            orders = manager.get_pending_orders()
            
            result = {
                "ret_code": 0,
                "ret_msg": "SUCCESS",
                "result": {
                    "orders": orders,
                    "count": len(orders)
                }
            }
            
        elif action == "get_chat":
            # Получить чат
            order_id = data.get("order_id", "")
            if not order_id:
                result = {
                    "ret_code": -1,
                    "ret_msg": "order_id required"
                }
            else:
                messages = manager.get_chat_messages(order_id)
                result = {
                    "ret_code": 0,
                    "ret_msg": "SUCCESS",
                    "result": {
                        "messages": messages,
                        "count": len(messages)
                    }
                }
                
        elif action == "send_message":
            # Отправить сообщение
            order_id = data.get("order_id", "")
            message = data.get("message", "")
            
            if not order_id or not message:
                result = {
                    "ret_code": -1,
                    "ret_msg": "order_id and message required"
                }
            else:
                success = manager.send_message(order_id, message)
                result = {
                    "ret_code": 0 if success else -1,
                    "ret_msg": "SUCCESS" if success else "Failed to send message"
                }
                
        elif action == "release_assets":
            # Отпустить средства
            order_id = data.get("order_id", "")
            
            if not order_id:
                result = {
                    "ret_code": -1,
                    "ret_msg": "order_id required"
                }
            else:
                success = manager.release_assets(order_id)
                result = {
                    "ret_code": 0 if success else -1,
                    "ret_msg": "SUCCESS" if success else "Failed to release assets"
                }
                
        else:
            result = {
                "ret_code": -1,
                "ret_msg": f"Unknown action: {action}"
            }
        
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