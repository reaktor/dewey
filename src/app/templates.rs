use askama::Template; // bring trait in scope

use crate::user::PersonUser;

#[derive(Clone)]
pub struct Flash {
    pub message: String,
    pub level: usize,
}

impl Flash {
    fn new<T: Into<String>>(message: T, level: usize) -> Self {
        Flash {
            message: message.into(),
            level,
        }
    }
}

pub struct Page<'a> {
    pub user_opt: Option<&'a PersonUser>, // the field name should match the variable name
    pub flashes: Vec<Flash>,
}

impl<'a> Default for Page<'a> {
    fn default() -> Self {
        Page {
            user_opt: None,
            flashes: Vec::new(),
        }
    }
}

impl<'a> Page<'a> {
    pub fn info<T: Into<String>>(&mut self, message: T) {
        self.flashes.push(Flash::new(message, 2));
    }
    pub fn person(&mut self, person: &'a PersonUser) {
        self.user_opt = Some(person);
    }
}

#[derive(Template)] // this will generate the code...
#[template(path = "hello.html.j2")] // using the template in this path, relative
                                    // to the templates dir in the crate root
pub struct HelloTemplate<'a> {
    // the name of the struct can be anything
    pub page: Page<'a>, // the field name should match the variable name
                        // in your template
}

#[derive(Template)] // this will generate the code...
#[template(path = "upload.html.j2")] // using the template in this path, relative
                                     // to the templates dir in the crate root
pub struct UploadTemplate<'a> {
    // the name of the struct can be anything
    pub page: Page<'a>, // the field name should match the variable name
                        // in your template
}
