mod window;

use gtk::prelude::*;
use gtk::{ gio, glib, Application };

use log::debug;

const APP_ID: &str = "org.gtk_rs.snack";

#[tokio::main]
async fn main() -> glib::ExitCode
{
    env_logger::init();

    debug!("Starting Snack application");
    gio::resources_register_include!("snack_resources.gresource")
        .expect("Failed to register resources");

    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    return app.run();
}

fn build_ui(app: &Application)
{
    let window = window::Window::new(app);
    window.set_default_size(800, 600);
    window.present();
}
