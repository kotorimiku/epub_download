use crate::model::App;

pub fn send(app: &Option<App>, msg: &str) {
    #[cfg(feature = "gui")]
    {
        if let Some(app) = app {
            crate::event::message(app, msg);
            return;
        }
    }

    println!("{}", msg);
}

pub fn print(app: &Option<App>, msg: &str) {
    #[cfg(feature = "gui")]
    {
        if let Some(app) = app {
            crate::event::message(app, msg);
            return;
        }
    }

    print!("{}", msg);
}
