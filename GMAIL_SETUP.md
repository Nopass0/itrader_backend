# Настройка Gmail API

## 1. Получение credentials.json

1. Перейдите в [Google Cloud Console](https://console.cloud.google.com/)
2. Создайте новый проект или выберите существующий
3. Включите Gmail API:
   - Перейдите в "APIs & Services" > "Library"
   - Найдите "Gmail API" и нажмите "Enable"
4. Создайте учетные данные:
   - Перейдите в "APIs & Services" > "Credentials"
   - Нажмите "Create Credentials" > "OAuth client ID"
   - Выберите "Desktop app"
   - Скачайте JSON файл

## 2. Размещение файлов

```bash
# Создайте папку для Gmail
mkdir -p db/gmail

# Поместите credentials.json
cp ~/Downloads/credentials.json db/gmail/credentials.json
```

## 3. Первая авторизация

```bash
# Запустите авторизацию Gmail
cargo run --bin gmail_auth

# Или используйте скрипт
./gmail_setup.sh
```

Это откроет браузер для авторизации. После авторизации токен будет сохранен в `db/gmail/token.json`.

## 4. Проверка работы

```bash
# Проверить последние письма
cargo test test_gmail_list_emails_today -- --ignored --nocapture

# Получить последний PDF
cargo test test_gmail_get_latest_pdf -- --ignored --nocapture
```

## Как работает система

1. **Мониторинг почты**: Система проверяет новые письма каждые 30 секунд
2. **Фильтрация**: Ищет письма от `receipts@tbank.ru` или других банков
3. **Извлечение PDF**: Автоматически скачивает PDF вложения
4. **OCR обработка**: Извлекает данные из чека
5. **Валидация**: Сравнивает с транзакцией Gate.io

## Формат хранения

Все данные Gmail хранятся в папке `db/gmail/`:
- `credentials.json` - OAuth2 credentials от Google
- `token.json` - Токен доступа (создается автоматически)

## Безопасность

- Токены шифруются с помощью ENCRYPTION_KEY
- Refresh token позволяет обновлять доступ без повторной авторизации
- Требуется только read-only доступ к Gmail