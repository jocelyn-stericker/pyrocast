use vgtk::lib::gtk::{prelude::*, Box, Label, Orientation};
use vgtk::{gtk, Component, UpdateAction, VNode};

#[derive(Debug, Default, Clone)]
pub struct HomeTab {}

#[derive(Clone, Debug)]
pub enum Message {}

impl Component for HomeTab {
    type Message = Message;
    type Properties = ();

    fn update(&mut self, _message: Message) -> UpdateAction<Self> {
        UpdateAction::None
    }

    fn change(&mut self, _props: Self::Properties) -> UpdateAction<Self> {
        UpdateAction::None
    }

    fn view(&self) -> VNode<HomeTab> {
        gtk! {
            <Box orientation=Orientation::Vertical hexpand=true vexpand=true>
                <Label label="When you add podcasts, they will appear here." vexpand=true />
            </Box>
        }
    }
}
