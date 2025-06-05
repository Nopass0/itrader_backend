# P2P Order Management System

## Overview

Система управления P2P ордерами на Bybit: чаты, сообщения, отпускание средств.

## Основные возможности

### 1. Управление ордерами
- **Список всех ордеров**: получение истории и активных ордеров
- **Ожидающие ордера**: ордера требующие действий
- **Фильтрация по статусу**: можно получить ордера определенного статуса

### 2. Чат и коммуникация
- **Получение сообщений**: вся история чата по ордеру
- **Отправка сообщений**: текст, изображения, PDF, видео
- **Системные сообщения**: автоматические уведомления

### 3. Управление транзакциями
- **Отпускание средств**: для продавца после получения оплаты
- **Отметка об оплате**: для покупателя после отправки денег

## Статусы ордеров

- `10` - Ожидает оплаты от покупателя
- `20` - Ожидает отпускания средств продавцом
- `30` - Апелляция
- `40` - Отменен
- `50` - Завершен
- `60` - Оплачивается (онлайн платеж)
- `70` - Ошибка оплаты

## Использование

### Python API

```python
from scripts.bybit_p2p_order_manager import P2POrderManager

# Инициализация
manager = P2POrderManager(api_key="KEY", api_secret="SECRET")

# Получить все ордера
orders = manager.get_orders(page=1, size=10)

# Получить ожидающие ордера
pending = manager.get_pending_orders()

# Получить чат
messages = manager.get_chat_messages(order_id="123")

# Отправить сообщение
manager.send_message(
    order_id="123",
    message="Оплата отправлена"
)

# Отпустить средства (продавец)
manager.release_assets(order_id="123")

# Отметить как оплаченное (покупатель)
manager.mark_as_paid(order_id="123")
```

### Через stdin/stdout

```bash
# Список ордеров
echo '{
  "api_key": "KEY",
  "api_secret": "SECRET",
  "action": "list_orders",
  "status": 20
}' | python scripts/bybit_p2p_order_manager.py

# Получить чат
echo '{
  "api_key": "KEY",
  "api_secret": "SECRET",
  "action": "get_chat",
  "order_id": "1234567890"
}' | python scripts/bybit_p2p_order_manager.py

# Отправить сообщение
echo '{
  "api_key": "KEY",
  "api_secret": "SECRET",
  "action": "send_message",
  "order_id": "1234567890",
  "message": "Спасибо за сделку!"
}' | python scripts/bybit_p2p_order_manager.py

# Отпустить средства
echo '{
  "api_key": "KEY",
  "api_secret": "SECRET",
  "action": "release_assets",
  "order_id": "1234567890"
}' | python scripts/bybit_p2p_order_manager.py
```

## Примеры сценариев

### Сценарий 1: Автоматическое отпускание средств
```python
# Проверяем ожидающие ордера
pending = manager.get_pending_orders()

for order in pending:
    if order['status'] == 20 and order['side'] == 1:
        # Продавец, ожидает отпускания
        messages = manager.get_chat_messages(order['id'])
        
        # Проверяем наличие подтверждения оплаты
        if has_payment_confirmation(messages):
            manager.release_assets(order['id'])
            manager.send_message(
                order['id'], 
                "Средства отпущены. Спасибо!"
            )
```

### Сценарий 2: Мониторинг новых сообщений
```python
# Получаем активные ордера
active_orders = [o for o in orders if o['status'] in [10, 20]]

for order in active_orders:
    messages = manager.get_chat_messages(order['id'])
    unread = [m for m in messages if m.get('isRead') == 0]
    
    if unread:
        print(f"Новые сообщения в ордере {order['id']}:")
        for msg in unread:
            print(manager.format_message(msg))
```

## Типы сообщений

- `0` - Системное сообщение
- `1` - Текст от пользователя
- `2` - Изображение от пользователя
- `5` - Текст от администратора
- `6` - Изображение от администратора
- `7` - PDF от пользователя
- `8` - Видео от пользователя

## Типы контента

- `str` - Текстовое сообщение
- `pic` - Изображение (URL)
- `pdf` - PDF документ
- `video` - Видео файл

## Безопасность

### Рекомендации
1. **Проверяйте оплату** перед отпусканием средств
2. **Сохраняйте скриншоты** платежей в чате
3. **Используйте системные сообщения** для подтверждений
4. **Не отправляйте** конфиденциальную информацию

### Автоматизация
- Можно настроить автоматическое отпускание средств
- Рекомендуется ручная проверка для крупных сумм
- Используйте таймауты для защиты от мошенничества

## Интеграция с умным созданием объявлений

```python
# Создаем объявление
from scripts.bybit_smart_ad_creator import SmartAdCreator
creator = SmartAdCreator(api_key, api_secret)
result = creator.create_smart_ad({"amount": "10000"})

# Мониторим новые ордера по этому объявлению
ad_id = result['result']['itemId']
# ... проверяем ордера связанные с ad_id
```

## Требования

- Python 3.7+
- Библиотеки: requests, hmac, hashlib, uuid
- Bybit API ключи с разрешениями FiatP2P
- Активные P2P объявления или ордера