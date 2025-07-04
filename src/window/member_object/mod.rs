mod imp;

use glib::Object;
use gtk::glib;

glib::wrapper!
{
    pub struct MemberObject(ObjectSubclass<imp::MemberObject>);
}

impl MemberObject
{
    pub fn new(member_name: String) -> Self
    {
        return Object::builder().property("member-name", member_name).build();
    }
}

#[derive(Default)]
pub struct MemberData
{
    pub member_name: String,
}
