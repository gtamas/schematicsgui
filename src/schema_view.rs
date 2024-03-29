use std::path::PathBuf;

use relm4::gtk::prelude::{OrientableExt, TextBufferExt, TextViewExt, WidgetExt};
use relm4::gtk::Orientation;
use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent};
use serde_json::Value;
use sourceview5::prelude::ViewExt;
use sourceview5::Buffer;

use crate::schematics::Collection;
use crate::traits::JsonBuffer;

pub struct SchemaViewModel {
    hidden: bool,
    json: Buffer,
    title: String,
}

#[derive(Debug)]
pub enum SchemaViewInput {
    Show(PathBuf),
}

#[derive(Debug)]
pub enum SchemaViewOutput {}

impl JsonBuffer for SchemaViewModel {}

#[relm4::component(pub)]
impl SimpleComponent for SchemaViewModel {
    type Input = SchemaViewInput;
    type Output = SchemaViewOutput;
    type Init = bool;

    view! {
        #[root]
        gtk::Box {
          set_hexpand: true,
          set_vexpand: true,
          set_orientation: Orientation::Vertical,
          set_css_classes: &["content_area"],
          gtk::Label {
            #[watch]
            set_visible: model.hidden,
            set_vexpand: true,
            set_halign: gtk::Align::Center,
            set_label: "Please, select a schematic!"
          },
          gtk::ScrolledWindow {
          #[watch]
          set_visible: !model.hidden,
          set_hscrollbar_policy: gtk::PolicyType::Never,
            sourceview5::View {
                  #[watch]
                  set_visible: !model.hidden,
                  set_editable: false,
                  set_hexpand: true,
                  set_vexpand: true,
                  set_show_line_numbers: true,
                  set_highlight_current_line: true,
                  set_tab_width: 4,
                  set_monospace: true,
                  set_buffer: Some(&model.json)
            }
          }
        }
    }

    fn init(
        _init: Self::Init,
        root: &Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = SchemaViewModel {
            hidden: true,
            title: String::from(""),
            json: Self::get_json_buffer(None),
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            SchemaViewInput::Show(schema_path) => {
                let schema: Value =
                    serde_json::from_str(&Collection::read_str(schema_path.to_str().unwrap()))
                        .unwrap();
                self.title = schema["title"].as_str().unwrap_or("").to_string();
                self.json
                    .set_text(&serde_json::to_string_pretty(&schema).unwrap());
                self.hidden = false
            }
        }
    }
}
