mod imp;
mod member_object;
mod member_row;

use glib::Object;
use gtk::glib::object::Cast;
use gtk::subclass::prelude::*;
use gtk::{ gio, glib, Application, NoSelection, prelude::* };

use crate::window::member_object::MemberObject;
use crate::window::member_row::MemberRow;

glib::wrapper!
{
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Window
{
    pub fn new(app: &Application) -> Self
    {
        return Object::builder().property("application", app).build();
    }

    fn members(&self) -> gio::ListStore
    {
        return self.imp().members.borrow().clone().expect("Members list store is not set");
    }

    fn setup_members(&self)
    {
        let model = gio::ListStore::new::<MemberObject>();
        self.imp().members.replace(Some(model));

        let selection = NoSelection::new(Some(self.members()));
        self.imp().members_list.set_model(Some(&selection));
    }

    pub fn new_member(&self, name: &str)
    {
        let member = MemberObject::new(name.to_string());
        self.members().append(&member);
    }

    pub fn remove_member(&self, member_name: &str)
    {
        let members = self.members();
        let mut position = 0;
        while let Some(item) = members.item(position)
        {
            if let Some(member_object) = item.downcast_ref::<MemberObject>()
            {
                if member_object.member_name() == member_name
                {
                    members.remove(position);
                    return;
                }
            }

            position += 1;
        }
    }

    pub fn add_chat_row(&self, text: &str)
    {
        let chat = self.imp().chat.clone();
        let buffer = chat.buffer();
        buffer.insert(&mut buffer.end_iter(), &format!("{}\n", text));
    }

    fn setup_factory(&self)
    {
        let factory = gtk::SignalListItemFactory::new();

        factory.connect_setup(move |_, item|
        {
            let row = MemberRow::new();
            item.downcast_ref::<gtk::ListItem>()
                .expect("Item should be a ListItem")
                .set_child(Some(&row));
        });

        factory.connect_bind(move |_, item| {
            let row = item.downcast_ref::<gtk::ListItem>()
                .expect("Item should be a ListItem")
                .child()
                .and_downcast::<MemberRow>()
                .expect("Child should be a MemberRow");

            let member_object = item.downcast_ref::<gtk::ListItem>()
                .expect("Item should be a ListItem")
                .item()
                .and_downcast::<MemberObject>()
                .expect("Child should be a MemberObject");

            row.bind(&member_object);
        });

        factory.connect_unbind(move |_, item| {
            let row = item.downcast_ref::<gtk::ListItem>()
                .expect("Item should be a ListItem")
                .child()
                .and_downcast::<MemberRow>()
                .expect("Child should be a MemberRow");

            row.unbind();
        });

        self.imp().members_list.set_factory(Some(&factory));
    }
}
