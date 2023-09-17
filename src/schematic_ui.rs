use std::path::PathBuf;

use relm4::gtk::glib::object::Object;
use relm4::gtk::glib::{DateTime, TimeZone};
use relm4::gtk::prelude::{BoxExt, ButtonExt, Cast, FrameExt, IsA, WidgetExt};
use relm4::gtk::Align;
use relm4::{gtk, Component, ComponentParts, ComponentSender};
use serde::{Deserialize, Serialize};

use crate::form_utils::FormUtils;
use crate::schematics::Collection;
use crate::xwidget_builder::XWidgetBuilder;

use std::fmt;
use strum::EnumString;

#[tracker::track]
pub struct SchematicUiModel {
    hidden: bool,
    json: serde_json::Value,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchemaProp {
    pub r#type: String,
    pub description: Option<String>,
    pub default: Option<Primitive>,
    pub alias: Option<String>,
    pub r#enum: Option<Vec<String>>,
    pub items: Option<SchemaPropItem>,
    #[serde(alias = "x-prompt")]
    pub x_prompt: Option<StringOrPrompt>,
    #[serde(alias = "x-widget")]
    pub x_widget: Option<XWidget>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(transparent)]
pub struct XWidget {
    pub options: XWidgetType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum XWidgetType {
    Text(TextEntry),
    Numeric(NumericEntry),
    Slider(NumericEntry),
    Date(DateEntry),
    Color(ColorEntry),
    File(FileEntry),
    Dir(DirEntry),
    Choice(ChoiceEntry),
    Menu(MenuEntry),
    Unknown(serde_json::Value),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FileEntry {
    pub mask: String,
}

impl Default for FileEntry {
    fn default() -> Self {
        FileEntry {
            mask: "*".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DirEntry {
    pub mask: String,
}

impl Default for DirEntry {
    fn default() -> Self {
        DirEntry {
            mask: "*".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DateEntry {
    pub format: String,
}

impl Default for DateEntry {
    fn default() -> Self {
        DateEntry {
            format: "".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ColorEntry {
    pub r#type: ColorEntryType,
    pub format: ColorEntryFormat,
    pub alpha: bool,
}

impl Default for ColorEntry {
    fn default() -> Self {
        ColorEntry {
            r#type: ColorEntryType::Button,
            format: ColorEntryFormat::RGB,
            alpha: false,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColorEntryType {
    Button,
    Input,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChoiceEntry {
    pub r#type: ChoiceType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MenuEntry {
    pub r#type: MenuType,
    pub searchable: bool,
    pub multichoice: bool,
}

impl Default for MenuEntry {
    fn default() -> Self {
        MenuEntry {
            r#type: MenuType::DropDown,
            multichoice: false,
            searchable: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TextEntry {
    pub max_len: i32,
    pub height: i32,
    pub placeholder: String,
    pub icon: String,
    pub icon_position: IconPositionType,
    pub tooltip: String,
    pub hint_text: String,
    pub multiline: bool,
    pub direction: TextAlignmentType,
    pub overwrite: bool,
}

impl Default for TextEntry {
    fn default() -> TextEntry {
        TextEntry {
            hint_text: "".to_string(),
            max_len: 0,
            height: 50,
            placeholder: "".to_string(),
            tooltip: "".to_string(),
            icon: "".to_string(),
            icon_position: IconPositionType::None,
            multiline: false,
            overwrite: false,
            direction: TextAlignmentType::Left,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct NumericEntry {
    pub r#type: NumericType,
    pub stepping: IntOrFloat,
    pub max: IntOrFloat,
    pub min: IntOrFloat,
    pub initial_value: IntOrFloat,
    pub page_increment: IntOrFloat,
    pub page_size: IntOrFloat,
    pub orientation: OrientationType,
}

impl Default for NumericEntry {
    fn default() -> Self {
        NumericEntry {
            r#type: NumericType::Float,
            stepping: IntOrFloat::Float(1.0),
            max: IntOrFloat::Float(f64::MAX),
            min: IntOrFloat::Float(0.0),
            initial_value: IntOrFloat::Float(0.0),
            page_increment: IntOrFloat::Float(5.0),
            page_size: IntOrFloat::Float(1.0),
            orientation: OrientationType::Horizontal,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, EnumString)]
#[serde(rename_all = "snake_case")]
pub enum OrientationType {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, EnumString)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
pub enum IntOrFloat {
    Int(i32),
    Float(f64),
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

impl fmt::Display for IntOrFloat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            IntOrFloat::Int(n) => n.to_string(),
            IntOrFloat::Float(f) => f.to_string(),
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, EnumString, strum::Display)]
#[serde(rename_all = "snake_case")]
pub enum NumericType {
    Int,
    Float,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, EnumString, strum::Display)]
#[serde(rename_all = "snake_case")]
pub enum ChoiceType {
    Checkbox,
    Switch,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, EnumString, strum::Display)]
#[serde(rename_all = "lowercase")]
pub enum MenuType {
    DropDown,
    Combobox,
    Radio,
    Toggle,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, EnumString)]
#[serde(rename_all = "snake_case")]
pub enum IconPositionType {
    Start,
    End,
    None
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, EnumString)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchemaItem {
    pub label: String,
    pub value: Primitive,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchemaPropItem {
    pub r#enum: Vec<String>,
    pub r#type: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct XPrompt {
    pub r#type: String,
    pub message: String,
    pub multiselect: Option<bool>,
    pub items: Option<VecOrString>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Primitive {
    Str(String),
    Int(i32),
    Float(f64),
    Bool(bool),
    StringVec(Vec<String>),
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
            Primitive::Unknown(v) => v.as_str().unwrap_or("").to_owned(),
        };
        write!(f, "{}", s)
    }
}

impl Into<String> for Primitive {
    fn into(self) -> String {
        match self {
            Primitive::Str(s) => s,
            _ => "".to_string(),
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

impl Into<DateTime> for Primitive {
    fn into(self) -> DateTime {
        match self {
            Primitive::Str(d) => {
                let date_str = d + "T00:00:00";
                DateTime::from_iso8601(&date_str, Some(&TimeZone::utc()))
                    .unwrap_or(DateTime::now_utc().unwrap())
            }
            _ => DateTime::now_utc().unwrap(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StringOrPrompt {
    Str(String),
    Prompt(XPrompt),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VecOrString {
    Str(String),
    StringVec(Vec<String>),
    ItemVec(Vec<SchemaItem>),
}

impl SchematicUiModel {
    fn is_a<W: IsA<Object> + IsA<gtk::Widget> + Clone, T: IsA<Object> + IsA<gtk::Widget>>(
        &self,
        widget: &W,
    ) -> bool {
        widget
            .clone()
            .upcast::<gtk::Widget>()
            .downcast::<T>()
            .is_ok()
    }

    fn build_form(&self, json: &serde_json::Value) -> Option<gtk::Box> {
        let utils = FormUtils::new();
        let form = gtk::Box::new(relm4::gtk::Orientation::Vertical, 5);
        form.set_halign(gtk::Align::Start);

        match json["$id"].as_str() {
            Some(_) => {
                let empty = serde_json::Map::new();
                form.append(&utils.label(
                    json["title"].as_str().unwrap_or(""),
                    "schema",
                    Some(Align::Start),
                ));
                let props = json["properties"].as_object().unwrap_or(&empty);
                let keys = props.keys();

                for key in keys {
                    let prop_value: serde_json::Value = props.get(key).unwrap().clone();
                    match serde_json::from_value::<SchemaProp>(prop_value) {
                        Ok(prop) => {
                            let label_text = prop.description.clone().unwrap_or("".to_string());
                            form.append(&utils.label(&label_text, key, Some(Align::Start)));
                            if prop.r#type == "string" {
                                if prop.r#enum == None && prop.items == None {
                                    form.append(&utils.text_input(key, None, prop.default.clone()));
                                } else if prop.r#enum.is_some() {
                                    form.append(&utils.dropdown(
                                        key,
                                        key,
                                        &prop.r#enum.clone().unwrap(),
                                        prop.default.clone(),
                                    ));
                                } else if prop.items.is_some() {
                                    // form.append(&utils.combobox(key, key, prop.items.clone().unwrap()));
                                }
                            } else if prop.r#type == "lala" {
                                let builder = XWidgetBuilder::new(&prop, key.clone());

                                form.append(&builder.get_widget());
                            } else if prop.r#type == "boolean" {
                                form.append(&utils.switch(key, key, prop.default.clone()));
                            } else if prop.r#type == "number" {
                                form.append(&utils.numeric_input(
                                    key,
                                    key,
                                    None,
                                    None,
                                    None,
                                    prop.default.clone(),
                                ));
                            }

                            prop
                        }
                        Err(e) => panic!("{}", e),
                    };
                }
                Some(form)
            }
            None => Some(form),
        }
    }
}

#[derive(Debug)]
pub enum SchematicUiInput {
    Show(PathBuf),
    Submit,
}

#[derive(Debug)]
pub enum SchematicUiOutput {}

#[relm4::component(pub)]
impl Component for SchematicUiModel {
    type Input = SchematicUiInput;
    type Output = SchematicUiOutput;
    type Init = bool;
    type CommandOutput = bool;

    view! {
        #[root]
        gtk::Box {
          set_hexpand: true,
          gtk::Label {
            #[watch]
            set_visible: model.hidden,
            set_hexpand: true,
            set_halign: gtk::Align::Center,
            set_label: "Please, select a schematic!"
          },

          append: frame = &gtk::Frame {
            set_hexpand: true,
            #[track = "model.changed(SchematicUiModel::json())"]
            set_child: Some(&model.build_form(&model.json).unwrap())
          },
          append: submit = &gtk::Button {
            set_label: "Submit",
            connect_clicked[sender] => move |_| {
             sender.input(SchematicUiInput::Submit);
            }
          }


        }
    }

    fn init(
        _init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = SchematicUiModel {
            hidden: true,
            json: serde_json::Value::default(),
            tracker: 0,
        };

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        self.reset();

        match message {
            SchematicUiInput::Show(schema_path) => {
                let json =
                    serde_json::from_str(&Collection::read_str(schema_path.to_str().unwrap()))
                        .unwrap();
                self.set_json(json);
                self.hidden = false
            }
            SchematicUiInput::Submit => {
                let w = &widgets
                    .frame
                    .child()
                    .unwrap()
                    .downcast::<gtk::Box>()
                    .unwrap()
                    .first_child()
                    .unwrap()
                    .next_sibling()
                    .unwrap()
                    .next_sibling()
                    .unwrap()
                    .next_sibling()
                    .unwrap()
                    .next_sibling()
                    .unwrap()
                    .downcast::<gtk::Entry>()
                    .unwrap();
                println!("{}", self.is_a::<_, gtk::Label>(w));
            }
        }

        self.update_view(widgets, sender)
    }

    // fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
    //     self.reset();

    //     match message {
    //         SchematicUiInput::Show(schema_path) => {
    //             let json =  serde_json::from_str(&Collection::read_str(schema_path.to_str().unwrap())).unwrap();
    //             self.set_json(json);
    //             self.hidden = false
    //         }
    //     }
    // }
}
