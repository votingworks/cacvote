use wasm_bindgen::JsValue;

#[derive(Debug, Clone, PartialEq)]
pub enum DateOrDateTime {
    Date(time::Date),
    DateTime(time::OffsetDateTime),
}

impl DateOrDateTime {
    pub fn to_iso8601(&self) -> String {
        match self {
            Self::Date(date) => date.to_string(),
            Self::DateTime(date_time) => date_time.to_string(),
        }
    }
}

impl From<time::Date> for DateOrDateTime {
    fn from(date: time::Date) -> Self {
        Self::Date(date)
    }
}

impl From<time::OffsetDateTime> for DateOrDateTime {
    fn from(date_time: time::OffsetDateTime) -> Self {
        Self::DateTime(date_time)
    }
}

impl From<&time::Date> for DateOrDateTime {
    fn from(date: &time::Date) -> Self {
        Self::Date(*date)
    }
}

impl From<&time::OffsetDateTime> for DateOrDateTime {
    fn from(date_time: &time::OffsetDateTime) -> Self {
        Self::DateTime(*date_time)
    }
}

#[derive(Debug, Clone, Copy, Default)]
#[allow(dead_code)]
pub enum DateStyle {
    #[default]
    Default,
    Full,
    Long,
    Medium,
    Short,
}

impl DateStyle {
    fn to_js_value(self) -> Option<JsValue> {
        match self {
            Self::Default => None,
            Self::Full => Some(JsValue::from_str("full")),
            Self::Long => Some(JsValue::from_str("long")),
            Self::Medium => Some(JsValue::from_str("medium")),
            Self::Short => Some(JsValue::from_str("short")),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
#[allow(dead_code)]
pub enum TimeStyle {
    #[default]
    Default,
    Full,
    Long,
    Medium,
    Short,
}

impl TimeStyle {
    pub fn to_js_value(self) -> Option<JsValue> {
        match self {
            Self::Default => None,
            Self::Full => Some(JsValue::from_str("full")),
            Self::Long => Some(JsValue::from_str("long")),
            Self::Medium => Some(JsValue::from_str("medium")),
            Self::Short => Some(JsValue::from_str("short")),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DateFormatOptions {
    date_style: DateStyle,
    time_style: TimeStyle,
}

#[allow(dead_code)]
impl DateFormatOptions {
    pub const DATETIME: Self = Self {
        date_style: DateStyle::Medium,
        time_style: TimeStyle::Short,
    };
    pub const DATE: Self = Self {
        date_style: DateStyle::Medium,
        time_style: TimeStyle::Default,
    };
    pub const DATE_WITH_DAY_OF_WEEK: Self = Self {
        date_style: DateStyle::Full,
        time_style: TimeStyle::Default,
    };
}

pub fn format(date_or_datetime: &DateOrDateTime, options: &DateFormatOptions) -> String {
    let locales = js_sys::Array::new();
    locales.push(&JsValue::from_str("default"));

    let js_options = js_sys::Object::new();

    if let Some(date_style) = options.date_style.to_js_value() {
        let _ = js_sys::Reflect::set(&js_options, &JsValue::from_str("dateStyle"), &date_style);
    }

    if let Some(time_style) = options.time_style.to_js_value() {
        let _ = js_sys::Reflect::set(&js_options, &JsValue::from_str("timeStyle"), &time_style);
    }

    let formatter = js_sys::Intl::DateTimeFormat::new(&locales, &js_options);
    let js_date = js_sys::Date::new_0();

    match date_or_datetime {
        DateOrDateTime::Date(date) => {
            js_date.set_utc_full_year(date.year() as u32);
            js_date.set_utc_month(date.month() as u32 - 1);
            js_date.set_utc_date(date.day() as u32);
        }
        DateOrDateTime::DateTime(datetime) => {
            js_date.set_utc_full_year(datetime.year() as u32);
            js_date.set_utc_month(datetime.month() as u32 - 1);
            js_date.set_utc_date(datetime.day() as u32);
            js_date.set_utc_hours(datetime.hour() as u32);
            js_date.set_utc_minutes(datetime.minute() as u32);
            js_date.set_utc_seconds(datetime.second() as u32);
        }
    }

    let formatted_date = formatter.format().call1(&formatter, &js_date).unwrap();
    formatted_date.as_string().unwrap()
}
