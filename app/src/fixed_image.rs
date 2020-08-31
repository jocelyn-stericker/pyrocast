use state::{Image as ImageObj, StateError};
use std::sync::Arc;
use vgtk::lib::gdk_pixbuf::{InterpType, Pixbuf, PixbufLoader, PixbufLoaderExt};
use vgtk::lib::gtk::{prelude::*, Image};
use vgtk::{ext::*, gtk, Component, UpdateAction, VNode};

#[derive(Clone, Default, Debug, PartialEq)]
pub struct Props {
    pub width: i32,
    pub image: Option<Arc<Result<ImageObj, StateError>>>,
}

#[derive(Debug, Default, Clone)]
pub struct FixedImage {
    props: Props,
    did_init: bool,
}

#[derive(Clone, Debug)]
pub enum Message {
    MaybeInit,
}

fn to_pixbuf(image: &ImageObj) -> Option<Pixbuf> {
    let loader = PixbufLoader::with_mime_type(image.mimetype.as_ref()?).ok()?;
    loader.write(image.data.as_ref()?).ok()?;
    loader.close().ok()?;

    Some(loader.get_pixbuf()?)
}

impl Component for FixedImage {
    type Message = Message;
    type Properties = Props;

    fn create(props: Self::Properties) -> Self {
        FixedImage {
            props,
            did_init: false,
        }
    }

    fn change(&mut self, mut props: Self::Properties) -> UpdateAction<Self> {
        std::mem::swap(&mut self.props, &mut props);
        let prev_props = props;

        if prev_props != self.props {
            UpdateAction::Render
        } else {
            UpdateAction::None
        }
    }

    fn view(&self) -> VNode<FixedImage> {
        let size = self.props.width;

        let image = self.props.image.as_ref();
        let image = image.and_then(|img| img.as_ref().as_ref().ok());
        let pixbuf = image
            .and_then(|img| to_pixbuf(&img))
            .as_ref()
            .and_then(|p| p.scale_simple(size, size, InterpType::Bilinear));

        gtk! {
            <Image
                on size_allocate=async |_, _| Message::MaybeInit
                property_width_request=size
                property_height_request=size
                visible=true
                pixbuf=pixbuf
            />
        }
    }
}
