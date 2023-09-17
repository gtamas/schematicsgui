use relm4::gtk::prelude::{EditableExt, EntryBufferExtManual, EntryExt, OrientableExt, WidgetExt};
use relm4::typed_list_view::TypedListView;
use relm4::{gtk, ComponentParts, ComponentSender, SimpleComponent};

use crate::schematics::Collection;
use crate::settings_utils::SettingsUtils;
use crate::string_list_item::StringListItem;

pub struct SchematicSelectorModel {
    hidden: bool,
    list_view_wrapper: TypedListView<StringListItem, gtk::SingleSelection>,
    search: gtk::EntryBuffer,
    schematics: Vec<String>,
}

impl SchematicSelectorModel {
    fn load_options() -> Vec<String> {
        let settings_util = SettingsUtils::new();
        let collection_utils: Collection;

        if settings_util.exists() {
            collection_utils = Collection::new(&settings_util.read().schematics_collection);
            return collection_utils.list_schematic_names();
        } else {
            vec!["".to_string()]
        }
    }
}

#[derive(Debug)]
pub enum SchematicSelectorInput {
    Show,
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
        set_hexpand: false,
        set_width_request: 300,
        gtk::Entry {
          set_buffer: &model.search,
          set_css_classes: &["search", "inputText"],
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
            schematics: Self::load_options(),
            search: gtk::EntryBuffer::default(),
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

            SchematicSelectorInput::Show => {
                self.schematics = Self::load_options();
                for schematic in &self.schematics {
                    self.list_view_wrapper
                        .append(StringListItem::new(schematic.to_string()));
                }
                self.hidden = false;
            }
        }
    }
}
