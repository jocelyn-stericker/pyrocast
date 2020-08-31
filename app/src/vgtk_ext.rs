use libhandy::{Leaflet, LeafletExt};
use std::collections::HashMap;
use vgtk::lib::glib::object::{CanDowncast, Cast, IsA, ObjectType};
use vgtk::lib::gtk::{
    ComboBoxExt, ComboBoxText, ComboBoxTextExt, Container, ContainerExt, FlowBox, FlowBoxChild,
    FlowBoxChildExt, FlowBoxExt, RangeExt, Scale, Stack, StackExt, TreeModelExt, Widget, WidgetExt,
};

pub trait GetNamedDescendants {
    fn get_named_descendants(&self) -> HashMap<String, Widget>;
}

impl<O: IsA<Widget>> GetNamedDescendants for O {
    fn get_named_descendants(&self) -> HashMap<String, Widget> {
        let mut map = HashMap::new();
        let mut widgets: Vec<Widget> = vec![self.upcast_ref::<Widget>().clone()];

        while let Some(widget) = widgets.pop() {
            let name = widget.get_widget_name();
            if !name.is_empty() {
                map.insert(name.to_string(), widget.clone());
            }

            if let Some(widget) = widget.downcast_ref::<Container>() {
                for child in widget.get_children() {
                    widgets.push(child);
                }
            }
        }

        map
    }
}

pub trait GetAsWidgetType {
    fn get_downcast<T: ObjectType>(&self, name: &str) -> Option<&T>
    where
        Widget: CanDowncast<T>;
}

impl GetAsWidgetType for HashMap<String, Widget> {
    fn get_downcast<T: ObjectType>(&self, name: &str) -> Option<&T>
    where
        Widget: CanDowncast<T>,
    {
        self.get(name).and_then(|val| val.downcast_ref::<T>())
    }
}

pub trait LeafletExtHelpers {
    /// get_child_visible and set_child_visible are taken
    fn get_child_is_visible_child<T: IsA<Widget>>(&self, child: &T) -> bool;
    fn set_child_is_visible_child<T: IsA<Widget>>(&self, child: &T, visible: bool);
}

impl LeafletExtHelpers for Leaflet {
    fn get_child_is_visible_child<T: IsA<Widget>>(&self, child: &T) -> bool {
        self.get_visible_child()
            .map(|c| &c == child)
            .unwrap_or(false)
    }

    fn set_child_is_visible_child<T: IsA<Widget>>(&self, child: &T, visible: bool) {
        // At least one must be selected.
        if visible {
            self.set_visible_child(child);
        }
    }
}

pub trait FlowBoxExtHelpers {
    fn get_child_selected<T: IsA<FlowBoxChild>>(&self, child: &T) -> bool;
    fn set_child_selected<T: IsA<FlowBoxChild>>(&self, child: &T, selected: bool);
}

impl FlowBoxExtHelpers for FlowBox {
    fn get_child_selected<T: IsA<FlowBoxChild>>(&self, child: &T) -> bool {
        child.is_selected()
    }

    fn set_child_selected<T: IsA<FlowBoxChild>>(&self, child: &T, selected: bool) {
        if selected {
            self.select_child(child);
        } else {
            self.unselect_child(child);
        }
    }
}

pub trait StackExtHelpers {
    fn get_child_selected<T: IsA<Widget>>(&self, child: &T) -> bool;
    fn set_child_selected<T: IsA<Widget>>(&self, child: &T, selected: bool);
}

impl StackExtHelpers for Stack {
    fn get_child_selected<T: IsA<Widget>>(&self, child: &T) -> bool {
        self.get_visible_child().as_ref() == Some(child.upcast_ref())
    }

    fn set_child_selected<T: IsA<Widget>>(&self, child: &T, selected: bool) {
        if selected {
            self.set_visible_child(child)
        }
    }
}

pub trait RangeExtHelpers {
    fn get_range_pair(&self) -> (f64, f64);
    fn set_range_pair(&self, range: (f64, f64));
}

impl RangeExtHelpers for Scale {
    fn get_range_pair(&self) -> (f64, f64) {
        // TODO
        (0.0, 0.0)
    }

    fn set_range_pair(&self, range: (f64, f64)) {
        self.set_range(range.0, range.1)
    }
}

pub trait ComboBoxExtHelpers {
    fn get_options(&self) -> Vec<(String, String)>;
    fn set_options(&self, options: Vec<(String, String)>);
}

impl ComboBoxExtHelpers for ComboBoxText {
    fn get_options(&self) -> Vec<(String, String)> {
        let column = self.get_entry_text_column();
        if let Some(model) = self.get_model() {
            let mut res = vec![];
            model.foreach(|model, _path, iter| {
                let val = model
                    .get_value(iter, column)
                    .get::<String>()
                    .ok()
                    .unwrap_or_default()
                    .unwrap_or_default();
                let id = model
                    .get_value(iter, 1)
                    .get::<String>()
                    .ok()
                    .unwrap_or_default()
                    .unwrap_or_default();
                res.push((id, val));

                false
            });
            res
        } else {
            vec![]
        }
    }

    fn set_options(&self, options: Vec<(String, String)>) {
        self.remove_all();
        for (id, text) in options {
            self.append(Some(&id), &text);
        }
    }
}
