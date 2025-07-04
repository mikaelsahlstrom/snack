use std::cell::RefCell;

use glib::subclass::InitializingObject;
use gtk::subclass::prelude::*;
use gtk::{ gio, glib, TextView, CompositeTemplate, Entry, Button, ListView };

// Object holding the state
#[derive(CompositeTemplate, Default)]
#[template(resource = "/org/gtk_rs/snack/main.ui")]
pub struct Window
{
    #[template_child]
    pub chat: TemplateChild<TextView>,
    #[template_child]
    pub entry: TemplateChild<Entry>,
    #[template_child]
    pub send: TemplateChild<Button>,
    #[template_child]
    pub members_list: TemplateChild<ListView>,
    pub members: RefCell<Option<gio::ListStore>>
}

#[glib::object_subclass]
impl ObjectSubclass for Window
{
    const NAME: &'static str = "snackMain";
    type Type = super::Window;
    type ParentType = gtk::ApplicationWindow;

    fn class_init(class: &mut Self::Class)
    {
        class.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>)
    {
        obj.init_template();
    }
}

impl ObjectImpl for Window
{
    fn constructed(&self)
    {
        self.parent_constructed();
        self.obj().setup_members();
        self.obj().setup_callbacks();
        self.obj().setup_factory();
    }
}

impl WidgetImpl for Window {}

impl WindowImpl for Window {}

impl ApplicationWindowImpl for Window {}
