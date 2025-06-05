# Настройка аккаунтов iTrader

Система хранит все данные аккаунтов в JSON файлах в папке `db/`.

## Структура папок

```
db/
├── gate/          # Gate.io аккаунты
│   └── {id}.json  # Файлы с данными каждого аккаунта
├── bybit/         # Bybit аккаунты  
│   └── {id}.json  # Файлы с данными каждого аккаунта
├── gmail/         # Gmail credentials
├── transactions/  # История транзакций
└── checks/        # Сохраненные чеки
```

## Добавление Gate.io аккаунта

### Вариант 1: Через WebSocket API
```bash
# Запустите приложение
./run_with_python.sh

# В другом терминале подключитесь через WebSocket
wscat -c ws://localhost:8080/ws

# Отправьте команду
{
  "type": "add_gate_account",
  "data": {
    "login": "user@example.com",
    "password": "password123"
  }
}
```

### Вариант 2: Создайте JSON файл вручную
Создайте файл `db/gate/{uuid}.json`:
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "login": "user@example.com",
  "password": "your_password",
  "cookies": null,
  "last_auth": null,
  "balance": 0.0,
  "created_at": "2025-01-04T12:00:00Z",
  "updated_at": "2025-01-04T12:00:00Z"
}
```

## Добавление Bybit аккаунта

### Вариант 1: Через WebSocket API
```bash
{
  "type": "add_bybit_account",
  "data": {
    "api_key": "your_api_key",
    "api_secret": "your_api_secret"
  }
}
```

### Вариант 2: Создайте JSON файл вручную
Создайте файл `db/bybit/{uuid}.json`:
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440001",
  "api_key": "your_api_key",
  "api_secret": "your_api_secret",
  "active_ads": 0,
  "last_used": null,
  "created_at": "2025-01-04T12:00:00Z",
  "updated_at": "2025-01-04T12:00:00Z"
}
```

## Настройка Gmail

1. Получите `credentials.json` из Google Cloud Console
2. Поместите файл в `db/gmail/credentials.json`
3. Запустите авторизацию:
   ```bash
   cargo run --bin gmail_auth
   ```
4. Токен сохранится в `db/gmail/token.json`

## Безопасность

⚠️ **ВАЖНО**: Так как данные не шифруются:
- Ограничьте доступ к папке `db/` на уровне файловой системы
- Используйте `chmod 700 db/` для ограничения доступа
- Не храните `db/` в публичных репозиториях
- Регулярно делайте резервные копии

## Проверка работы

После добавления аккаунтов система автоматически:
1. Авторизуется в Gate.io при запуске
2. Установит баланс 10M RUB на всех Gate аккаунтах
3. Начнет мониторинг транзакций
4. Будет создавать объявления на Bybit при появлении новых транзакций