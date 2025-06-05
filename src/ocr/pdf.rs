use anyhow::{Result, Context};
use chrono::{DateTime, Utc, NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use std::path::Path;
use regex::Regex;
use lazy_static::lazy_static;

// Public function for external use
pub fn extract_text_from_pdf<P: AsRef<Path>>(path: P) -> Result<String> {
    pdf_extract::extract_text(path.as_ref())
        .or_else(|_| {
            // Fallback to lopdf
            let doc = lopdf::Document::load(path.as_ref())?;
            let mut text = String::new();
            let pages = doc.get_pages();
            
            for (_, page_id) in pages {
                if let Ok(page_text) = doc.extract_text(&[page_id.0]) {
                    text.push_str(&page_text);
                    text.push('\n');
                }
            }
            
            Ok(text)
        })
}

lazy_static! {
    // Common Russian bank transfer patterns
    static ref AMOUNT_REGEX: Regex = Regex::new(r"(?i)(?:сумма|amount|итого|total)[:\s]*([0-9\s]+[,.]?[0-9]*)\s*(?:руб|rub|₽)?").unwrap();
    static ref DATE_REGEX: Regex = Regex::new(r"(\d{1,2}[./]\d{1,2}[./]\d{2,4}|\d{4}-\d{2}-\d{2})").unwrap();
    static ref TIME_REGEX: Regex = Regex::new(r"(\d{1,2}:\d{2}(?::\d{2})?)").unwrap();
    static ref REFERENCE_REGEX: Regex = Regex::new(r"(?i)(?:номер операции|transaction|операция|reference|ref)[:\s]*([A-Za-z0-9\-]+)").unwrap();
    static ref BANK_REGEX: Regex = Regex::new(r"(?i)(сбербанк|sberbank|тинькофф|tinkoff|т-банк|t-bank|альфа-банк|alfa-bank|втб|vtb|райффайзен|raiffeisen)").unwrap();
    static ref RECIPIENT_REGEX: Regex = Regex::new(r"(?i)(?:получатель|recipient|кому)[:\s]*([А-Яа-яA-Za-z\s]+)").unwrap();
    static ref SENDER_REGEX: Regex = Regex::new(r"(?i)(?:отправитель|sender|от кого)[:\s]*([А-Яа-яA-Za-z\s]+)").unwrap();
    static ref CARD_REGEX: Regex = Regex::new(r"(?:\*{4}\s*\d{4}|\d{4}\s*\*{4}\s*\*{4}\s*\d{4})").unwrap();
    static ref PHONE_REGEX: Regex = Regex::new(r"(?:\+7|8)?[\s\-]?\(?(\d{3})\)?[\s\-]?(\d{3})[\s\-]?(\d{2})[\s\-]?(\d{2})").unwrap();
    static ref STATUS_REGEX: Regex = Regex::new(r"(?i)(?:статус|status)[:\s]*([А-Яа-яA-Za-z\s]+)|(?:выполнен|completed|успешно|successful|исполнен|executed|завершен|finished)").unwrap();
}

#[derive(Debug, Clone)]
pub struct ReceiptInfo {
    pub date_time: DateTime<Utc>,
    pub amount: Decimal,
    pub recipient: Option<String>,
    pub sender: Option<String>,
    pub transaction_id: Option<String>,
    pub bank_name: Option<String>,
    pub card_number: Option<String>,
    pub phone_number: Option<String>,
    pub status: Option<String>,
    pub raw_text: String,
}

pub struct PdfReceiptParser;

impl PdfReceiptParser {
    pub fn new() -> Self {
        Self
    }

    pub async fn parse_receipt<P: AsRef<Path>>(&self, pdf_path: P) -> Result<ReceiptInfo> {
        let path = pdf_path.as_ref();
        
        // Extract text from PDF
        let text = self.extract_text_from_pdf(path)?;
        
        // Parse the extracted text
        self.parse_receipt_text(&text)
    }

    fn extract_text_from_pdf(&self, path: &Path) -> Result<String> {
        // Try pdf-extract first
        match pdf_extract::extract_text(path) {
            Ok(text) => Ok(text),
            Err(_) => {
                // Fallback to lopdf
                self.extract_with_lopdf(path)
            }
        }
    }

    fn extract_with_lopdf(&self, path: &Path) -> Result<String> {
        use lopdf::Document;
        
        let doc = Document::load(path)
            .context("Failed to load PDF document")?;
        
        let mut text = String::new();
        let pages = doc.get_pages();
        
        for (_, page_id) in pages {
            // page_id is a tuple (u32, u16), we need just the first element
            if let Ok(page_text) = doc.extract_text(&[page_id.0]) {
                text.push_str(&page_text);
                text.push('\n');
            }
        }
        
        Ok(text)
    }

    fn parse_receipt_text(&self, text: &str) -> Result<ReceiptInfo> {
        // Clean and normalize text
        let normalized_text = self.normalize_text(text);
        
        
        // Extract amount
        let amount = self.extract_amount(&normalized_text)
            .context("Failed to extract amount from receipt")?;
        
        // Extract date and time
        let date_time = self.extract_datetime(&normalized_text)
            .unwrap_or_else(|| Utc::now());
        
        // Extract other fields
        let transaction_id = self.extract_transaction_id(&normalized_text);
        let bank_name = self.extract_bank_name(&normalized_text);
        let recipient = self.extract_recipient(&normalized_text);
        let sender = self.extract_sender(&normalized_text);
        let card_number = self.extract_card_number(&normalized_text);
        let phone_number = self.extract_phone_number(&normalized_text);
        let status = self.extract_status(&normalized_text);
        
        Ok(ReceiptInfo {
            date_time,
            amount,
            recipient,
            sender,
            transaction_id,
            bank_name,
            card_number,
            phone_number,
            status,
            raw_text: text.to_string(),
        })
    }

    fn normalize_text(&self, text: &str) -> String {
        text.replace('\u{00a0}', " ") // Replace non-breaking spaces
            .replace('\r', "")
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn extract_amount(&self, text: &str) -> Result<Decimal> {
        if let Some(caps) = AMOUNT_REGEX.captures(text) {
            if let Some(amount_str) = caps.get(1) {
                let cleaned = amount_str.as_str()
                    .replace(' ', "")
                    .replace(',', ".");
                
                return Decimal::from_str_exact(&cleaned)
                    .context("Failed to parse amount as decimal");
            }
        }
        
        // Try to find any number that looks like an amount
        let amount_pattern = Regex::new(r"(\d{1,3}(?:\s?\d{3})*(?:[,.]\d{2})?)\s*(?:руб|rub|₽)").unwrap();
        if let Some(caps) = amount_pattern.captures(text) {
            if let Some(amount_str) = caps.get(1) {
                let cleaned = amount_str.as_str()
                    .replace(' ', "")
                    .replace(',', ".");
                
                return Decimal::from_str_exact(&cleaned)
                    .context("Failed to parse amount as decimal");
            }
        }
        
        anyhow::bail!("No amount found in receipt")
    }

    fn extract_datetime(&self, text: &str) -> Option<DateTime<Utc>> {
        let date = DATE_REGEX.captures(text)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str());
        
        let time = TIME_REGEX.captures(text)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str())
            .unwrap_or("00:00");
        
        if let Some(date_str) = date {
            // Try different date formats
            let formats = vec![
                "%d.%m.%Y %H:%M:%S",
                "%d.%m.%Y %H:%M",
                "%d/%m/%Y %H:%M:%S",
                "%d/%m/%Y %H:%M",
                "%Y-%m-%d %H:%M:%S",
                "%Y-%m-%d %H:%M",
                "%d.%m.%y %H:%M:%S",
                "%d.%m.%y %H:%M",
            ];
            
            let datetime_str = format!("{} {}", date_str, time);
            
            for format in formats {
                if let Ok(naive_dt) = NaiveDateTime::parse_from_str(&datetime_str, format) {
                    return Some(DateTime::from_naive_utc_and_offset(naive_dt, Utc));
                }
            }
        }
        
        None
    }

    fn extract_transaction_id(&self, text: &str) -> Option<String> {
        REFERENCE_REGEX.captures(text)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().trim().to_string())
    }

    fn extract_bank_name(&self, text: &str) -> Option<String> {
        // Special case: T-Bank receipts don't have bank field
        if text.contains("fb@tbank.ru") && !text.contains("Банк получателя") {
            return Some("T-Банк (Тинькофф)".to_string());
        }
        
        // First try to find "Банк получателя <bank name>" pattern
        let bank_patterns = vec![
            // Pattern for short bank names like "ПСБ", "МТС-Банк" - stop at next field
            r"Банк\s*получателя\s*([А-ЯA-Z][А-Яа-яёЁA-Za-z\-]*(?:\s+[А-Яа-яёЁA-Za-z\-]+)?)(?:\s+(?:Счет|Идентификатор|$))",
            // Pattern for compressed text - specifically match bank name before "Счет"
            r"Банк\s*получателя\s*([А-Яа-яёЁ][а-яёЁ]+)(?:\s+Счет)",
            // Pattern for bank name that may have spaces
            r"Банк\s*получателя\s*([А-Яа-яёЁ][а-яёЁ]+(?:-[А-Яа-яёЁ][а-яёЁ]+)?)",
            // Pattern for text without spaces between fields
            r"Банк\s*получателя\s*([А-Яа-яёЁA-Za-z][А-Яа-яёЁA-Za-z\s\-\.]*?)(?=\s*Счет|\s*Идентификатор|$)",
        ];
        
        for pattern in bank_patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if let Some(caps) = regex.captures(text) {
                    if let Some(bank_match) = caps.get(1) {
                        let bank_name = bank_match.as_str().trim();
                        return Some(self.normalize_bank_name(bank_name));
                    }
                }
            }
        }
        
        // Try alternative patterns
        let alt_patterns = vec![
            r"(?i)банк[:?\s]+([А-Яа-яёЁA-Za-z][А-Яа-яёЁA-Za-z\s\-\.]+?)(?=\s*\n|\s*$)",
            r"(?i)в\s+банке[:?\s]+([А-Яа-яёЁA-Za-z][А-Яа-яёЁA-Za-z\s\-\.]+?)(?=\s*\n|\s*$)",
        ];
        
        for pattern in alt_patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if let Some(caps) = regex.captures(text) {
                    if let Some(bank_match) = caps.get(1) {
                        let bank_name = bank_match.as_str().trim();
                        return Some(self.normalize_bank_name(bank_name));
                    }
                }
            }
        }
        
        // Fallback to original bank regex
        BANK_REGEX.captures(text)
            .and_then(|caps| caps.get(1))
            .map(|m| {
                let bank_name = m.as_str().trim();
                self.normalize_bank_name(bank_name)
            })
    }

    fn extract_recipient(&self, text: &str) -> Option<String> {
        RECIPIENT_REGEX.captures(text)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().trim().to_string())
    }

    fn extract_sender(&self, text: &str) -> Option<String> {
        SENDER_REGEX.captures(text)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().trim().to_string())
    }

    fn extract_card_number(&self, text: &str) -> Option<String> {
        CARD_REGEX.captures(text)
            .and_then(|caps| caps.get(0))
            .map(|m| m.as_str().trim().to_string())
    }

    fn extract_phone_number(&self, text: &str) -> Option<String> {
        PHONE_REGEX.captures(text)
            .map(|caps| {
                let area = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                let prefix = caps.get(2).map(|m| m.as_str()).unwrap_or("");
                let line1 = caps.get(3).map(|m| m.as_str()).unwrap_or("");
                let line2 = caps.get(4).map(|m| m.as_str()).unwrap_or("");
                format!("+7 {} {}-{}-{}", area, prefix, line1, line2)
            })
    }

    fn extract_status(&self, text: &str) -> Option<String> {
        if let Some(caps) = STATUS_REGEX.captures(text) {
            if let Some(status) = caps.get(1) {
                // If we captured a status after "статус:" or "status:"
                return Some(status.as_str().trim().to_string());
            } else {
                // If we matched status keywords like "выполнен", "completed", etc.
                return Some("Выполнен".to_string());
            }
        }
        None
    }
    
    fn normalize_bank_name(&self, bank: &str) -> String {
        let bank_upper = bank.to_uppercase();
        let bank_trimmed = bank_upper.trim();
        
        match bank_trimmed {
            // Сбербанк
            "СБЕРБАНК" | "СБЕР" | "SBERBANK" | "SBER" | "СБ РФ" | "СБЕРБАНК РОССИИ" => "Сбербанк".to_string(),
            
            // T-Банк (Тинькофф)
            "Т-БАНК" | "T-BANK" | "ТИНЬКОФФ" | "ТИНЬК" | "TINKOFF" | "Т БАНК" | "ТИНЬКОФФ БАНК" => "T-Банк (Тинькофф)".to_string(),
            
            // РНКБ Банк
            "РНКБ" | "РНКБ БАНК" | "RNKB" => "РНКБ Банк".to_string(),
            
            // Райффайзенбанк
            "РАЙФФАЙЗЕН" | "РАЙФФАЙЗЕНБАНК" | "RAIFFEISEN" | "RAIFFEISENBANK" | "РАЙ" => "Райффайзенбанк".to_string(),
            
            // БАНК УРАЛСИБ
            "УРАЛСИБ" | "БАНК УРАЛСИБ" | "URALSIB" => "БАНК УРАЛСИБ".to_string(),
            
            // Озон Банк
            "ОЗОН" | "ОЗОН БАНК" | "OZON" | "OZON BANK" | "ОЗОНБАНК" => "Озон Банк (Ozon)".to_string(),
            
            // КБ УБРиР
            "УБРИР" | "КБ УБРИР" | "UBRIR" | "УРАЛЬСКИЙ БАНК" => "КБ УБРиР".to_string(),
            
            // Цифра банк
            "ЦИФРА" | "ЦИФРА БАНК" | "CIFRA" | "ЦИФРАБАНК" => "Цифра банк".to_string(),
            
            // Банк ДОМ.РФ
            "ДОМ.РФ" | "БАНК ДОМ.РФ" | "ДОМ РФ" | "ДОМРФ" => "Банк ДОМ.РФ".to_string(),
            
            // Газпромбанк
            "ГАЗПРОМБАНК" | "ГАЗПРОМ" | "ГПБ" | "GAZPROMBANK" | "GPB" => "Газпромбанк".to_string(),
            
            // АКБ Абсолют Банк
            "АБСОЛЮТ" | "АБСОЛЮТ БАНК" | "АКБ АБСОЛЮТ БАНК" | "ABSOLUT" => "АКБ Абсолют Банк".to_string(),
            
            // АЛЬФА-БАНК
            "АЛЬФА-БАНК" | "АЛЬФА" | "ALFA-BANK" | "ALFA" | "АЛЬФАБАНК" | "АЛЬФА БАНК" => "АЛЬФА-БАНК".to_string(),
            
            // Банк ВТБ
            "ВТБ" | "VTB" | "БАНК ВТБ" | "ВТБ БАНК" => "Банк ВТБ".to_string(),
            
            // АК БАРС БАНК
            "АК БАРС" | "АК БАРС БАНК" | "АКБАРС" | "AK BARS" => "АК БАРС БАНК".to_string(),
            
            // Хоум кредит
            "ХОУМ КРЕДИТ" | "ХОУМ" | "HOME CREDIT" | "ХОУМКРЕДИТ" => "Хоум кредит".to_string(),
            
            // РОСБАНК
            "РОСБАНК" | "ROSBANK" => "РОСБАНК".to_string(),
            
            // ОТП Банк
            "ОТП" | "ОТП БАНК" | "OTP" | "OTP BANK" => "ОТП Банк".to_string(),
            
            // КБ Ренессанс Кредит
            "РЕНЕССАНС" | "РЕНЕССАНС КРЕДИТ" | "КБ РЕНЕССАНС КРЕДИТ" | "RENAISSANCE" => "КБ Ренессанс Кредит".to_string(),
            
            // Банк ЗЕНИТ
            "ЗЕНИТ" | "БАНК ЗЕНИТ" | "ZENIT" => "Банк ЗЕНИТ".to_string(),
            
            // Россельхозбанк
            "РОССЕЛЬХОЗБАНК" | "РСХБ" | "РОССЕЛЬХОЗ" | "RSHB" => "Россельхозбанк".to_string(),
            
            // Промсвязьбанк
            "ПРОМСВЯЗЬБАНК" | "ПСБ" | "ПРОМСВЯЗЬ" | "PSB" => "Промсвязьбанк".to_string(),
            
            // Почта Банк
            "ПОЧТА БАНК" | "ПОЧТА" | "ПОЧТАБАНК" | "POCHTABANK" => "Почта Банк".to_string(),
            
            // МТС-Банк
            "МТС-БАНК" | "МТС" | "MTS-BANK" | "MTS" | "МТСБАНК" => "МТС-Банк".to_string(),
            
            // Банк Русский Стандарт
            "РУССКИЙ СТАНДАРТ" | "БАНК РУССКИЙ СТАНДАРТ" | "РС" | "RUSSIAN STANDARD" => "Банк Русский Стандарт".to_string(),
            
            // АКБ АВАНГАРД
            "АВАНГАРД" | "АКБ АВАНГАРД" | "AVANGARD" => "АКБ АВАНГАРД".to_string(),
            
            // КБ Солидарность
            "СОЛИДАРНОСТЬ" | "КБ СОЛИДАРНОСТЬ" | "SOLIDARNOST" => "КБ Солидарность".to_string(),
            
            // Дальневосточный банк
            "ДАЛЬНЕВОСТОЧНЫЙ" | "ДАЛЬНЕВОСТОЧНЫЙ БАНК" | "ДВБ" | "DVB" => "Дальневосточный банк".to_string(),
            
            // ББР Банк
            "ББР" | "ББР БАНК" | "BBR" => "ББР Банк".to_string(),
            
            // ЮниКредит Банк
            "ЮНИКРЕДИТ" | "ЮНИКРЕДИТ БАНК" | "UNICREDIT" => "ЮниКредит Банк".to_string(),
            
            // ГЕНБАНК
            "ГЕНБАНК" | "ГЕН" | "GENBANK" => "ГЕНБАНК".to_string(),
            
            // ЦМРБанк
            "ЦМРБАНК" | "ЦМР" | "CMR" | "ЦМР БАНК" => "ЦМРБанк".to_string(),
            
            // Свой Банк
            "СВОЙ БАНК" | "СВОЙ" | "СВОЙБАНК" => "Свой Банк".to_string(),
            
            // Ингосстрах Банк
            "ИНГОССТРАХ" | "ИНГОССТРАХ БАНК" | "INGOSSTRAKH" => "Ингосстрах Банк".to_string(),
            
            // МОСКОВСКИЙ КРЕДИТНЫЙ БАНК
            "МКБ" | "МОСКОВСКИЙ КРЕДИТНЫЙ БАНК" | "МКБ БАНК" | "MKB" => "МОСКОВСКИЙ КРЕДИТНЫЙ БАНК".to_string(),
            
            // Совкомбанк
            "СОВКОМБАНК" | "СОВКОМ" | "SOVCOMBANK" | "SOVCOM" => "Совкомбанк".to_string(),
            
            // КБ Модульбанк
            "МОДУЛЬБАНК" | "КБ МОДУЛЬБАНК" | "МОДУЛЬ" | "MODULBANK" => "КБ Модульбанк".to_string(),
            
            // Яндекс Банк
            "ЯНДЕКС БАНК" | "ЯНДЕКС" | "YANDEX BANK" | "ЯНДЕКСБАНК" => "Яндекс Банк".to_string(),
            
            // КБ ЮНИСТРИМ
            "ЮНИСТРИМ" | "КБ ЮНИСТРИМ" | "UNISTREAM" => "КБ ЮНИСТРИМ".to_string(),
            
            // КБ ПОЙДЁМ!
            "ПОЙДЁМ!" | "КБ ПОЙДЁМ!" | "ПОЙДЕМ" | "POIDEM" => "КБ ПОЙДЁМ!".to_string(),
            
            // Первый инвестиционный банк
            "ПЕРВЫЙ ИНВЕСТИЦИОННЫЙ" | "ПЕРВЫЙ ИНВЕСТИЦИОННЫЙ БАНК" | "ПИБ" => "Первый инвестиционный банк".to_string(),
            
            // ИШБАНК
            "ИШБАНК" | "ИШ" | "ISHBANK" => "ИШБАНК".to_string(),
            
            // КБ СИНАРА
            "СИНАРА" | "КБ СИНАРА" | "SINARA" => "КБ СИНАРА".to_string(),
            
            // ТРАНСКАПИТАЛБАНК
            "ТРАНСКАПИТАЛБАНК" | "ТКБ" | "ТРАНСКАПИТАЛ" | "TKB" => "ТРАНСКАПИТАЛБАНК".to_string(),
            
            // СКБ-Банк
            "СКБ-БАНК" | "СКБ" | "SKB" => "СКБ-Банк".to_string(),
            
            // Банк Левобережный
            "ЛЕВОБЕРЕЖНЫЙ" | "БАНК ЛЕВОБЕРЕЖНЫЙ" => "Банк Левобережный".to_string(),
            
            // Новикомбанк
            "НОВИКОМБАНК" | "НОВИКОМ" | "NOVIKOMBANK" => "Новикомбанк".to_string(),
            
            // КБ Кубань Кредит
            "КУБАНЬ КРЕДИТ" | "КБ КУБАНЬ КРЕДИТ" | "КУБАНЬКРЕДИТ" => "КБ Кубань Кредит".to_string(),
            
            // ВСЕИНСТРУМЕНТЫ.РУ
            "ВСЕИНСТРУМЕНТЫ.РУ" | "ВСЕИНСТРУМЕНТЫ" => "ВСЕИНСТРУМЕНТЫ.РУ".to_string(),
            
            // АВТОГРАДБАНК
            "АВТОГРАДБАНК" | "АВТОГРАД" | "AVTOGRAD" => "АВТОГРАДБАНК".to_string(),
            
            // АКБ Связь-Банк
            "СВЯЗЬ-БАНК" | "АКБ СВЯЗЬ-БАНК" | "СВЯЗЬБАНК" => "АКБ Связь-Банк".to_string(),
            
            // Банк Санкт-Петербург
            "БАНК САНКТ-ПЕТЕРБУРГ" | "БСПБ" | "САНКТ-ПЕТЕРБУРГ" => "Банк Санкт-Петербург".to_string(),
            
            // СДМ-Банк
            "СДМ-БАНК" | "СДМ" | "SDM" => "СДМ-Банк".to_string(),
            
            // Экспобанк
            "ЭКСПОБАНК" | "ЭКСПО" | "EXPOBANK" => "Экспобанк".to_string(),
            
            // АКБ Металлинвестбанк
            "МЕТАЛЛИНВЕСТБАНК" | "АКБ МЕТАЛЛИНВЕСТБАНК" | "МЕТАЛЛИНВЕСТ" => "АКБ Металлинвестбанк".to_string(),
            
            // МОСОБЛБАНК
            "МОСОБЛБАНК" | "МОСОБЛ" => "МОСОБЛБАНК".to_string(),
            
            // КОШЕЛЕВ-БАНК
            "КОШЕЛЕВ-БАНК" | "КОШЕЛЕВ" => "КОШЕЛЕВ-БАНК".to_string(),
            
            // РУСНАРБАНК
            "РУСНАРБАНК" | "РУСНАР" => "РУСНАРБАНК".to_string(),
            
            // Банк СОЮЗ
            "БАНК СОЮЗ" | "СОЮЗ" | "SOYUZ" => "Банк СОЮЗ".to_string(),
            
            // БСТ-БАНК
            "БСТ-БАНК" | "БСТ" | "BST" => "БСТ-БАНК".to_string(),
            
            // Банк БКФ
            "БКФ" | "БАНК БКФ" | "BKF" => "Банк БКФ".to_string(),
            
            // ЭНЕРГОТРАНСБАНК
            "ЭНЕРГОТРАНСБАНК" | "ЭТБ" | "ЭНЕРГОТРАНС" => "ЭНЕРГОТРАНСБАНК".to_string(),
            
            // Плюс Банк
            "ПЛЮС БАНК" | "ПЛЮС" | "PLUS BANK" => "Плюс Банк".to_string(),
            
            // СНГБ
            "СНГБ" | "SNGB" => "СНГБ".to_string(),
            
            // КБ ПРИСКО КАПИТАЛ БАНК
            "ПРИСКО" | "ПРИСКО КАПИТАЛ БАНК" | "КБ ПРИСКО КАПИТАЛ БАНК" => "КБ ПРИСКО КАПИТАЛ БАНК".to_string(),
            
            // КБ ЛОКО-Банк
            "ЛОКО-БАНК" | "КБ ЛОКО-БАНК" | "ЛОКО" | "LOKO" => "КБ ЛОКО-Банк".to_string(),
            
            // Банк Раунд
            "РАУНД" | "БАНК РАУНД" | "ROUND" => "Банк Раунд".to_string(),
            
            // МС Банк Рус
            "МС БАНК РУС" | "МС БАНК" | "MS BANK" => "МС Банк Рус".to_string(),
            
            // ВЛАДБИЗНЕСБАНК
            "ВЛАДБИЗНЕСБАНК" | "ВББ" => "ВЛАДБИЗНЕСБАНК".to_string(),
            
            // КБ Кубань Кредит
            "БАНК ФИНСЕРВИС" | "ФИНСЕРВИС" => "Банк Финсервис".to_string(),
            
            // Москоммерцбанк
            "МОСКОММЕРЦБАНК" | "МКМ" => "Москоммерцбанк".to_string(),
            
            // Уральский банк
            "УРАЛЬСКИЙ БАНК" | "УБ" => "Уральский банк".to_string(),
            
            // ФОРА-БАНК
            "ФОРА-БАНК" | "ФОРА" | "FORA" => "ФОРА-БАНК".to_string(),
            
            // Земский банк
            "ЗЕМСКИЙ БАНК" | "ЗЕМСКИЙ" => "Земский банк".to_string(),
            
            // СМП Банк
            "СМП БАНК" | "СМП" | "SMP" => "СМП Банк".to_string(),
            
            // Кредит Европа Банк
            "КРЕДИТ ЕВРОПА БАНК" | "КРЕДИТ ЕВРОПА" | "CREDIT EUROPE" => "Кредит Европа Банк".to_string(),
            
            // Кузнецкбизнесбанк
            "КУЗНЕЦКБИЗНЕСБАНК" | "КББ" => "Кузнецкбизнесбанк".to_string(),
            
            // ЧЕЛЯБИНВЕСТБАНК
            "ЧЕЛЯБИНВЕСТБАНК" | "ЧИБ" => "ЧЕЛЯБИНВЕСТБАНК".to_string(),
            
            // Банк РЕСО Кредит
            "РЕСО КРЕДИТ" | "БАНК РЕСО КРЕДИТ" | "РЕСО" => "Банк РЕСО Кредит".to_string(),
            
            // РТС-Банк
            "РТС-БАНК" | "РТС" | "RTS" => "РТС-Банк".to_string(),
            
            // КБ Геобанк
            "ГЕОБАНК" | "КБ ГЕОБАНК" | "ГЕО" => "КБ Геобанк".to_string(),
            
            // Банк РНКО
            "РНКО" | "БАНК РНКО" => "Банк РНКО".to_string(),
            
            // ТГБ
            "ТГБ" | "TGB" => "ТГБ".to_string(),
            
            // Банк РАУНД
            "БЖФ" | "БАНК БЖФ" => "Банк БЖФ".to_string(),
            
            // Default case - return original if no match found
            _ => bank.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_parse_receipt_files() {
        let parser = PdfReceiptParser::new();
        let test_data_dir = PathBuf::from("/home/user/projects/itrader_backend/test_data");
        
        let receipt_files = vec![
            "Receipt (8).pdf",
            "Receipt (9).pdf",
            "Receipt (10).pdf",
            "receipt_27.05.2025.pdf",
        ];
        
        for file_name in receipt_files {
            let file_path = test_data_dir.join(file_name);
            println!("\n=== Processing {} ===", file_name);
            
            match parser.parse_receipt(&file_path).await {
                Ok(info) => {
                    println!("Successfully parsed {}:", file_name);
                    println!("  Date/Time: {}", info.date_time);
                    println!("  Amount: {} RUB", info.amount);
                    println!("  Bank: {:?}", info.bank_name);
                    println!("  Transaction ID: {:?}", info.transaction_id);
                    println!("  Recipient: {:?}", info.recipient);
                    println!("  Sender: {:?}", info.sender);
                    println!("  Card: {:?}", info.card_number);
                    println!("  Phone: {:?}", info.phone_number);
                    println!("  Status: {:?}", info.status);
                }
                Err(e) => {
                    println!("Failed to parse {}: {}", file_name, e);
                }
            }
        }
    }

    #[test]
    fn test_amount_extraction() {
        let parser = PdfReceiptParser::new();
        
        let test_cases = vec![
            ("Сумма: 1000.50 руб", "1000.50"),
            ("Amount: 2 500,00 RUB", "2500.00"),
            ("Итого: 10 000 ₽", "10000"),
            ("TOTAL: 999.99", "999.99"),
        ];
        
        for (text, expected) in test_cases {
            let amount = parser.extract_amount(text).unwrap();
            assert_eq!(amount.to_string(), expected);
        }
    }

    #[test]
    fn test_datetime_extraction() {
        let parser = PdfReceiptParser::new();
        
        let test_cases = vec![
            "27.05.2025 14:30:00",
            "27/05/2025 14:30",
            "2025-05-27 14:30:00",
        ];
        
        for text in test_cases {
            let datetime = parser.extract_datetime(text);
            assert!(datetime.is_some());
        }
    }

    #[test]
    fn test_bank_name_extraction() {
        let parser = PdfReceiptParser::new();
        
        let test_cases = vec![
            ("Сбербанк России", Some("Сбербанк")),
            ("Tinkoff Bank", Some("Тинькофф")),
            ("Т-Банк перевод", Some("Т-Банк")),
            ("Альфа-Банк платеж", Some("Альфа-Банк")),
        ];
        
        for (text, expected) in test_cases {
            let bank = parser.extract_bank_name(text);
            assert_eq!(bank.as_deref(), expected);
        }
    }
}