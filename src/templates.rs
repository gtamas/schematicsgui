use gtk::prelude::OrientableExt;
use relm4::{
    gtk::{self, prelude::WidgetExt},
    WidgetTemplate,
};

#[relm4::widget_template(pub)]
impl WidgetTemplate for DateTimeSpinButton {
    view! {
        gtk::SpinButton {
            set_orientation: gtk::Orientation::Vertical,
            set_value: 0.0,
            set_increments: (1.0, 10.0),
            set_digits: 0,
            set_numeric: true,
            set_wrap: true,
            set_update_policy: gtk::SpinButtonUpdatePolicy::IfValid,
            set_snap_to_ticks: true,
        },
    }
}

#[relm4::widget_template(pub)]
impl WidgetTemplate for TimeInput {
    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            #[template]
            DateTimeSpinButton {
                set_widget_name: "hours",
                set_range: (0.0, 23.0),
            },
            #[template]
            DateTimeSpinButton {
                set_widget_name: "minutes",
                set_range: (0.0, 59.0),
            },
            #[template]
            DateTimeSpinButton {
                set_widget_name: "seconds",
                set_range: (0.0, 59.0),
            },

        }
    }
}
