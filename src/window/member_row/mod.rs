mod imp;

use glib::Object;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::glib;

use crate::window::member_object::MemberObject;

glib::wrapper!
{
    pub struct MemberRow(ObjectSubclass<imp::MemberRow>)
    @extends gtk::Box, gtk::Widget,
    @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl Default for MemberRow
{
    fn default() -> Self
    {
        return Self::new();
    }
}

impl MemberRow
{
    pub fn new() -> Self
    {
        return Object::builder().build();
    }

    pub fn bind(&self, member_object: &MemberObject)
    {
        let member_label = self.imp().member_label.get();
        let mut bindings = self.imp().bindings.borrow_mut();

        let member_label_binding = member_object
            .bind_property("member-name", &member_label, "label")
            .sync_create()
            .build();

        bindings.push(member_label_binding);
    }

    pub fn unbind(&self)
    {
        for binding in self.imp().bindings.borrow_mut().drain(..)
        {
            binding.unbind();
        }
    }
}
