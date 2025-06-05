use async_openai::{Client, config::OpenAIConfig, types::{CreateChatCompletionRequestArgs, ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs, ChatCompletionRequestAssistantMessageArgs}};
use crate::core::config::AIConfig;
use crate::utils::error::{Result, AppError};
use serde::{Deserialize, Serialize};
use tracing::{info, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueState {
    pub stage: DialogueStage,
    pub responses: DialogueResponses,
    pub completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DialogueStage {
    Initial,
    AwaitingTBankConfirmation,
    AwaitingPdfConfirmation,
    AwaitingSbpWarningConfirmation,
    SendingPaymentDetails,
    AwaitingReceipt,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DialogueResponses {
    pub tbank_confirmed: Option<bool>,
    pub pdf_confirmed: Option<bool>,
    pub sbp_warning_confirmed: Option<bool>,
}

pub struct ChatManager {
    config: AIConfig,
    client: Option<Client<OpenAIConfig>>,
}

impl ChatManager {
    pub fn new(config: AIConfig) -> Self {
        let client = if !config.openrouter_api_key.is_empty() {
            let openai_config = OpenAIConfig::new()
                .with_api_key(&config.openrouter_api_key)
                .with_api_base("https://openrouter.ai/api/v1");
            Some(Client::with_config(openai_config))
        } else {
            None
        };
        
        Self { config, client }
    }
    
    pub fn get_initial_message() -> String {
        "Здравствуйте!\n\n\
        1. Оплата будет с Т банка?\n\
        ( просто напишите да/нет)\n\n\
        2. Чек в формате пдф с официальной почты Т банка сможете отправить ?\n\
        ( просто напишите да/нет)\n\n\
        3. При СБП, если оплата будет на неверный банк, деньги потеряны.\n\
        ( просто напишите подтверждаю/ не подтверждаю)\n\n\
        После чего одним смс я пришлю реквизиты, банк, почту и сумму для перевода.".to_string()
    }
    
    pub async fn process_message(
        &self,
        dialogue_state: &DialogueState,
        user_message: &str,
    ) -> Result<(String, DialogueState)> {
        let mut new_state = dialogue_state.clone();
        
        match dialogue_state.stage {
            DialogueStage::Initial => {
                // Initial message already sent, should not reach here
                Ok((Self::get_initial_message(), new_state))
            }
            DialogueStage::AwaitingTBankConfirmation => {
                let (is_yes, response) = self.parse_yes_no_response(user_message).await?;
                new_state.responses.tbank_confirmed = Some(is_yes);
                
                if !is_yes {
                    new_state.stage = DialogueStage::Failed;
                    Ok(("К сожалению, мы работаем только с Т банком. Сделка отменена.".to_string(), new_state))
                } else {
                    new_state.stage = DialogueStage::AwaitingPdfConfirmation;
                    Ok((response, new_state))
                }
            }
            DialogueStage::AwaitingPdfConfirmation => {
                let (is_yes, response) = self.parse_yes_no_response(user_message).await?;
                new_state.responses.pdf_confirmed = Some(is_yes);
                
                if !is_yes {
                    new_state.stage = DialogueStage::Failed;
                    Ok(("Чек в формате PDF обязателен для подтверждения платежа. Сделка отменена.".to_string(), new_state))
                } else {
                    new_state.stage = DialogueStage::AwaitingSbpWarningConfirmation;
                    Ok((response, new_state))
                }
            }
            DialogueStage::AwaitingSbpWarningConfirmation => {
                let (is_confirmed, response) = self.parse_confirmation_response(user_message).await?;
                new_state.responses.sbp_warning_confirmed = Some(is_confirmed);
                
                if !is_confirmed {
                    new_state.stage = DialogueStage::Failed;
                    Ok(("Необходимо подтвердить понимание условий СБП. Сделка отменена.".to_string(), new_state))
                } else {
                    new_state.stage = DialogueStage::SendingPaymentDetails;
                    Ok((response, new_state))
                }
            }
            DialogueStage::SendingPaymentDetails => {
                // This stage is handled by the system sending payment details
                new_state.stage = DialogueStage::AwaitingReceipt;
                Ok(("Ожидаем получение чека на указанную почту.".to_string(), new_state))
            }
            DialogueStage::AwaitingReceipt => {
                // Keep waiting for receipt
                Ok(("Пожалуйста, отправьте чек в формате PDF на указанную почту.".to_string(), new_state))
            }
            DialogueStage::Completed | DialogueStage::Failed => {
                Ok(("Сделка завершена.".to_string(), new_state))
            }
        }
    }
    
    pub fn get_payment_details_message(
        &self,
        phone: &str,
        bank: &str,
        email: &str,
        amount: f64,
    ) -> String {
        format!(
            "✅ Все условия приняты. Вот реквизиты для перевода:\n\n\
            💰 Сумма: {} RUB\n\
            🏦 Банк: {}\n\
            📱 Номер телефона: {}\n\
            📧 Email для чека: {}\n\n\
            После перевода обязательно отправьте PDF чек с официальной почты Т банка на указанный email.",
            amount, bank, phone, email
        )
    }
    
    async fn parse_yes_no_response(&self, message: &str) -> Result<(bool, String)> {
        let normalized = message.trim().to_lowercase();
        
        // Simple parsing first
        if normalized.contains("да") || normalized.contains("yes") {
            return Ok((true, "Принято. Переходим к следующему вопросу.".to_string()));
        }
        if normalized.contains("нет") || normalized.contains("no") {
            return Ok((false, "Понятно.".to_string()));
        }
        
        // If we have AI client, use it for complex parsing
        if let Some(client) = &self.client {
            match self.ai_parse_yes_no(client, message).await {
                Ok(result) => Ok(result),
                Err(e) => {
                    error!("AI parsing failed: {}", e);
                    // Fallback to asking again
                    Ok((false, "Пожалуйста, ответьте 'да' или 'нет'.".to_string()))
                }
            }
        } else {
            Ok((false, "Пожалуйста, ответьте 'да' или 'нет'.".to_string()))
        }
    }
    
    async fn parse_confirmation_response(&self, message: &str) -> Result<(bool, String)> {
        let normalized = message.trim().to_lowercase();
        
        if normalized.contains("подтверждаю") || normalized.contains("confirm") {
            return Ok((true, "Отлично! Сейчас отправлю реквизиты для перевода.".to_string()));
        }
        if normalized.contains("не подтверждаю") || normalized.contains("not confirm") {
            return Ok((false, "Понятно.".to_string()));
        }
        
        // If we have AI client, use it
        if let Some(client) = &self.client {
            match self.ai_parse_confirmation(client, message).await {
                Ok(result) => Ok(result),
                Err(e) => {
                    error!("AI parsing failed: {}", e);
                    Ok((false, "Пожалуйста, напишите 'подтверждаю' или 'не подтверждаю'.".to_string()))
                }
            }
        } else {
            Ok((false, "Пожалуйста, напишите 'подтверждаю' или 'не подтверждаю'.".to_string()))
        }
    }
    
    async fn ai_parse_yes_no(&self, client: &Client<OpenAIConfig>, message: &str) -> Result<(bool, String)> {
        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.config.model)
            .messages([
                ChatCompletionRequestSystemMessageArgs::default()
                    .content("You are parsing user responses to yes/no questions in Russian. \
                             Respond with JSON: {\"is_yes\": true/false, \"response\": \"appropriate Russian response\"}")
                    .build()?
                    .into(),
                ChatCompletionRequestUserMessageArgs::default()
                    .content(format!("Parse this response: {}", message))
                    .build()?
                    .into(),
            ])
            .temperature(0.1)
            .build()?;
            
        let response = client.chat().create(request).await?;
        let content = response.choices.first()
            .ok_or_else(|| AppError::InternalError("No AI response".to_string()))?
            .message.content.as_ref()
            .ok_or_else(|| AppError::InternalError("Empty AI response".to_string()))?;
            
        let parsed: serde_json::Value = serde_json::from_str(content)
            .map_err(|e| AppError::InternalError(format!("Failed to parse AI response: {}", e)))?;
            
        Ok((
            parsed["is_yes"].as_bool().unwrap_or(false),
            parsed["response"].as_str().unwrap_or("Обработано.").to_string()
        ))
    }
    
    async fn ai_parse_confirmation(&self, client: &Client<OpenAIConfig>, message: &str) -> Result<(bool, String)> {
        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.config.model)
            .messages([
                ChatCompletionRequestSystemMessageArgs::default()
                    .content("You are parsing user responses to confirmation requests in Russian. \
                             User should confirm they understand SBP risks. \
                             Respond with JSON: {\"is_confirmed\": true/false, \"response\": \"appropriate Russian response\"}")
                    .build()?
                    .into(),
                ChatCompletionRequestUserMessageArgs::default()
                    .content(format!("Parse this response: {}", message))
                    .build()?
                    .into(),
            ])
            .temperature(0.1)
            .build()?;
            
        let response = client.chat().create(request).await?;
        let content = response.choices.first()
            .ok_or_else(|| AppError::InternalError("No AI response".to_string()))?
            .message.content.as_ref()
            .ok_or_else(|| AppError::InternalError("Empty AI response".to_string()))?;
            
        let parsed: serde_json::Value = serde_json::from_str(content)
            .map_err(|e| AppError::InternalError(format!("Failed to parse AI response: {}", e)))?;
            
        Ok((
            parsed["is_confirmed"].as_bool().unwrap_or(false),
            parsed["response"].as_str().unwrap_or("Обработано.").to_string()
        ))
    }
}