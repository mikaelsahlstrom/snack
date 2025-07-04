use std::cell::RefCell;

use glib::Binding;
use gtk::subclass::prelude::*;
use gtk::{ glib, CompositeTemplate, Label };

#[derive(Default, CompositeTemplate)]
#[template(resource = "/org/gtk_rs/snack/member_row.ui")]
pub struct MemberRow
{
    #[template_child]
    pub member_label: TemplateChild<Label>,
    pub bindings: RefCell<Vec<Binding>>,
}

#[glib::object_subclass]
impl ObjectSubclass for MemberRow
{
    const NAME: &'static str = "MemberRow";
    type Type = super::MemberRow;
    type ParentType = gtk::Box;

    fn class_init(klass: &mut Self::Class)
    {
        klass.bind_template();
    }

    fn instance_init(obj: &glib::subclass::InitializingObject<Self>)
    {
        obj.init_template();
    }
}

impl ObjectImpl for MemberRow {}

impl WidgetImpl for MemberRow {}

impl BoxImpl for MemberRow {}
