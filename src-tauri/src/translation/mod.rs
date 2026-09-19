mod google;

use serde::Serialize;

pub use google::GoogleTranslationProvider;

pub trait TranslationProvider {
    fn name(&self) -> &'static str;
    fn translate_read(&self, text: &str) -> Result<String, TranslationError>;
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslationStarted {
    pub original_text: String,
    pub provider: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslationResult {
    pub success: bool,
    pub original_text: String,
    pub translated_text: Option<String>,
    pub provider: &'static str,
    pub error_code: Option<&'static str>,
    pub message: Option<String>,
}

impl TranslationResult {
    pub fn success(original_text: String, translated_text: String, provider: &'static str) -> Self {
        Self {
            success: true,
            original_text,
            translated_text: Some(translated_text),
            provider,
            error_code: None,
            message: None,
        }
    }

    pub fn error(original_text: String, provider: &'static str, error: TranslationError) -> Self {
        Self {
            success: false,
            original_text,
            translated_text: None,
            provider,
            error_code: Some(error.code()),
            message: Some(error.user_message()),
        }
    }
}

#[derive(Debug)]
pub enum TranslationError {
    MissingApiKey,
    Network,
    Api(String),
    InvalidResponse,
}

impl TranslationError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::MissingApiKey => "missing_api_key",
            Self::Network => "network_error",
            Self::Api(_) => "provider_error",
            Self::InvalidResponse => "invalid_response",
        }
    }

    pub fn user_message(&self) -> String {
        match self {
            Self::MissingApiKey => {
                "Please set the GOOGLE_TRANSLATE_API_KEY environment variable".into()
            }
            Self::Network => "Unable to reach Google Translate".into(),
            Self::Api(message) if !message.is_empty() => message.clone(),
            Self::Api(_) => "Google Translate rejected the request".into(),
            Self::InvalidResponse => "Google Translate returned an invalid response".into(),
        }
    }
}
