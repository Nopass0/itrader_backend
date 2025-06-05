# 🚀 Полная рабочая система Auto-Trader

## Обзор системы

Это полностью функциональная система автоматической торговли между Gate.io и Bybit P2P, включающая:

- ✅ Мониторинг транзакций Gate.io
- ✅ Создание объявлений на Bybit P2P
- ✅ Автоматизированный чат с покупателями на русском языке
- ✅ Проверка согласия покупателя с правилами P2P
- ✅ Показ реквизитов только после подтверждения
- ✅ Мониторинг email для получения чеков
- ✅ Валидация чеков от noreply@tinkoff.ru
- ✅ OCR обработка PDF чеков

## Структура проекта

```
Python система (рабочая):
├── trader_system.py      # Главное приложение
├── gate_client.py        # Работа с Gate.io через cookies
├── bybit_client.py       # Интеграция с Bybit P2P API
├── chat_manager.py       # Автоматизация чата (русский язык)
├── email_monitor.py      # Мониторинг email для чеков
├── ocr_processor.py      # OCR обработка чеков
├── config.py            # Управление конфигурацией
├── models.py            # Модели данных
├── utils.py             # Утилиты
├── requirements.txt     # Python зависимости
├── setup.sh            # Скрипт установки
├── run.sh              # Скрипт запуска
├── start_trader.sh     # Удобный запуск с выбором режима
└── config/
    └── default.toml    # Конфигурация по умолчанию

Вспомогательные файлы:
├── python_modules/
│   └── bybit_wrapper.py  # Обертка для Bybit SDK
├── .gate_cookies.json    # Cookies от Gate.io (создать вручную)
├── .env                  # Переменные окружения (создать из .env.trader)
└── .env.trader          # Пример переменных окружения
```

## Быстрый старт

### 1. Установка
```bash
chmod +x setup.sh run.sh start_trader.sh
./setup.sh
```

### 2. Настройка
```bash
# Скопировать и настроить переменные окружения
cp .env.trader .env
nano .env

# Добавить cookies от Gate.io в .gate_cookies.json
```

### 3. Запуск
```bash
# Ручной режим (рекомендуется для начала)
./start_trader.sh

# Автоматический режим
./start_trader.sh --auto
```

## Рабочий процесс системы

### 1. Обнаружение транзакции
```
[15:45:23] 👀 Checking Gate.io for pending transactions...
[15:45:24] 📦 Found 1 pending transaction
[15:45:24] 🆕 New transaction: 932.84 USDT → 75000.00 RUB @ 80.45
```

### 2. Создание объявления (ручной режим)
```
============================================================
📊 TRANSACTION DETAILS
============================================================
Transaction ID: GATE-TX-123456
Amount: 932.84 USDT → 75000.00 RUB
Rate: 80.45 RUB/USDT
Phone: +7900******67
Bank: Тинькофф

💡 Recommended rate: 82.06

Create Bybit P2P advertisement? [y/N]: y
Enter custom rate (default 82.06): [Enter]

✅ Advertisement created with ID: AD-789012
```

### 3. Автоматический чат с покупателем
```
[15:46:15] 🆕 New order ORDER-456 for ad AD-789012
[15:46:16] 📤 Sent greeting to order ORDER-456
[15:46:25] 💬 Buyer message: да, прочитал правила
[15:46:25] ✅ Buyer agreed to terms
[15:46:26] 💳 Sent payment details for order ORDER-456
```

### 4. Сообщения в чате

**Приветствие:**
```
Добрый день! Вы прочитали условия объявления и правила P2P?
```

**После согласия:**
```
✅ Спасибо за подтверждение!

💳 Реквизиты для оплаты:
Банк: Тинькофф
Телефон: +7 900 123-45-67
Сумма: 75000.00 RUB

📧 ВАЖНО: После оплаты обязательно отправьте чек на email: receipts@example.com

⚠️ Чек должен прийти с адреса noreply@tinkoff.ru
⏰ Время на оплату: 15 минут

После получения и проверки чека я подтвержу получение платежа.
```

**Напоминание (через 5 минут):**
```
⏰ Напоминаю:
- Отправьте чек на email после оплаты
- Чек должен прийти с адреса noreply@tinkoff.ru
- Осталось времени: 10 минут
```

### 5. Получение и проверка чека
```
[15:48:30] 📧 Checking emails...
[15:48:31] 📨 Found new email from noreply@tinkoff.ru
[15:48:32] 📎 Processing PDF attachment: receipt_123.pdf
[15:48:33] ✅ Valid receipt: amount 75000.00 matches order
[15:48:34] 🎉 Order ORDER-456 completed successfully
```

## Конфигурация

### Переменные окружения (.env)
```env
# Bybit API
BYBIT_API_KEY=your_api_key
BYBIT_API_SECRET=your_api_secret

# Реквизиты для платежей
PAYMENT_BANK=Тинькофф
PAYMENT_PHONE=+7 900 123-45-67
RECEIPT_EMAIL=your-receipts@gmail.com

# Email для мониторинга
EMAIL_USERNAME=your-email@gmail.com
EMAIL_PASSWORD=app-specific-password
```

### Настройки торговли (config/default.toml)
```toml
[trading]
profit_margin_percent = 2.0  # Ваша маржа
min_order_amount = 1000.0    # Минимальная сумма
max_order_amount = 50000.0   # Максимальная сумма
```

## Безопасность

- API ключи хранятся в переменных окружения
- Gate.io использует cookie аутентификацию
- Email использует app-specific пароли
- Чеки принимаются только от noreply@tinkoff.ru

## Режимы работы

### Ручной режим (по умолчанию)
- Требует подтверждения для каждого действия
- Позволяет изменить курс перед созданием объявления
- Безопасен для тестирования

### Автоматический режим
- Полностью автоматическая работа
- Использует рекомендованные курсы
- Требует двойного подтверждения при запуске

## Мониторинг и логи

```bash
# Просмотр логов в реальном времени
tail -f trader.log

# Проверка активных процессов
ps aux | grep trader_system.py
```

## Обработка ошибок

- Автоматическое переподключение при сбоях
- Повторные попытки для критических операций
- Детальное логирование всех действий
- Graceful shutdown при Ctrl+C

## Поддержка

При возникновении проблем:
1. Проверьте логи в `trader.log`
2. Убедитесь что `.gate_cookies.json` актуальный
3. Проверьте права доступа API ключей
4. Убедитесь в правильности email настроек

Система полностью готова к работе!