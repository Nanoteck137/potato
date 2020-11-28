#![no_std]

#![allow(dead_code)]

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum Category {
    Bootloader,
    Kernel,
}

impl Default for Category {
    fn default() -> Self {
        Self::Bootloader
    }
}

pub struct OptionParser<'a> {
    text: &'a str
}

impl<'a> OptionParser<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text
        }
    }

    pub fn options<F>(&self, mut func: F) -> Option<()>
        where F: FnMut(Category, &str, &str) -> Option<()>
    {
        let mut current_category = Category::default();

        for line in self.text.lines() {
            let line = line.trim();

            if line.is_empty() {
                continue;
            }

            if &line[0..1] == "[" {
                let category = &line[1..line.len() - 1];

                match category {
                    "bootloader" => {
                        current_category = Category::Bootloader;
                    }

                    "kernel" => {
                        current_category = Category::Kernel;
                    }

                    _ => {
                        panic!("Unknown category");
                    }
                }
            } else {
                let index = line.find('=')?;

                let key = &line[0..index];
                let value = &line[index+1..];

                func(current_category, key, value)?;
            }
        }

        Some(())
    }
}
