use serde::Serialize;

pub mod taxa;

pub struct JsonView<'a, T: Serialize> {
    obj: &'a T,
}

impl<'a, T: Serialize> JsonView<'a, T> {
    pub fn new(obj: &'a T) -> Self {
        Self { obj }
    }

    pub fn render(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string_pretty(self.obj)?)
    }
}

pub struct YamlView<'a, T: Serialize> {
    obj: &'a T,
}

impl<'a, T: Serialize> YamlView<'a, T> {
    pub fn new(obj: &'a T) -> Self {
        Self { obj }
    }

    pub fn render(&self) -> anyhow::Result<String> {
        Ok(serde_yaml::to_string(self.obj)?)
    }
}
