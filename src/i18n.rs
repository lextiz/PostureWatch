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

    #[cfg_attr(not(windows), allow(dead_code))]
    pub fn from_config() -> Self {
        Self::from_code(&Config::load().language)
    }
}

#[cfg_attr(not(test), allow(dead_code))]
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
    DialogErrorTitle,
    DialogSavedTitle,
    DialogSettingsSaved,
    ApiKeyLabel,
    ModelLabel,
    PostureThresholdLabel,
    PostureRangeHint,
    AlertsAfterLabel,
    CheckIntervalLabel,
    CheckIntervalHint,
    KeepCameraOnLabel,
    CameraIndexLabel,
    CameraIndexHint,
    StandReminderLabel,
    StandReminderHint,
    BreakReminderLabel,
    SessionMaxHint,
    DayMaxHint,
    NotifyEveryHint,
    AdvancedPromptLabel,
    ValidationPostureThreshold,
    ValidationAlertThreshold,
    ValidationInterval,
    ValidationCameraIndex,
    ValidationStandReminder,
    ValidationBreakAfter,
    ValidationDailyMax,
    ValidationBreakRepeat,
    ValidationModelEmpty,
    ValidationPromptEmpty,
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
    TooltipApiCredentials,
    TooltipModel,
    TooltipPostureThreshold,
    TooltipAlertThreshold,
    TooltipInterval,
    TooltipCameraIndex,
    TooltipKeepCameraOn,
    TooltipStandReminderEnabled,
    TooltipStandReminderInterval,
    TooltipBreakReminderEnabled,
    TooltipSessionLimit,
    TooltipDailyLimit,
    TooltipBreakRepeat,
    TooltipLanguage,
    TooltipPrompt,
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
        (Language::En, Key::LanguageHint) => "en, ru",
        (Language::Ru, Key::LanguageHint) => "en, ru",
        (Language::En, Key::Save) => "Save",
        (Language::Ru, Key::Save) => "Сохранить",
        (Language::En, Key::Cancel) => "Cancel",
        (Language::Ru, Key::Cancel) => "Отмена",

        (Language::En, Key::LanguageValidationError) => "Language must be 'en' or 'ru'",
        (Language::Ru, Key::LanguageValidationError) => "Язык должен быть 'en' или 'ru'",
        (Language::En, Key::DialogErrorTitle) => "Error",
        (Language::Ru, Key::DialogErrorTitle) => "Ошибка",
        (Language::En, Key::DialogSavedTitle) => "Saved",
        (Language::Ru, Key::DialogSavedTitle) => "Сохранено",
        (Language::En, Key::DialogSettingsSaved) => "Settings saved.",
        (Language::Ru, Key::DialogSettingsSaved) => "Настройки сохранены.",
        (Language::En, Key::ApiKeyLabel) => "API Key:",
        (Language::Ru, Key::ApiKeyLabel) => "API-ключ:",
        (Language::En, Key::ModelLabel) => "Model:",
        (Language::Ru, Key::ModelLabel) => "Модель:",
        (Language::En, Key::PostureThresholdLabel) => "Posture threshold:",
        (Language::Ru, Key::PostureThresholdLabel) => "Порог осанки:",
        (Language::En, Key::PostureRangeHint) => "(1-10)",
        (Language::Ru, Key::PostureRangeHint) => "(1-10)",
        (Language::En, Key::AlertsAfterLabel) => "Alerts after:",
        (Language::Ru, Key::AlertsAfterLabel) => "Сигнал после:",
        (Language::En, Key::CheckIntervalLabel) => "Check interval:",
        (Language::Ru, Key::CheckIntervalLabel) => "Интервал проверки:",
        (Language::En, Key::CheckIntervalHint) => "seconds (5-300)",
        (Language::Ru, Key::CheckIntervalHint) => "секунд (5-300)",
        (Language::En, Key::KeepCameraOnLabel) => "Keep camera on between checks",
        (Language::Ru, Key::KeepCameraOnLabel) => "Не выключать камеру между проверками",
        (Language::En, Key::CameraIndexLabel) => "Camera index:",
        (Language::Ru, Key::CameraIndexLabel) => "Индекс камеры:",
        (Language::En, Key::CameraIndexHint) => "blank = auto",
        (Language::Ru, Key::CameraIndexHint) => "пусто = авто",
        (Language::En, Key::StandReminderLabel) => "Stand reminder",
        (Language::Ru, Key::StandReminderLabel) => "Напоминание встать",
        (Language::En, Key::StandReminderHint) => "minutes (1-480)",
        (Language::Ru, Key::StandReminderHint) => "минут (1-480)",
        (Language::En, Key::BreakReminderLabel) => "Break reminder",
        (Language::Ru, Key::BreakReminderLabel) => "Напоминание о перерыве",
        (Language::En, Key::SessionMaxHint) => "session max mins (1-480)",
        (Language::Ru, Key::SessionMaxHint) => "макс. сессия мин (1-480)",
        (Language::En, Key::DayMaxHint) => "day max mins (30-1440)",
        (Language::Ru, Key::DayMaxHint) => "макс. день мин (30-1440)",
        (Language::En, Key::NotifyEveryHint) => "notify every secs (5-600)",
        (Language::Ru, Key::NotifyEveryHint) => "повтор каждые сек (5-600)",
        (Language::En, Key::AdvancedPromptLabel) => "Advanced: LLM prompt",
        (Language::Ru, Key::AdvancedPromptLabel) => "Дополнительно: промпт LLM",
        (Language::En, Key::ValidationPostureThreshold) => "Posture threshold must be 1-10",
        (Language::Ru, Key::ValidationPostureThreshold) => "Порог осанки должен быть 1-10",
        (Language::En, Key::ValidationAlertThreshold) => "Alert threshold must be 1-10",
        (Language::Ru, Key::ValidationAlertThreshold) => "Порог сигнала должен быть 1-10",
        (Language::En, Key::ValidationInterval) => "Interval must be 5-300",
        (Language::Ru, Key::ValidationInterval) => "Интервал должен быть 5-300",
        (Language::En, Key::ValidationCameraIndex) => {
            "Camera index must be blank or a non-negative integer"
        }
        (Language::Ru, Key::ValidationCameraIndex) => {
            "Индекс камеры должен быть пустым или неотрицательным числом"
        }
        (Language::En, Key::ValidationStandReminder) => "Stand reminder must be 1-480 minutes",
        (Language::Ru, Key::ValidationStandReminder) => "Напоминание встать: 1-480 минут",
        (Language::En, Key::ValidationBreakAfter) => "Break reminder after must be 1-480 minutes",
        (Language::Ru, Key::ValidationBreakAfter) => "Напоминание о перерыве: 1-480 минут",
        (Language::En, Key::ValidationDailyMax) => "Daily screen time max must be 30-1440 minutes",
        (Language::Ru, Key::ValidationDailyMax) => "Макс. дневное время: 30-1440 минут",
        (Language::En, Key::ValidationBreakRepeat) => "Break repeat interval must be 5-600 seconds",
        (Language::Ru, Key::ValidationBreakRepeat) => "Интервал повтора перерыва: 5-600 секунд",
        (Language::En, Key::ValidationModelEmpty) => "Model cannot be empty",
        (Language::Ru, Key::ValidationModelEmpty) => "Модель не может быть пустой",
        (Language::En, Key::ValidationPromptEmpty) => "LLM prompt cannot be empty",
        (Language::Ru, Key::ValidationPromptEmpty) => "Промпт LLM не может быть пустым",
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
        (Language::En, Key::TooltipApiCredentials) => "API key for your LLM provider.",
        (Language::Ru, Key::TooltipApiCredentials) => "API-ключ для вашего провайдера LLM.",
        (Language::En, Key::TooltipModel) => "Vision model name for posture analysis.",
        (Language::Ru, Key::TooltipModel) => "Имя модели зрения для анализа осанки.",
        (Language::En, Key::TooltipPostureThreshold) => {
            "Minimum posture score (1-10) treated as good."
        }
        (Language::Ru, Key::TooltipPostureThreshold) => {
            "Минимальная оценка осанки (1-10), считающаяся хорошей."
        }
        (Language::En, Key::TooltipAlertThreshold) => {
            "How many low scores in a row trigger an alert."
        }
        (Language::Ru, Key::TooltipAlertThreshold) => {
            "Сколько низких оценок подряд нужно для сигнала."
        }
        (Language::En, Key::TooltipInterval) => "Seconds between posture checks.",
        (Language::Ru, Key::TooltipInterval) => "Секунды между проверками осанки.",
        (Language::En, Key::TooltipCameraIndex) => {
            "Camera index. Empty value = automatic selection."
        }
        (Language::Ru, Key::TooltipCameraIndex) => "Индекс камеры. Пусто = авто-выбор.",
        (Language::En, Key::TooltipKeepCameraOn) => {
            "Keep camera open between checks for faster capture."
        }
        (Language::Ru, Key::TooltipKeepCameraOn) => {
            "Держать камеру открытой между проверками для более быстрого захвата."
        }
        (Language::En, Key::TooltipStandReminderEnabled) => "Enable stand-up reminders.",
        (Language::Ru, Key::TooltipStandReminderEnabled) => "Включить напоминания встать.",
        (Language::En, Key::TooltipStandReminderInterval) => "Minutes between stand-up reminders.",
        (Language::Ru, Key::TooltipStandReminderInterval) => "Минуты между напоминаниями встать.",
        (Language::En, Key::TooltipBreakReminderEnabled) => "Enable break reminders.",
        (Language::Ru, Key::TooltipBreakReminderEnabled) => "Включить напоминания о перерыве.",
        (Language::En, Key::TooltipSessionLimit) => {
            "Session limit (minutes) before break reminders start."
        }
        (Language::Ru, Key::TooltipSessionLimit) => {
            "Лимит сессии (мин) до начала напоминаний о перерыве."
        }
        (Language::En, Key::TooltipDailyLimit) => {
            "Daily limit (minutes) before break reminders start."
        }
        (Language::Ru, Key::TooltipDailyLimit) => {
            "Дневной лимит (мин) до начала напоминаний о перерыве."
        }
        (Language::En, Key::TooltipBreakRepeat) => "Seconds between repeated break reminders.",
        (Language::Ru, Key::TooltipBreakRepeat) => {
            "Секунды между повторными напоминаниями о перерыве."
        }
        (Language::En, Key::TooltipLanguage) => "Interface language.",
        (Language::Ru, Key::TooltipLanguage) => "Язык интерфейса.",
        (Language::En, Key::TooltipPrompt) => "Advanced prompt sent with each image.",
        (Language::Ru, Key::TooltipPrompt) => {
            "Дополнительный промпт, отправляемый с каждым изображением."
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{text, Key, Language};

    const ALL_KEYS: &[Key] = &[
        Key::Pause,
        Key::Resume,
        Key::Configure,
        Key::About,
        Key::Exit,
        Key::TrayToday,
        Key::TraySession,
        Key::TrayScore,
        Key::TrayPaused,
        Key::SettingsTitle,
        Key::LanguageLabel,
        Key::LanguageHint,
        Key::Save,
        Key::Cancel,
        Key::LanguageValidationError,
        Key::DialogErrorTitle,
        Key::DialogSavedTitle,
        Key::DialogSettingsSaved,
        Key::ApiKeyLabel,
        Key::ModelLabel,
        Key::PostureThresholdLabel,
        Key::PostureRangeHint,
        Key::AlertsAfterLabel,
        Key::CheckIntervalLabel,
        Key::CheckIntervalHint,
        Key::KeepCameraOnLabel,
        Key::CameraIndexLabel,
        Key::CameraIndexHint,
        Key::StandReminderLabel,
        Key::StandReminderHint,
        Key::BreakReminderLabel,
        Key::SessionMaxHint,
        Key::DayMaxHint,
        Key::NotifyEveryHint,
        Key::AdvancedPromptLabel,
        Key::ValidationPostureThreshold,
        Key::ValidationAlertThreshold,
        Key::ValidationInterval,
        Key::ValidationCameraIndex,
        Key::ValidationStandReminder,
        Key::ValidationBreakAfter,
        Key::ValidationDailyMax,
        Key::ValidationBreakRepeat,
        Key::ValidationModelEmpty,
        Key::ValidationPromptEmpty,
        Key::AboutTitle,
        Key::AboutBody,
        Key::NotificationApp,
        Key::BadPosture,
        Key::StandTitle,
        Key::StandBody,
        Key::BreakTitle,
        Key::BreakBody,
        Key::SessionLimitTitle,
        Key::SessionLimitBody,
        Key::DailyLimitTitle,
        Key::DailyLimitBody,
        Key::ApiSetupTitle,
        Key::ApiSetupBody,
        Key::ApiSetupSummary,
        Key::ApiSetupDetails,
        Key::TooltipApiCredentials,
        Key::TooltipModel,
        Key::TooltipPostureThreshold,
        Key::TooltipAlertThreshold,
        Key::TooltipInterval,
        Key::TooltipCameraIndex,
        Key::TooltipKeepCameraOn,
        Key::TooltipStandReminderEnabled,
        Key::TooltipStandReminderInterval,
        Key::TooltipBreakReminderEnabled,
        Key::TooltipSessionLimit,
        Key::TooltipDailyLimit,
        Key::TooltipBreakRepeat,
        Key::TooltipLanguage,
        Key::TooltipPrompt,
    ];

    #[test]
    fn language_from_code_defaults_to_english() {
        assert_eq!(Language::from_code("ru"), Language::Ru);
        assert_eq!(Language::from_code("RU"), Language::Ru);
        assert_eq!(Language::from_code("en"), Language::En);
        assert_eq!(Language::from_code("anything-else"), Language::En);
    }

    #[test]
    fn all_translation_keys_have_non_empty_text_for_all_languages() {
        for key in ALL_KEYS {
            assert!(!text(Language::En, *key).trim().is_empty());
            assert!(!text(Language::Ru, *key).trim().is_empty());
        }
    }
}
