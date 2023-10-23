use std::{
    fmt::Display,
    str::FromStr,
    convert::Infallible,
};
use rooting::{
    El,
    el,
};
pub use rooting_forms_proc_macros::Form;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;

/// Republished types for macro use.
pub mod republish {
    pub use web_sys::HtmlSelectElement;
}

/// Used for the text label before form fields.
pub const CSS_CLASS_LABEL: &'static str = "form_label";

/// Used for single-column inputs (text entry, checkbox). Exclusive with other
/// `form_input_` classes.
pub const CSS_CLASS_SMALL_INPUT: &'static str = "form_input_small";

/// Used for two-column inputs like text area. Exclusive with other `form_input_`
/// classes.
pub const CSS_CLASS_BIG_INPUT: &'static str = "form_input_big";

/// Used by the checkbox for options. Exclusive with other `form_input_` classes.
pub const CSS_CLASS_OPTION_ENABLE: &'static str = "form_input_option";

/// Used for validation errors, appears before the input (also before the
/// associated label, if there is one).
pub const CSS_CLASS_ERROR: &'static str = "form_error";

/// Used for nested struct/enum fields, namely within variants or options.
pub const CSS_CLASS_SUBFORM: &'static str = "subform";

/// Used to hide disabled variants - hidden to keep user input in case they
/// re-enable later.
pub const CSS_CLASS_HIDDEN: &'static str = "disable_hide";

/// This should be used on all inputs, since `<label>` isn't used.
pub const ATTR_LABEL: &'static str = "aria-label";

pub struct FormElements {
    /// The error display element, with `CSS_CLASS_ERROR`. This may be placed before
    /// the label in a struct context.
    pub error: Option<El>,
    /// The input, and any additional controls.
    pub elements: Vec<El>,
}

/// An object representing a form (the state of the form).
pub trait FormState<T> {
    /// Get the elements for this form or subform.
    fn elements(&self) -> FormElements;

    /// Parse the elements into the resulting type.
    fn parse(&self) -> Result<T, ()>;
}

/// This represnts a rust datatype that can be included in a form.
pub trait Form {
    /// Generates a form for a new value (no existing value).
    ///
    /// * `field` - is the field name, for accessibility using `aria-label`. `<label>`
    ///   isn't used sometime due to anonymous fields in tuples.
    fn new_form(field: &'static str) -> Box<dyn FormState<Self>>;
}

/// A minimal string wrapper that creates a password form input.
pub struct Password(String);

impl FromStr for Password {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        return Ok(Password(s.to_string()));
    }
}

/// A minimal string wrapper that creates a textarea form input.
pub struct BigString(String);

impl FromStr for BigString {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        return Ok(BigString(s.to_string()));
    }
}

/// A helper form type for rust types that implement `FromStr`.
pub struct FromStrFormState {
    el: El,
    error_el: El,
}

impl FromStrFormState {
    pub fn new<E: Display, T: FromStr<Err = E>>(label: &str, type_: &str) -> Box<dyn FormState<T>> {
        let error_el = el("span").classes(&[CSS_CLASS_ERROR]);
        return Box::new(FromStrFormState {
            el: el("input")
                .classes(&[CSS_CLASS_SMALL_INPUT])
                .attr(ATTR_LABEL, label)
                .attr("type", type_)
                .on("change", {
                    let error_el = error_el.clone();
                    move |ev| {
                        let text = ev.target().unwrap().dyn_ref::<HtmlInputElement>().unwrap().value();
                        if text.len() >= 1 {
                            match T::from_str(&text) {
                                Err(e) => {
                                    error_el.ref_text(&e.to_string());
                                    return;
                                },
                                _ => { },
                            }
                        }
                        error_el.ref_text("");
                    }
                }),
            error_el: error_el,
        });
    }
}

impl<E: Display, T: FromStr<Err = E>> FormState<T> for FromStrFormState {
    fn elements(&self) -> FormElements {
        return FormElements {
            error: Some(self.error_el.clone()),
            elements: vec![self.el.clone()],
        };
    }

    fn parse(&self) -> Result<T, ()> {
        match T::from_str(&self.el.raw().dyn_ref::<HtmlInputElement>().unwrap().value()) {
            Ok(v) => {
                self.error_el.ref_text("");
                return Ok(v);
            },
            Err(e) => {
                self.error_el.ref_text(&e.to_string());
                return Err(());
            },
        }
    }
}

impl Form for String {
    fn new_form(field: &'static str) -> Box<dyn FormState<Self>> {
        return FromStrFormState::new::<_, String>(field, "text");
    }
}

impl Form for Password {
    fn new_form(field: &'static str) -> Box<dyn FormState<Self>> {
        return FromStrFormState::new::<_, Password>(field, "password");
    }
}

impl Form for BigString {
    fn new_form(field: &'static str) -> Box<dyn FormState<Self>> {
        return FromStrFormState::new::<_, BigString>(field, "text");
    }
}

impl Form for u8 {
    fn new_form(field: &'static str) -> Box<dyn FormState<Self>> {
        return FromStrFormState::new::<_, Self>(field, "text");
    }
}

impl Form for u16 {
    fn new_form(field: &'static str) -> Box<dyn FormState<Self>> {
        return FromStrFormState::new::<_, Self>(field, "text");
    }
}

impl Form for u32 {
    fn new_form(field: &'static str) -> Box<dyn FormState<Self>> {
        return FromStrFormState::new::<_, Self>(field, "text");
    }
}

impl Form for u64 {
    fn new_form(field: &'static str) -> Box<dyn FormState<Self>> {
        return FromStrFormState::new::<_, Self>(field, "text");
    }
}

impl Form for i8 {
    fn new_form(field: &'static str) -> Box<dyn FormState<Self>> {
        return FromStrFormState::new::<_, Self>(field, "text");
    }
}

impl Form for i16 {
    fn new_form(field: &'static str) -> Box<dyn FormState<Self>> {
        return FromStrFormState::new::<_, Self>(field, "text");
    }
}

impl Form for i32 {
    fn new_form(field: &'static str) -> Box<dyn FormState<Self>> {
        return FromStrFormState::new::<_, Self>(field, "text");
    }
}

impl Form for i64 {
    fn new_form(field: &'static str) -> Box<dyn FormState<Self>> {
        return FromStrFormState::new::<_, Self>(field, "text");
    }
}

impl Form for f32 {
    fn new_form(field: &'static str) -> Box<dyn FormState<Self>> {
        return FromStrFormState::new::<_, Self>(field, "text");
    }
}

impl Form for f64 {
    fn new_form(field: &'static str) -> Box<dyn FormState<Self>> {
        return FromStrFormState::new::<_, Self>(field, "text");
    }
}

struct BoolFormState {
    input: El,
}

impl FormState<bool> for BoolFormState {
    fn elements(&self) -> FormElements {
        return FormElements {
            error: None,
            elements: vec![self.input.clone()],
        };
    }

    fn parse(&self) -> Result<bool, ()> {
        return Ok(self.input.raw().dyn_ref::<HtmlInputElement>().unwrap().checked());
    }
}

impl Form for bool {
    fn new_form(field: &'static str) -> Box<dyn FormState<Self>> {
        return Box::new(
            BoolFormState {
                input: el("input")
                    .classes(&[CSS_CLASS_SMALL_INPUT])
                    .attr(ATTR_LABEL, field)
                    .attr("type", "checkbox"),
            },
        );
    }
}

struct OptionFormState<T> {
    elements: Vec<El>,
    subform: Box<dyn FormState<T>>,
}

impl<T: Form> FormState<Option<T>> for OptionFormState<T> {
    fn elements(&self) -> FormElements {
        return FormElements {
            error: None,
            elements: self.elements.clone(),
        };
    }

    fn parse(&self) -> Result<Option<T>, ()> {
        let checked = self.elements[0].raw().dyn_ref::<HtmlInputElement>().unwrap().checked();
        if checked {
            return Ok(Some(self.subform.parse()?));
        } else {
            return Ok(None);
        }
    }
}

impl<T: Form + 'static> Form for Option<T> {
    fn new_form(field: &'static str) -> Box<dyn FormState<Self>> {
        let subform = T::new_form(field);
        let subform_elements = subform.elements();
        let mut additional = vec![];
        additional.extend(subform_elements.error.iter().cloned());
        additional.extend(subform_elements.elements);
        for e in &additional {
            e.ref_modify_classes(&[(CSS_CLASS_HIDDEN, true)]);
        }
        let mut elements = vec![
            //. .
            el("input")
                .classes(&[CSS_CLASS_OPTION_ENABLE])
                .attr(ATTR_LABEL, &format!("{} - Enabled", field))
                .attr("type", "checkbox")
                .on("click", {
                    let additional = additional.clone();
                    move |ev| {
                        let checked = ev.target().unwrap().dyn_ref::<HtmlInputElement>().unwrap().checked();
                        for e in &additional {
                            e.ref_modify_classes(&[(CSS_CLASS_HIDDEN, checked)]);
                        }
                    }
                })
        ];
        elements.extend(additional);
        return Box::new(OptionFormState {
            elements: elements,
            subform: subform,
        });
    }
}
