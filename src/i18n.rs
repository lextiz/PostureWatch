use crate::config::Config;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Language {
    En,
    Ru,
}

impl Language {
    pub fn from_code(code: &str) -> Self {
        if code.trim().eq_ignore_ascii_case("ru") {
            Self::Ru
        } else {
            Self::En
        }
    }

    pub fn from_config() -> Self {
        Self::from_code(&Config::load().language)
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Key {
    Pause,
    Resume,
    Configure,
    About,
    Exit,
    TrayToday,
    TraySession,
    TrayScore,
    TrayPaused,
    SettingsTitle,
    LanguageLabel,
    LanguageHint,
    Save,
    Cancel,
    LanguageValidationError,
    AboutTitle,
    AboutBody,
    NotificationApp,
    BadPosture,
    StandTitle,
    StandBody,
    BreakTitle,
    BreakBody,
    SessionLimitTitle,
    SessionLimitBody,
    DailyLimitTitle,
    DailyLimitBody,
    ApiSetupTitle,
    ApiSetupBody,
    ApiSetupSummary,
    ApiSetupDetails,
}

pub fn text(language: Language, key: Key) -> &'static str {
    match (language, key) {
        (Language::En, Key::Pause) => "Pause",
        (Language::Ru, Key::Pause) => "Пауза",
        (Language::En, Key::Resume) => "Resume",
        (Language::Ru, Key::Resume) => "Продолжить",
        (Language::En, Key::Configure) => "Configure...",
        (Language::Ru, Key::Configure) => "Настройки...",
        (Language::En, Key::About) => "About",
        (Language::Ru, Key::About) => "О программе",
        (Language::En, Key::Exit) => "Exit",
        (Language::Ru, Key::Exit) => "Выход",
        (Language::En, Key::TrayToday) => "Today",
        (Language::Ru, Key::TrayToday) => "Сегодня",
        (Language::En, Key::TraySession) => "Session",
        (Language::Ru, Key::TraySession) => "Сессия",
        (Language::En, Key::TrayScore) => "Score",
        (Language::Ru, Key::TrayScore) => "Оценка",
        (Language::En, Key::TrayPaused) => "PostureWatch (Paused)",
        (Language::Ru, Key::TrayPaused) => "PostureWatch (Пауза)",
        (Language::En, Key::SettingsTitle) => "PostureWatch Settings",
        (Language::Ru, Key::SettingsTitle) => "Настройки PostureWatch",
        (Language::En, Key::LanguageLabel) => "Language:",
        (Language::Ru, Key::LanguageLabel) => "Язык интерфейса:",
        (Language::En, Key::LanguageHint) => "supported: en, ru",
        (Language::Ru, Key::LanguageHint) => "доступно: en, ru",
        (Language::En, Key::Save) => "Save",
        (Language::Ru, Key::Save) => "Сохранить",
        (Language::En, Key::Cancel) => "Cancel",
        (Language::Ru, Key::Cancel) => "Отмена",
        (Language::En, Key::LanguageValidationError) => "Language must be 'en' or 'ru'",
        (Language::Ru, Key::LanguageValidationError) => "Язык должен быть 'en' или 'ru'",
        (Language::En, Key::AboutTitle) => "About PostureWatch",
        (Language::Ru, Key::AboutTitle) => "О PostureWatch",
        (Language::En, Key::AboutBody) => "PostureWatch v1.0\n\nAI-powered posture monitoring.",
        (Language::Ru, Key::AboutBody) => "PostureWatch v1.0\n\nМониторинг осанки с помощью ИИ.",
        (Language::En, Key::NotificationApp) => "Posture Watch",
        (Language::Ru, Key::NotificationApp) => "Posture Watch",
        (Language::En, Key::BadPosture) => "Please sit up straight!",
        (Language::Ru, Key::BadPosture) => "Пожалуйста, сядьте ровнее!",
        (Language::En, Key::StandTitle) => "Posture Watch - Stand up!",
        (Language::Ru, Key::StandTitle) => "Posture Watch - Встаньте!",
        (Language::En, Key::StandBody) => "Time to raise your desk or stretch your legs.",
        (Language::Ru, Key::StandBody) => "Пора поднять стол или немного размяться.",
        (Language::En, Key::BreakTitle) => "Posture Watch - Break reminder",
        (Language::Ru, Key::BreakTitle) => "Posture Watch - Напоминание о перерыве",
        (Language::En, Key::BreakBody) => {
            "You've been at your desk for a while. Please take a break."
        }
        (Language::Ru, Key::BreakBody) => "Вы давно за столом. Сделайте небольшой перерыв.",
        (Language::En, Key::SessionLimitTitle) => "Posture Watch - Session limit reached",
        (Language::Ru, Key::SessionLimitTitle) => "Posture Watch - Лимит сессии достигнут",
        (Language::En, Key::SessionLimitBody) => {
            "Screen time session limit reached. Please take a break."
        }
        (Language::Ru, Key::SessionLimitBody) => {
            "Лимит экранного времени за сессию достигнут. Сделайте перерыв."
        }
        (Language::En, Key::DailyLimitTitle) => "Posture Watch - Daily limit reached",
        (Language::Ru, Key::DailyLimitTitle) => "Posture Watch - Дневной лимит достигнут",
        (Language::En, Key::DailyLimitBody) => {
            "Daily screen time limit reached. Please rest your eyes."
        }
        (Language::Ru, Key::DailyLimitBody) => {
            "Дневной лимит экранного времени достигнут. Дайте глазам отдохнуть."
        }
        (Language::En, Key::ApiSetupTitle) => "Posture Watch setup required",
        (Language::Ru, Key::ApiSetupTitle) => "Требуется настройка Posture Watch",
        (Language::En, Key::ApiSetupBody) => "Your API key is missing or not working.",
        (Language::Ru, Key::ApiSetupBody) => "API-ключ отсутствует или не работает.",
        (Language::En, Key::ApiSetupSummary) => "Posture Watch setup required",
        (Language::Ru, Key::ApiSetupSummary) => "Требуется настройка Posture Watch",
        (Language::En, Key::ApiSetupDetails) => "Details",
        (Language::Ru, Key::ApiSetupDetails) => "Детали",
    }
}
