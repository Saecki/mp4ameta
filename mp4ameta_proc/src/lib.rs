use proc_macro::TokenStream;

fn base_values(input: TokenStream) -> (String, String, String, String, String) {
    let input_string = input.to_string();
    let mut strings = input_string.split(',');

    let value_ident = strings
        .next()
        .expect("Missing first positional argument: value identifier")
        .trim()
        .replace("\"", "");
    if value_ident.is_empty() {
        panic!("Found empty value identifier.");
    }

    let name = value_ident.replace('_', " ");

    let mut name_chars = name.chars();
    let headline = name_chars.next().unwrap().to_uppercase().chain(name_chars).collect::<String>();

    let atom_ident = format!("atom::{}", value_ident.to_uppercase());

    let atom_ident_string = strings
        .next()
        .expect("Missing second positional argument: atom ident string")
        .trim()
        .replace("\"", "");
    if atom_ident_string.is_empty() {
        panic!("Found empty atom identifier string.");
    }

    if let Some(arg) = strings.next().map(|s| s.trim()) {
        if !arg.is_empty() {
            panic!("Found unexpected third positional argument: {}.", arg);
        }
    }

    (value_ident, name, headline, atom_ident, atom_ident_string)
}

#[proc_macro]
pub fn individual_string_value_accessor(input: TokenStream) -> TokenStream {
    let (value_ident, name, headline, atom_ident, atom_ident_string) = base_values(input);

    format!(
        "
/// ### {0}
impl Tag {{
    /// Returns the {1} (`{2}`).
    pub fn {3}(&self) -> Option<&str> {{
        self.string({4}).next()
    }}

    /// Consumes and returns the {1} (`{2}`).
    pub fn take_{3}(&mut self) -> Option<String> {{
        self.take_string({4}).next()
    }}

    /// Sets the {1} (`{2}`).
    pub fn set_{3}(&mut self, {3}: impl Into<String>) {{
        self.set_data({4}, Data::Utf8({3}.into()));
    }}

    /// Removes the {1} (`{2}`).
    pub fn remove_{3}(&mut self) {{
        self.remove_data({4});
    }}

    /// Returns the {1} formatted in an easily readable way.
    fn format_{3}(&self) -> Option<String> {{
        match self.{3}() {{
            Some(s) => Some(format!(\"{3}: {{}}\\n\", s)),
            None => None,
        }}
    }}
}}
    ",
        headline, name, atom_ident_string, value_ident, atom_ident,
    )
    .parse()
    .expect("Error parsing accessor impl block:")
}

#[proc_macro]
pub fn multiple_string_values_accessor(input: TokenStream) -> TokenStream {
    let (value_ident, name, headline, atom_ident, atom_ident_string) = base_values(input);

    let mut value_ident_plural = value_ident.clone();
    if value_ident_plural.ends_with('y') {
        value_ident_plural.pop();
        value_ident_plural.push_str("ies");
    } else {
        value_ident_plural.push('s');
    };

    let name_plural = value_ident_plural.replace('_', " ");

    format!(
        "
/// ### {0}
impl Tag {{
    /// Returns all {2} (`{3}`).
    pub fn {5}(&self) -> impl Iterator<Item=&str> {{
        self.string({6})
    }}

    /// Returns the first {1} (`{3}`).
    pub fn {4}(&self) -> Option<&str> {{
        self.string({6}).next()
    }}

    /// Consumes and returns all {2} (`{3}`).
    pub fn take_{5}(&mut self) -> impl Iterator<Item=String> + '_ {{
        self.take_string({6})
    }}

    /// Consumes all and returns the first {1} (`{3}`).
    pub fn take_{4}(&mut self) -> Option<String> {{
        self.take_string({6}).next()
    }}

    /// Sets the {1} (`{3}`). This will remove all other {2}.
    pub fn set_{4}(&mut self, {4}: impl Into<String>) {{
        self.set_data({6}, Data::Utf8({4}.into()));
    }}

    /// Adds an {1} (`{3}`).
    pub fn add_{4}(&mut self, {4}: impl Into<String>) {{
        self.add_data({6}, Data::Utf8({4}.into()));
    }}

    /// Removes all {2} (`{3}`).
    pub fn remove_{5}(&mut self) {{
        self.remove_data({6});
    }}

    /// Returns all {2} formatted in an easily readable way.
    fn format_{5}(&self) -> Option<String> {{
        if self.{5}().count() > 1 {{
            let mut string = String::from(\"{5}:\\n\");
            for v in self.{5}() {{
                string.push_str(\"    \");
                string.push_str(v);
                string.push('\\n');
            }}
            return Some(string);
        }}

        match self.{4}() {{
            Some(s) => Some(format!(\"{4}: {{}}\\n\", s)),
            None => None,
        }}
    }}
}}
    ",
        headline, name, name_plural, atom_ident_string, value_ident, value_ident_plural, atom_ident,
    )
    .parse()
    .expect("Error parsing accessor impl block:")
}

#[proc_macro]
pub fn flag_value_accessor(input: TokenStream) -> TokenStream {
    let (value_ident, name, headline, atom_ident, atom_ident_string) = base_values(input);

    format!(
        "
/// ### {0}
impl Tag {{
    /// Returns the {1} flag (`{2}`).
    pub fn {3}(&self) -> bool {{
        let vec = match self.data({4}).next() {{
            Some(Data::Reserved(v)) => v,
            Some(Data::BeSigned(v)) => v,
            _ => return false,
        }};

        vec.get(0).map(|&v| v == 1).unwrap_or(false)
    }}

    /// Sets the {1} flag to true (`{2}`).
    pub fn set_{3}(&mut self) {{
        self.set_data({4}, Data::BeSigned(vec![1u8]));
    }}

    /// Removes the {1} flag (`{2}`).
    pub fn remove_{3}(&mut self) {{
        self.remove_data({4})
    }}
}}
    ",
        headline, name, atom_ident_string, value_ident, atom_ident,
    )
    .parse()
    .expect("Error parsing accessor impl block:")
}

#[proc_macro]
pub fn integer_value_accessor(input: TokenStream) -> TokenStream {
    let (value_ident, name, headline, atom_ident, atom_ident_string) = base_values(input);

    format!(
        "
/// ### {0}
impl Tag {{
    /// Returns the {1} (`{2}`)
    pub fn {3}(&self) -> Option<u16> {{
        let vec = match self.data({4}).next()? {{
            Data::Reserved(v) => v,
            Data::BeSigned(v) => v,
            _ => return None,
        }};

        be_int!(vec, 0, u16)
    }}

    /// Sets the {1} (`{2}`)
    pub fn set_{3}(&mut self, {3}: u16) {{
        let vec: Vec<u8> = {3}.to_be_bytes().to_vec();
        self.set_data({4}, Data::BeSigned(vec));
    }}

    /// Removes the {1} (`{2}`).
    pub fn remove_{3}(&mut self) {{
        self.remove_data({4});
    }}
}}
    ",
        headline, name, atom_ident_string, value_ident, atom_ident,
    )
    .parse()
    .expect("Error parsing accessor impl block:")
}
