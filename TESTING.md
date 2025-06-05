# Тестирование iTrader Backend

## Подготовка окружения

### 1. Настройка PostgreSQL

Убедитесь, что PostgreSQL запущен и пользователь `postgres` имеет пароль `root`:

```bash
# Установка пароля для пользователя postgres
sudo -u postgres psql -c "ALTER USER postgres PASSWORD 'root';"

# Проверка подключения
PGPASSWORD=root psql -h localhost -U postgres -c "SELECT 1;"
```

### 2. Запуск сервисов

```bash
# PostgreSQL
sudo systemctl start postgresql
# или на macOS
brew services start postgresql

# Redis
sudo systemctl start redis
# или на macOS
brew services start redis
```

### 3. Подготовка тестовых данных

Убедитесь, что в папке `test_data/` есть актуальные данные:
- `gate_cookie.json` - куки для Gate.io
- `bybit_creditials.json` - API ключи для Bybit

## Запуск в режиме разработки

```bash
./dev.sh
```

Скрипт автоматически:
- Создаст `.env` файл с значениями по умолчанию
- Создаст базу данных `itrader_db` если её нет
- Запустит миграции
- Запустит приложение с hot reload

## Запуск тестов

### Все тесты
```bash
./test.sh all
```

### Тесты Gate.io

```bash
# Тест авторизации через куки
./test.sh gate-auth

# Тест получения транзакций
./test.sh gate-tx

# Тест установки баланса
./test.sh gate-balance

# Все тесты Gate.io
./test.sh gate-all
```

### Тесты Bybit

```bash
# Тест авторизации
./test.sh bybit-auth

# Тест получения объявлений
./test.sh bybit-ads

# Проверка доступности аккаунта
./test.sh bybit-available

# Тест получения P2P заказов
./test.sh bybit-orders

# Все тесты Bybit
./test.sh bybit-all
```

## Примеры вывода тестов

### Успешная авторизация Gate.io:
```
=== Testing Gate.io Authentication with Cookies ===
Loaded 2 cookies from test data
✓ Cookies set successfully
✓ Successfully authenticated with cookies
  Current balance: 1000000.00 RUB
  Total balance: 1000000.00 RUB
  Locked: 0.00 RUB
```

### Успешная авторизация Bybit:
```
=== Testing Bybit P2P Authentication ===
Loaded API credentials from test data
  API Key: nSkH1BHV...
✓ Successfully authenticated with Bybit
  Account ID: 12345
  Nickname: TestUser
  Active ads: 0
  Status: active
```

## Возможные проблемы

### 1. Expired cookies (Gate.io)
Если видите ошибку:
```
✗ Failed to authenticate with cookies: Session expired
  Session expired, please update cookies
```

Нужно обновить куки в `test_data/gate_cookie.json`

### 2. Invalid API credentials (Bybit)
Если видите ошибку:
```
✗ Failed to authenticate with Bybit: Invalid API credentials
```

Проверьте API ключи в `test_data/bybit_creditials.json`

### 3. Database connection error
Если видите ошибку подключения к БД:
```
Failed to create database. Make sure PostgreSQL user 'postgres' with password 'root' exists
```

Установите пароль для пользователя postgres:
```bash
sudo -u postgres psql -c "ALTER USER postgres PASSWORD 'root';"
```

## Структура тестов

```
tests/
├── common/
│   └── mod.rs         # Общие утилиты для тестов
├── gate_tests.rs      # Тесты Gate.io
└── bybit_tests.rs     # Тесты Bybit
```

## Добавление новых тестов

1. Создайте новый файл в `tests/`
2. Используйте `mod common;` для доступа к утилитам
3. Используйте `#[tokio::test]` для async тестов
4. Добавьте новую команду в `test.sh`

Пример:
```rust
mod common;

#[tokio::test]
async fn test_new_feature() {
    common::setup();
    // Ваш тест
}
```