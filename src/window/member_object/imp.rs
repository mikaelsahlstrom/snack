use std::cell::RefCell;

use glib::Properties;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use super::MemberData;

#[derive(Properties, Default)]
#[properties(wrapper_type = super::MemberObject)]
pub struct MemberObject
{
    #[property(name = "member-name", get, set, type = String, member = member_name)]
    pub data: RefCell<MemberData>,
}

#[glib::object_subclass]
impl ObjectSubclass for MemberObject
{
    const NAME: &'static str = "MemberObject";
    type Type = super::MemberObject;
}

#[glib::derived_properties]
impl ObjectImpl for MemberObject {}
