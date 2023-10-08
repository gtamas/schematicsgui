use relm4::gtk::glib::object::Object;
use relm4::gtk::prelude::{BoxExt, ButtonExt, Cast, FrameExt, IsA, OrientableExt, WidgetExt};
use relm4::gtk::{Align, ApplicationWindow, DialogFlags, MessageDialog};
use relm4::{gtk, Component, ComponentParts, ComponentSender};
use std::path::PathBuf;

use crate::command_builder::{CommandBuilder, Param};
use crate::default_widget_builder::DefaultWidgetBuilder;
use crate::form_utils::FormUtils;
use crate::schema_parsing::SchemaProp;
use crate::schematics::Collection;
use crate::value_extractor::ValueExtractor;
use crate::xwidget_builder::XWidgetBuilder;

#[tracker::track]
#[derive(Debug, PartialEq)]
pub struct SchematicUiModel {
    hidden: bool,
    json: serde_json::Value,
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

    fn build_form(&self, parent: &gtk::Frame, json: &serde_json::Value) -> Option<gtk::Box> {
        let utils = FormUtils::new();
        let form = gtk::Box::new(relm4::gtk::Orientation::Vertical, 5);
        form.set_css_classes(&["ui"]);
        form.set_hexpand(true);

        match json["$id"].as_str() {
            Some(_) => {
                let empty = serde_json::Map::new();
                form.append(&utils.label(
                    &json["title"].as_str().unwrap_or("").replace("schema", ""),
                    "schema",
                    None,
                    Some(vec! ["label_title"])
                ));
                let props = json["properties"].as_object().unwrap_or(&empty);
                let keys = props.keys();

                for key in keys {
                    let prop_value: serde_json::Value = props.get(key).unwrap().clone();
                    match serde_json::from_value::<SchemaProp>(prop_value) {
                        Ok(prop) => {
                            let label_text = prop.description.clone().unwrap_or(String::default());
                            form.append(&utils.label(&label_text, key, None, None));
                            if prop.x_widget.is_some() {
                                let builder = XWidgetBuilder::new(&prop, key.clone());

                                form.append(&builder.get_widget());
                            } else {
                                let builder = DefaultWidgetBuilder::new(&prop, key.clone());

                                form.append(&builder.get_widget());
                            }

                            prop
                        }
                        Err(e) => {
                            let window: ApplicationWindow = parent
                                .root()
                                .unwrap()
                                .downcast::<ApplicationWindow>()
                                .unwrap();
                            let dialog = MessageDialog::new(
                                Some(&window),
                                DialogFlags::all(),
                                gtk::MessageType::Error,
                                gtk::ButtonsType::YesNo,
                                format!("{}", "Oops.. an error has occured!"),
                            );
                            dialog.set_secondary_text(Some(&format!(
                                "{}\n{}",
                                e, "Do you wish to try again?"
                            )));
                            dialog.show();
                            return None;
                        }
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
pub enum SchematicUiOutput {
    Params(Vec<Param>),
}

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
          set_orientation: relm4::gtk::Orientation::Vertical,
          gtk::Label {
            #[watch]
            set_visible: model.hidden,
            set_hexpand: true,
            set_vexpand: true,
            set_halign: gtk::Align::Center,
            set_label: "Please, select a schematic!"
          },

          append: frame = &gtk::Frame {
            set_hexpand: true,
            set_css_classes: &["ui_container"],
            #[track = "model.changed(SchematicUiModel::json())"]
            set_child: Some(&model.build_form(&frame, &model.json).unwrap())
          },
          append: submit = &gtk::Button {
            set_label: "Submit",
            #[watch]
            set_visible: !model.hidden,
            connect_clicked[sender] => move |_| {
             sender.input(SchematicUiInput::Submit);
            },
            set_halign: Align::End,
            set_valign: Align::Start,
            set_css_classes: &["action"]
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
                let mut w = widgets
                    .frame
                    .child()
                    .unwrap()
                    .downcast::<gtk::Box>()
                    .unwrap()
                    .first_child();
                let mut command = CommandBuilder::new(None);

                loop {
                    let widget = w.as_ref().unwrap();

                    let extractor = ValueExtractor::new(widget);
                    let param: Param = extractor.get_name_value();

                    if param.name.len() > 0 {
                        command.add(param);
                    }

                    w = w.as_ref().unwrap().next_sibling();

                    if w.is_none() {
                        break;
                    }
                }

                sender
                    .output_sender()
                    .emit(SchematicUiOutput::Params(command.to_params()));
            }
        }

        self.update_view(widgets, sender)
    }
}
