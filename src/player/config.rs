use web_sys::HtmlCanvasElement;

#[derive(Default, Debug, Clone)]
pub struct Config {
    pub canvas: Option<HtmlCanvasElement>,
    pub attrs: Attributes,
}

macro_rules! attributes {
    {$($name:ident,)*} => {
        #[derive(Default, Debug, Clone)]
        pub struct Attributes {
            $(pub $name: Option<String>,)*
        }

		impl Attributes {
			pub fn names() -> &'static [&'static str] {
				&[$(stringify!($name),)*]
			}

            pub fn update(
                &mut self,
                name: String,
                value: Option<String>,
            ) {
                match name.as_str() {
                    $(stringify!($name) => self.$name = value,)*
                    _ => unreachable!(),
                };
            }
        }
    };
}

// Makes a attr of Option<String> types
attributes! {
    src,
    broadcast,
}
