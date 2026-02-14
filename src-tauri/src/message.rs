use crate::model::App;

pub fn send(_app: Option<&App>, msg: &str) {
    #[cfg(feature = "gui")]
    {
        if let Some(app) = _app {
            crate::event::message(app, msg);
        }
    }

    println!("{}", msg);
}

pub fn print(_app: Option<&App>, msg: &str) {
    #[cfg(feature = "gui")]
    {
        if let Some(app) = _app {
            crate::event::print(app, msg);
        }
    }

    print!("{}", msg);
}
