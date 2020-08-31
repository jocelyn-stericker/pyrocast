use vgtk::lib::gtk::{prelude::*, Box, Label, Orientation};
use vgtk::{gtk, Component, UpdateAction, VNode};

#[derive(Debug, Default, Clone)]
pub struct SettingsTab {}

#[derive(Clone, Debug)]
pub enum Message {}

impl Component for SettingsTab {
    type Message = Message;
    type Properties = ();

    fn update(&mut self, _message: Message) -> UpdateAction<Self> {
        UpdateAction::None
    }

    fn change(&mut self, _props: Self::Properties) -> UpdateAction<Self> {
        UpdateAction::None
    }

    fn view(&self) -> VNode<SettingsTab> {
        gtk! {
            <Box orientation=Orientation::Vertical hexpand=true vexpand=true>
                <Label label="To implement" vexpand=true />
            </Box>
        }
    }
}
