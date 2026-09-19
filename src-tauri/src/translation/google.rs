use std::{env, time::Duration};

use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use super::{TranslationError, TranslationProvider};

const GOOGLE_TRANSLATE_URL: &str = "https://translation.googleapis.com/language/translate/v2";
const API_KEY_ENV: &str = "GOOGLE_TRANSLATE_API_KEY";

pub struct GoogleTranslationProvider {
    api_key: String,
    client: Client,
}

impl GoogleTranslationProvider {
    pub fn from_environment() -> Result<Self, TranslationError> {
        let api_key = env::var(API_KEY_ENV)
            .ok()
            .filter(|key| !key.trim().is_empty())
            .ok_or(TranslationError::MissingApiKey)?;
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(20))
            .build()
            .map_err(|_| TranslationError::Network)?;

        Ok(Self { api_key, client })
    }
}

impl TranslationProvider for GoogleTranslationProvider {
    fn name(&self) -> &'static str {
        "Google Translate"
    }

    fn translate_read(&self, text: &str) -> Result<String, TranslationError> {
        let response = self
            .client
            .post(GOOGLE_TRANSLATE_URL)
            .query(&[("key", self.api_key.as_str())])
            .json(&TranslateRequest {
                q: text,
                target: "zh-CN",
                format: "text",
            })
            .send()
            .map_err(|_| TranslationError::Network)?;

        if !response.status().is_success() {
            let message = response
                .json::<GoogleErrorResponse>()
                .ok()
                .map(|body| body.error.message)
                .unwrap_or_default();
            return Err(TranslationError::Api(message));
        }

        let body = response
            .json::<TranslateResponse>()
            .map_err(|_| TranslationError::InvalidResponse)?;
        let translated = body
            .data
            .translations
            .into_iter()
            .next()
            .map(|item| html_escape::decode_html_entities(&item.translated_text).into_owned())
            .filter(|text| !text.trim().is_empty())
            .ok_or(TranslationError::InvalidResponse)?;

        Ok(translated)
    }
}

#[derive(Serialize)]
struct TranslateRequest<'a> {
    q: &'a str,
    target: &'static str,
    format: &'static str,
}

#[derive(Deserialize)]
struct TranslateResponse {
    data: TranslateData,
}

#[derive(Deserialize)]
struct TranslateData {
    translations: Vec<TranslatedItem>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TranslatedItem {
    translated_text: String,
}

#[derive(Deserialize)]
struct GoogleErrorResponse {
    error: GoogleError,
}

#[derive(Deserialize)]
struct GoogleError {
    message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_google_translate_response() {
        let response: TranslateResponse = serde_json::from_str(
            r#"{"data":{"translations":[{"translatedText":"你好 &amp; 欢迎"}]}}"#,
        )
        .expect("valid response");

        assert_eq!(
            response.data.translations[0].translated_text,
            "你好 &amp; 欢迎"
        );
    }
}
