use relm4::gtk::prelude::{EditableExt, EntryBufferExtManual, EntryExt, OrientableExt, WidgetExt};
use relm4::typed_list_view::TypedListView;
use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent};

use crate::schematics::Collection;
use crate::settings_utils::{SettingsData, SettingsUtils};
use crate::string_list_item::StringListItem;

pub struct SchematicSelectorModel {
    hidden: bool,
    list_view_wrapper: TypedListView<StringListItem, gtk::SingleSelection>,
    search: gtk::EntryBuffer,
    schematics: Vec<String>,
    settings: Option<SettingsData>,
}

impl SchematicSelectorModel {
    fn load_options(&self) -> Vec<String> {
        let settings_util = SettingsUtils::new();
        let mut collection_utils: Collection;
        let settings = self.settings.as_ref().unwrap();

        if settings_util.exists() {
            collection_utils = Collection::new(settings.clone());
            collection_utils.init();
            return collection_utils.list_schematic_names();
        } else {
            vec![String::default()]
        }
    }
}

#[derive(Debug)]
pub enum SchematicSelectorInput {
    Show(SettingsData),
    FilterChange,
    Selected(u32),
}

#[derive(Debug)]
pub enum SchematicSelectorOutput {
    Selected(String),
    Load,
}

pub struct ComponentInit {}

#[relm4::component(pub)]
impl SimpleComponent for SchematicSelectorModel {
    type Input = SchematicSelectorInput;
    type Output = SchematicSelectorOutput;
    type Init = bool;

    view! {
        #[root]
        gtk::Box {
        #[watch]
        set_visible: !model.hidden,
        set_orientation: gtk::Orientation::Vertical,
        set_css_classes: &["schematics_selector"],
        set_hexpand: false,
        set_width_request: 300,
        gtk::Entry {
          set_buffer: &model.search,
          set_css_classes: &["search", "text_input"],
          set_vexpand: false,
          set_hexpand: false,
          set_width_request: 300,
          set_margin_bottom: 10,
          set_icon_from_icon_name[Some("system-search")]: gtk::EntryIconPosition::Primary,
          connect_changed[sender] => move |_| {
             sender.input(SchematicSelectorInput::FilterChange);
            }
        },
        gtk::ScrolledWindow {
          set_vexpand: true,
          set_hexpand: true,
          set_hscrollbar_policy: gtk::PolicyType::Never,
          set_max_content_width: 300,
          set_css_classes: &["collection_list"],
          #[local_ref]
          my_view -> gtk::ListView {
            set_single_click_activate: true,
            connect_activate[sender] => move |_, selected| {
             sender.input(SchematicSelectorInput::Selected(selected));
            }
          }
        }
        }
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let list_view_wrapper: TypedListView<StringListItem, gtk::SingleSelection> =
            TypedListView::with_sorting();

        let model = SchematicSelectorModel {
            hidden: init,
            list_view_wrapper,
            schematics: vec![String::default()],
            search: gtk::EntryBuffer::default(),
            settings: None,
        };

        let my_view = &model.list_view_wrapper.view;

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            SchematicSelectorInput::Selected(n) => {
                let item = self.list_view_wrapper.get(n).unwrap();
                let _ = sender.output(SchematicSelectorOutput::Selected(
                    item.borrow().value.clone(),
                ));
            }
            SchematicSelectorInput::FilterChange => {
                let query_str = self.search.text().to_string();
                self.list_view_wrapper.pop_filter();
                self.list_view_wrapper
                    .add_filter(move |item| item.value.starts_with(&query_str));
                self.list_view_wrapper.set_filter_status(0, true);
            }

            SchematicSelectorInput::Show(settings) => {
                self.settings = Some(settings);
                self.schematics = self.load_options();
                self.list_view_wrapper.clear();
                self.list_view_wrapper.clear_filters();

                for schematic in &self.schematics {
                    self.list_view_wrapper
                        .append(StringListItem::new(schematic.to_string()));
                }
                self.hidden = false;
            }
        }
    }
}
