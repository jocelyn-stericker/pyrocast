use std::cell::Cell;
use vgtk::lib::glib::{
    glib_object_impl, glib_object_subclass, glib_object_wrapper, glib_wrapper, subclass,
    translate::*, value::ToValue, Object, ParamFlags, ParamSpec, Value,
};
use vgtk::lib::gtk::{subclass::prelude::*, Bin, Container, SizeRequestMode, Widget, WidgetExt};

#[derive(Debug)]
pub struct PreferredSizePrivate {
    width: Cell<i32>,
    height: Cell<i32>,
}

impl PreferredSizePrivate {
    fn set_explicit_preferred_width(&self, width: i32) {
        self.width.set(width);
        let w = self.get_instance();
        w.queue_draw();
    }
    fn set_explicit_preferred_height(&self, height: i32) {
        self.height.set(height);
        let w = self.get_instance();
        w.queue_draw();
    }
}

static PROPERTIES: [subclass::Property; 2] = [
    subclass::Property("explicit_preferred_width", |explicit_preferred_width| {
        ParamSpec::int(
            explicit_preferred_width,
            "Preferred-width",
            "The explicit value for the minimum and preferred width",
            -1,
            10000000,
            -1, // Default value
            ParamFlags::READWRITE,
        )
    }),
    subclass::Property("explicit_preferred_height", |explicit_preferred_height| {
        ParamSpec::int(
            explicit_preferred_height,
            "Preferred-height",
            "The explicit value for the minimum and preferred height",
            -1,
            10000000,
            -1, // Default value
            ParamFlags::READWRITE,
        )
    }),
];

impl ObjectSubclass for PreferredSizePrivate {
    const NAME: &'static str = "PreferredSize";
    type ParentType = Bin;
    type Instance = subclass::simple::InstanceStruct<Self>;
    type Class = subclass::simple::ClassStruct<Self>;

    glib_object_subclass!();

    fn class_init(klass: &mut Self::Class) {
        klass.install_properties(&PROPERTIES);
    }

    fn new() -> Self {
        Self {
            width: Cell::new(-1),
            height: Cell::new(-1),
        }
    }
}

impl ObjectImpl for PreferredSizePrivate {
    glib_object_impl!();

    fn set_property(&self, _obj: &Object, id: usize, value: &Value) {
        let prop = &PROPERTIES[id];

        match *prop {
            subclass::Property("explicit_preferred_height", ..) => {
                let explicit_preferred_height = value
                    .get_some()
                    .expect("type conformity checked by `Object::set_property`");
                self.height.set(explicit_preferred_height);
            }
            subclass::Property("explicit_preferred_width", ..) => {
                let explicit_preferred_width = value
                    .get_some()
                    .expect("type conformity checked by `Object::set_property`");
                self.width.set(explicit_preferred_width);
            }
            _ => unimplemented!(),
        }
    }

    fn get_property(&self, _obj: &Object, id: usize) -> Result<Value, ()> {
        let prop = &PROPERTIES[id];

        match *prop {
            subclass::Property("explicit_preferred_height", ..) => Ok(self.height.get().to_value()),
            subclass::Property("explicit_preferred_width", ..) => Ok(self.width.get().to_value()),
            _ => unimplemented!(),
        }
    }
}

impl WidgetImpl for PreferredSizePrivate {
    fn get_request_mode(&self, _as_width: &Widget) -> SizeRequestMode {
        SizeRequestMode::ConstantSize
    }

    fn get_preferred_width(&self, as_widget: &Widget) -> (i32, i32) {
        // (min, nat)
        let size = self.width.get();
        if size == -1 {
            self.parent_get_preferred_width(as_widget)
        } else {
            (size, size)
        }
    }

    fn get_preferred_height(&self, as_widget: &Widget) -> (i32, i32) {
        // (min, nat)
        let size = self.height.get();
        if size == -1 {
            self.parent_get_preferred_height(as_widget)
        } else {
            (size, size)
        }
    }
}

impl ContainerImpl for PreferredSizePrivate {}
impl BinImpl for PreferredSizePrivate {}

glib_wrapper! {
    pub struct PreferredSize(
        Object<subclass::simple::InstanceStruct<PreferredSizePrivate>,
        subclass::simple::ClassStruct<PreferredSizePrivate>,
        SimpleAppWindowClass>)
        @extends Widget, Container, Bin;

    match fn {
        get_type => || PreferredSizePrivate::get_type().to_glib(),
    }
}

pub trait PreferredSizeExt {
    fn set_explicit_preferred_width(&self, explicit_preferred_width: i32);
    fn set_explicit_preferred_height(&self, explicit_preferred_height: i32);
    fn get_explicit_preferred_width(&self) -> i32;
    fn get_explicit_preferred_height(&self) -> i32;
}

impl PreferredSizeExt for PreferredSize {
    fn set_explicit_preferred_width(&self, explicit_preferred_width: i32) {
        let priv_ = PreferredSizePrivate::from_instance(self);
        priv_.set_explicit_preferred_width(explicit_preferred_width);
    }
    fn set_explicit_preferred_height(&self, explicit_preferred_height: i32) {
        let priv_ = PreferredSizePrivate::from_instance(self);
        priv_.set_explicit_preferred_height(explicit_preferred_height);
    }
    fn get_explicit_preferred_width(&self) -> i32 {
        PreferredSizePrivate::from_instance(self).width.get()
    }
    fn get_explicit_preferred_height(&self) -> i32 {
        PreferredSizePrivate::from_instance(self).height.get()
    }
}
