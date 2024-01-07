use relm4::gtk::{
    glib::{DateTime, TimeZone},
    InputHints, InputPurpose, Justification, Orientation, PositionType,
};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Map;
use serde_with::{serde_as, DefaultOnError};
use std::fmt;

#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Schema {
    #[serde(alias = "$schema")]
    pub schema: String,
    #[serde(alias = "$id")]
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub configurable: Option<String>,
    pub r#type: String,
    pub properties: Map<String, serde_json::Value>,
}

impl Schema {
    pub fn get_property(&self, key: &str) -> Option<SchemaProp> {
        let v = self.properties.get(key);
        if v.is_some() {
            let prop: SchemaProp = serde_json::from_value(v.unwrap().clone()).unwrap();
            return Some(prop);
        } else {
            return None;
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchemaProp {
    pub r#type: String,
    pub description: Option<String>,
    pub default: Option<Primitive>,
    pub alias: Option<String>,
    pub r#enum: Option<Vec<String>>,
    pub items: Option<SchemaPropItem>,
    pub format: Option<String>,
    #[serde(alias = "x-prompt")]
    pub x_prompt: Option<StringOrPrompt>,
    #[serde(alias = "x-widget")]
    pub x_widget: Option<XWidget>,
}

impl Default for SchemaProp {
    fn default() -> Self {
        SchemaProp {
            r#type: String::default(),
            description: None,
            default: None,
            alias: None,
            r#enum: None,
            items: None,
            format: None,
            x_prompt: None,
            x_widget: None,
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(transparent)]
pub struct XWidget {
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub options: XWidgetType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum XWidgetType {
    Text(TextEntry),
    Numeric(NumericEntry),
    Date(DateEntry),
    Color(ColorEntry),
    File(FsEntry),
    Dir(FsEntry),
    Choice(ChoiceEntry),
    Menu(MenuEntry),
}

impl Default for XWidgetType {
    fn default() -> Self {
        XWidgetType::Text(TextEntry::default())
    }
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FsEntry {
    #[serde(deserialize_with = "deserialize_mask")]
    pub mask: String,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub relative: Option<String>,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub is_new: bool,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub current_folder: Option<String>,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub multiple: bool,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub default_name: Option<String>,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub is_dir: bool,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub title: Option<String>,
}

fn deserialize_mask<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let deserialized = String::deserialize(deserializer);
    Ok(deserialized.unwrap_or(String::from("*")))
}

impl Default for FsEntry {
    fn default() -> Self {
        FsEntry {
            mask: String::from("*"),
            relative: None,
            is_new: false,
            current_folder: None,
            multiple: false,
            default_name: None,
            is_dir: false,
            title: Some(String::from("Choose a file")),
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DateEntry {
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub format: String,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub r#type: DateEntryType,
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeInput(pub i32, pub i32, pub i32);

impl Default for TimeInput {
    fn default() -> Self {
        TimeInput(0, 0, 0)
    }
}

impl Default for DateEntry {
    fn default() -> Self {
        DateEntry {
            format: String::default(),
            r#type: DateEntryType::Date,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DateEntryType {
    Date,
    Time,
    DateTime,
}

impl Default for DateEntryType {
    fn default() -> Self {
        DateEntryType::Date
    }
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ColorEntry {
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub r#type: ColorEntryType,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub format: ColorEntryFormat,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub alpha: bool,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub title: String,
}

impl Default for ColorEntry {
    fn default() -> Self {
        ColorEntry {
            r#type: ColorEntryType::Button,
            format: ColorEntryFormat::RGB,
            alpha: false,
            title: String::from("Choose a color"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColorEntryFormat {
    Hex,
    RGB,
    HSL,
}

impl Default for ColorEntryFormat {
    fn default() -> Self {
        ColorEntryFormat::RGB
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColorEntryType {
    Button,
    Input,
}

impl Default for ColorEntryType {
    fn default() -> Self {
        ColorEntryType::Input
    }
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ChoiceEntry {
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub r#type: ChoiceType,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub label: String,
}

impl Default for ChoiceEntry {
    fn default() -> Self {
        ChoiceEntry {
            r#type: ChoiceType::Checkbox,
            label: String::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ChoiceType {
    Checkbox,
    Switch,
    Toggle,
}

impl Default for ChoiceType {
    fn default() -> Self {
        ChoiceType::Checkbox
    }
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MenuEntry {
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub r#type: MenuType,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub searchable: bool,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub multichoice: bool,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub orientation: OrientationType,
}

impl Default for MenuEntry {
    fn default() -> Self {
        MenuEntry {
            r#type: MenuType::DropDown,
            multichoice: false,
            searchable: false,
            orientation: OrientationType::Vertical,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MenuType {
    DropDown,
    Combobox,
    Multiselect,
    Radio,
    Toggle,
}

impl Default for MenuType {
    fn default() -> Self {
        MenuType::DropDown
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OrientationType {
    Horizontal,
    Vertical,
}

impl Default for OrientationType {
    fn default() -> Self {
        OrientationType::Vertical
    }
}

impl Into<Orientation> for OrientationType {
    fn into(self) -> Orientation {
        match self {
            OrientationType::Horizontal => Orientation::Horizontal,
            OrientationType::Vertical => Orientation::Vertical,
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TextEntry {
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub max_len: i32,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub height: i32,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub placeholder: String,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub icon: String,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub icon_position: IconPositionType,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub tooltip: String,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub hint_text: String,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub multiline: bool,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub direction: TextAlignmentType,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub overwrite: bool,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub purpose: PurposeType,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub hint: HintType,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub justify: JustificationType,
}

impl Default for TextEntry {
    fn default() -> TextEntry {
        TextEntry {
            hint_text: String::default(),
            max_len: 0,
            height: 50,
            placeholder: String::default(),
            tooltip: String::default(),
            icon: String::default(),
            icon_position: IconPositionType::None,
            multiline: false,
            overwrite: false,
            direction: TextAlignmentType::Left,
            purpose: PurposeType::FreeForm,
            hint: HintType::None,
            justify: JustificationType::None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum IconPositionType {
    Start,
    None,
    End,
}

impl Default for IconPositionType {
    fn default() -> Self {
        IconPositionType::None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TextAlignmentType {
    Left,
    Right,
}

impl Into<f32> for TextAlignmentType {
    fn into(self) -> f32 {
        match self {
            TextAlignmentType::Left => 0.0,
            TextAlignmentType::Right => 1.0,
        }
    }
}

impl Default for TextAlignmentType {
    fn default() -> Self {
        TextAlignmentType::Left
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PurposeType {
    FreeForm,
    Alpha,
    Digits,
    Number,
    Phone,
    Url,
    Email,
    Name,
    Terminal,
}

impl Into<InputPurpose> for PurposeType {
    fn into(self) -> InputPurpose {
        match self {
            PurposeType::Digits => InputPurpose::Digits,
            PurposeType::Alpha => InputPurpose::Alpha,
            PurposeType::FreeForm => InputPurpose::FreeForm,
            PurposeType::Email => InputPurpose::Email,
            PurposeType::Number => InputPurpose::Number,
            PurposeType::Terminal => InputPurpose::Terminal,
            PurposeType::Url => InputPurpose::Url,
            PurposeType::Phone => InputPurpose::Phone,
            PurposeType::Name => InputPurpose::Name,
        }
    }
}

impl Default for PurposeType {
    fn default() -> Self {
        PurposeType::FreeForm
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HintType {
    Lowercase,
    UppercaseChars,
    UppercaseWords,
    UppercaseSentences,
    InhibitOsk,
    VerticalWriting,
    Private,
    Spellcheck,
    NoSpellcheck,
    WordCompletion,
    None,
}

impl Default for HintType {
    fn default() -> Self {
        HintType::None
    }
}

impl Into<InputHints> for HintType {
    fn into(self) -> InputHints {
        match self {
            HintType::Lowercase => InputHints::LOWERCASE,
            HintType::UppercaseChars => InputHints::UPPERCASE_CHARS,
            HintType::UppercaseWords => InputHints::UPPERCASE_WORDS,
            HintType::UppercaseSentences => InputHints::UPPERCASE_SENTENCES,
            HintType::InhibitOsk => InputHints::INHIBIT_OSK,
            HintType::Spellcheck => InputHints::SPELLCHECK,
            HintType::NoSpellcheck => InputHints::NO_SPELLCHECK,
            HintType::VerticalWriting => InputHints::VERTICAL_WRITING,
            HintType::WordCompletion => InputHints::WORD_COMPLETION,
            HintType::Private => InputHints::PRIVATE,
            HintType::None => InputHints::NONE,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum JustificationType {
    Left,
    Right,
    Center,
    Fill,
    None,
}

impl Default for JustificationType {
    fn default() -> Self {
        JustificationType::Fill
    }
}

impl Into<Justification> for JustificationType {
    fn into(self) -> Justification {
        match self {
            JustificationType::Center => Justification::Center,
            JustificationType::Left => Justification::Left,
            JustificationType::Right => Justification::Right,
            JustificationType::Fill => Justification::Fill,
            JustificationType::None => Justification::__Unknown(0),
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct NumericEntry {
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub r#type: NumericType,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub value_type: NumericValueType,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub stepping: IntOrFloat,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub max: IntOrFloat,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub min: IntOrFloat,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub initial_value: IntOrFloat,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub page_increment: IntOrFloat,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub page_size: IntOrFloat,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub orientation: OrientationType,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub precision: IntOrFloat,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub wrap: bool,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub show_current: CurrentValuePosType,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub marks: Vec<MarkData>,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub slider_height: i32,
}

impl Default for NumericEntry {
    fn default() -> Self {
        NumericEntry {
            r#type: NumericType::Input,
            value_type: NumericValueType::Float,
            stepping: IntOrFloat::Float(1.0),
            max: IntOrFloat::Float(f64::MAX),
            min: IntOrFloat::Float(0.0),
            initial_value: IntOrFloat::Float(0.0),
            page_increment: IntOrFloat::Float(5.0),
            page_size: IntOrFloat::Float(1.0),
            orientation: OrientationType::Horizontal,
            precision: IntOrFloat::Int(2),
            slider_height: 100,
            wrap: false,
            show_current: CurrentValuePosType::Top,
            marks: vec![],
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct MarkData {
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub value: f64,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CurrentValuePosType {
    Top,
    Bottom,
    Right,
    Left,
    None,
}

impl Default for CurrentValuePosType {
    fn default() -> Self {
        CurrentValuePosType::Top
    }
}

impl Into<PositionType> for CurrentValuePosType {
    fn into(self) -> PositionType {
        match self {
            CurrentValuePosType::Bottom => PositionType::Bottom,
            CurrentValuePosType::Top => PositionType::Top,
            CurrentValuePosType::Left => PositionType::Left,
            CurrentValuePosType::Right => PositionType::Right,
            CurrentValuePosType::None => PositionType::__Unknown(999),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NumericType {
    Input,
    Slider,
}

impl Default for NumericType {
    fn default() -> Self {
        NumericType::Input
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
pub enum IntOrFloat {
    Int(i32),
    Float(f64),
}

impl Default for IntOrFloat {
    fn default() -> Self {
        IntOrFloat::Float(f64::default())
    }
}

impl Into<f64> for IntOrFloat {
    fn into(self) -> f64 {
        match self {
            IntOrFloat::Float(f) => f,
            IntOrFloat::Int(i) => i as f64,
        }
    }
}

impl Into<i32> for IntOrFloat {
    fn into(self) -> i32 {
        match self {
            IntOrFloat::Float(f) => f as i32,
            IntOrFloat::Int(i) => i,
        }
    }
}

impl Into<u32> for IntOrFloat {
    fn into(self) -> u32 {
        match self {
            IntOrFloat::Float(f) => f as u32,
            IntOrFloat::Int(i) => i as u32,
        }
    }
}

impl fmt::Display for IntOrFloat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            IntOrFloat::Int(n) => n.to_string(),
            IntOrFloat::Float(f) => f.to_string(),
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum NumericValueType {
    Int,
    Float,
}

impl Default for NumericValueType {
    fn default() -> Self {
        NumericValueType::Float
    }
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchemaItem {
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub label: String,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub value: Primitive,
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchemaPropItem {
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub r#enum: Vec<String>,
    #[serde_as(deserialize_as = "DefaultOnError")]
    pub r#type: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct XPrompt {
    pub r#type: String,
    pub message: String,
    pub multiselect: Option<bool>,
    pub items: Option<VecOrString>,
}

impl XPrompt {
    pub fn has_multiselect(&self) -> bool {
        self.multiselect.is_some()
    }

    pub fn has_items(&self) -> bool {
        self.items.is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Primitive {
    Str(String),
    Int(i32),
    Float(f64),
    Bool(bool),
    StringVec(Vec<String>),
    Time(TimeInput),
    Unknown(serde_json::Value),
}

impl fmt::Display for Primitive {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Primitive::Str(s) => s.to_owned(),
            Primitive::Bool(b) => b.to_string().to_owned(),
            Primitive::Int(n) => n.to_string(),
            Primitive::Float(n) => n.to_string(),
            Primitive::StringVec(v) => v.join(","),
            Primitive::Time(v) => format!("{}:{}:{}", v.0, v.1, v.2),
            Primitive::Unknown(v) => v.as_str().unwrap_or("").to_owned(),
        };
        write!(f, "{}", s)
    }
}

impl Into<TimeInput> for Primitive {
    fn into(self) -> TimeInput {
        match self {
            Primitive::Str(s) => {
                let time: Vec<i32> = s
                    .split(":")
                    .map(|s| s.parse::<i32>().unwrap_or_default())
                    .collect();

                if time.len() != 3 {
                    return TimeInput::default();
                }
                TimeInput(time[0], time[1], time[2])
            }
            _ => TimeInput::default(),
        }
    }
}

impl Into<String> for Primitive {
    fn into(self) -> String {
        match self {
            Primitive::Str(s) => s,
            Primitive::Float(f) => f.to_string(),
            Primitive::Int(i) => i.to_string(),
            _ => String::default(),
        }
    }
}

impl Into<bool> for Primitive {
    fn into(self) -> bool {
        match self {
            Primitive::Bool(b) => b,
            _ => false,
        }
    }
}

impl Into<f64> for Primitive {
    fn into(self) -> f64 {
        match self {
            Primitive::Float(f) => f,
            Primitive::Int(i) => i as f64,
            _ => 0.0,
        }
    }
}

impl Into<i32> for Primitive {
    fn into(self) -> i32 {
        match self {
            Primitive::Int(i) => i,
            Primitive::Float(f) => f as i32,
            _ => 0,
        }
    }
}

impl Into<u32> for Primitive {
    fn into(self) -> u32 {
        match self {
            Primitive::Int(i) => i as u32,
            Primitive::Float(f) => f as u32,
            _ => 0,
        }
    }
}

impl Into<DateTime> for Primitive {
    fn into(self) -> DateTime {
        match self {
            Primitive::Str(d) => DateTime::from_iso8601(&d, Some(&TimeZone::utc()))
                .unwrap_or(DateTime::now_utc().unwrap()),
            _ => DateTime::now_utc().unwrap(),
        }
    }
}

impl Into<Vec<String>> for Primitive {
    fn into(self) -> Vec<String> {
        match self {
            Primitive::StringVec(i) => i,
            _ => vec![],
        }
    }
}

impl Default for Primitive {
    fn default() -> Self {
        Primitive::Str(String::default())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StringOrPrompt {
    Str(String),
    Prompt(XPrompt),
}

impl StringOrPrompt {
    pub fn has_multiselect(&self) -> bool {
        match self {
            StringOrPrompt::Prompt(x) => x.has_multiselect(),
            _ => false,
        }
    }

    pub fn has_items(&self) -> bool {
        match self {
            StringOrPrompt::Prompt(x) => x.has_items(),
            _ => false,
        }
    }

    pub fn get_items(&self) -> Vec<String> {
        match self {
            StringOrPrompt::Prompt(x) => x.items.as_ref().unwrap().clone().into(),
            _ => vec![],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VecOrString {
    Str(String),
    StringVec(Vec<String>),
    ItemVec(Vec<SchemaItem>),
}

impl Into<Vec<String>> for VecOrString {
    fn into(self) -> Vec<String> {
        match self {
            VecOrString::Str(s) => vec![s],
            VecOrString::StringVec(v) => v,
            VecOrString::ItemVec(v) => v.into_iter().map(|i| i.value.to_string()).collect(),
        }
    }
}
