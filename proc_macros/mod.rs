use proc_macro2::TokenStream;
use quote::{
    format_ident,
    quote,
    ToTokens,
};
use syn::{
    self,
    parse_macro_input,
    Ident,
    DeriveInput,
    Attribute,
    punctuated::Punctuated,
    Field,
};

fn parse_title(attrs: &Vec<Attribute>) -> Result<String, &'static str> {
    match attrs.iter().find_map(|a| a.parse_meta().ok()).and_then(|m| match m {
        syn::Meta::List(m) if m.path.to_token_stream().to_string() == "title" => {
            Some(m.nested)
        },
        _ => None,
    }) {
        Some(m) => {
            if m.len() != 1 {
                return Err("#[title()] needs exactly one literal string argument");
            }
            match m.first().unwrap() {
                syn::NestedMeta::Lit(l) => {
                    match l {
                        syn::Lit::Str(l) => {
                            return Ok(
                                litrs::StringLit::parse(l.to_token_stream().to_string()).unwrap().value().to_string(),
                            );
                        },
                        _ => return Err("#[title()] needs exactly one literal string argument"),
                    }
                },
                _ => {
                    return Err("#[title()] argument must be a string literal");
                },
            }
        },
        None => return Err("Missing #[title(\"Field name\")]"),
    };
}

fn build_fields_form<
    X,
>(
    form_ident: &Ident,
    value_type_ident: &Ident,
    value_construct_ident: &TokenStream,
    fields: &Punctuated<Field, X>,
) -> TokenStream {
    let mut form_fields = vec![];
    let mut form_construct_fields = vec![];
    let mut form_elements = vec![];
    let mut form_parse = vec![];
    let mut form_parse_assemble = vec![];
    for f in fields {
        let f_ident = f.ident.as_ref().unwrap();
        let f_name = parse_title(&f.attrs).expect(&format!("Error with attributes on field {}", f_ident));
        let f_type_ident = f.ty.to_token_stream();
        form_fields.push(quote!{
            #f_ident: Box < dyn rooting_forms:: FormState < #f_type_ident >>,
        });
        form_construct_fields.push(quote!{
            #f_ident: #f_type_ident:: new_form(#f_name),
        });
        form_elements.push(quote!{
            {
                let subelements = self.#f_ident.elements();
                elements.extend(subelements.error.into_iter());
                elements.push(rooting:: el("span").classes(&[rooting_forms::CSS_CLASS_LABEL]).text(#f_name));
                elements.extend(subelements.elements);
            }
        });
        form_parse.push(quote!{
            let #f_ident = match self.#f_ident.parse() {
                Ok(v) => Some(v),
                Err(e) => {
                    errored = true;
                    None
                }
            };
        });
        form_parse_assemble.push(quote!{
            #f_ident: #f_ident.unwrap(),
        });
    }
    return quote!{
        #[allow(non_camel_case_types)] struct #form_ident {
            #(#form_fields) *
        }
        impl rooting_forms:: FormState < #value_type_ident > for #form_ident {
            fn elements(&self) -> rooting_forms:: FormElements {
                let mut elements = Vec::new();
                #(#form_elements) * 
                //. .
                return rooting_forms:: FormElements {
                    error: None,
                    elements: elements
                };
            }
            fn parse(&self) -> Result < #value_type_ident,
            () > {
                let mut errored = false;
                #(#form_parse) * 
                //. .
                if errored {
                    return Err(());
                }
                return Ok(#value_construct_ident {
                    #(#form_parse_assemble) *
                });
            }
        }
        Box:: new(#form_ident {
            #(#form_construct_fields) *
        })
    };
}

fn derive1(body: DeriveInput) -> TokenStream {
    let t_ident = &body.ident;
    match body.data {
        syn::Data::Struct(s) => {
            match s.fields {
                syn::Fields::Named(fields) => {
                    let form_build =
                        build_fields_form(
                            &format_ident!("FormStateImpl"),
                            &t_ident,
                            &t_ident.to_token_stream(),
                            &fields.named,
                        );
                    return quote!{
                        impl rooting_forms:: Form for #t_ident {
                            fn new_form(field: &str) -> Box < dyn rooting_forms:: FormState < Self >> {
                                use rooting_forms::FormState;
                                use std::str::FromStr;
                                use wasm_bindgen::JsCast;
                                #form_build
                            }
                        }
                    };
                },
                syn::Fields::Unnamed(_) => panic!("Tuple structs aren't supported"),
                syn::Fields::Unit => panic!("Unit structs aren't supported"),
            }
        },
        syn::Data::Enum(e) => {
            let mut build_variants = vec![];
            for (i, v) in e.variants.iter().enumerate() {
                let v_ident = &v.ident;
                let v_name =
                    parse_title(&v.attrs).expect(&format!("Error with attributes on {}::{}", t_ident, v_ident));
                let v_value = format!("{}", i);
                let build_option;
                {
                    let selected;
                    if i == 0 {
                        selected = quote!(.attr("selected", "selected"));
                    } else {
                        selected = quote!();
                    }
                    build_option = quote!{
                        select.ref_push(rooting:: el("option").text(#v_name).attr("value", #v_value) #selected);
                    };
                }
                let container = quote!(rooting::el("div").classes(&[rooting_forms::CSS_CLASS_SUBFORM]));
                match &v.fields {
                    syn::Fields::Named(fields) => {
                        let subform_build =
                            build_fields_form(
                                &format_ident!("{}_{}_FormState", t_ident, v.ident),
                                &t_ident,
                                &quote!(#t_ident:: #v_ident),
                                &fields.named,
                            );
                        build_variants.push(quote!{
                            {
                                #build_option 
                                //. .
                                let subform = {
                                    #subform_build
                                };
                                let subform_elements = subform.elements();
                                let container = #container;
                                if let Some(error) = subform_elements.error {
                                    container.ref_push(error);
                                }
                                container.ref_extend(subform_elements.elements);
                                variant_elements.push(container);
                                variant_parse.push(Box::new(move || subform.parse()));
                            }
                        });
                    },
                    syn::Fields::Unnamed(fields) => {
                        if fields.unnamed.len() != 1 {
                            panic!(
                                "Only single field tuple enum variants are supported currently ({}::{})",
                                t_ident,
                                v_ident
                            );
                        }
                        let f = fields.unnamed.first().unwrap();
                        let f_type_ident = f.ty.to_token_stream();
                        build_variants.push(quote!{
                            {
                                #build_option 
                                //. .
                                let subform = #f_type_ident:: new_form(#v_name);
                                let subform_elements = subform.elements();
                                let container = #container;
                                if let Some(error) = subform_elements.error {
                                    container.ref_push(error);
                                }
                                container.ref_extend(subform_elements.elements);
                                variant_elements.push(container);
                                variant_parse.push(
                                    Box:: new(move || subform.parse().map(| v | #t_ident:: #v_ident(v)))
                                );
                            }
                        });
                    },
                    syn::Fields::Unit => {
                        build_variants.push(quote!{
                            {
                                #build_option 
                                //. .
                                variant_parse.push(Box:: new(|| Ok(#t_ident:: #v_ident)));
                                variant_elements.push(#container);
                            }
                        });
                    },
                }
            }
            return quote!{
                impl rooting_forms:: Form for #t_ident {
                    fn new_form(field: &str) -> Box < dyn rooting_forms:: FormState < Self >> {
                        use rooting_forms::FormState;
                        use std::str::FromStr;
                        use wasm_bindgen::JsCast;
                        struct FormStateImpl {
                            select: rooting::El,
                            variant_parse: Vec < Box < dyn Fn() -> Result < #t_ident,
                            () >>>,
                            variant_elements: Vec<rooting::El>,
                            current_variant: std::rc::Rc<std::cell::Cell<usize>>,
                        }
                        impl rooting_forms:: FormState < #t_ident > for FormStateImpl {
                            fn elements(&self) -> rooting_forms::FormElements {
                                let mut out = vec![];
                                out.push(self.select.clone());
                                out.extend(self.variant_elements.clone());
                                return rooting_forms::FormElements {
                                    error: None,
                                    elements: out,
                                };
                            }
                            fn parse(&self) -> Result < #t_ident,
                            () > {
                                return self.variant_parse[self.current_variant.get()]();
                            }
                        }
                        let variant = std::rc::Rc::new(std::cell::Cell::new(0));
                        let mut elements = vec![];
                        let select =
                            rooting::el("select")
                                .classes(&[rooting_forms::CSS_CLASS_SMALL_INPUT])
                                .attr(rooting_forms::ATTR_LABEL, field);
                        elements.push(select.clone());
                        let mut variant_parse: Vec < Box < dyn Fn() -> Result < #t_ident,
                        () >>>
                        //. 
                        = vec ![];
                        let mut variant_elements = vec![];
                        #(#build_variants) * 
                        //. .
                        select.ref_on("change", {
                            let variant_elements = variant_elements.clone();
                            let variant = variant.clone();
                            move |event| {
                                let index =
                                    usize::from_str(
                                        &event
                                            .target()
                                            .unwrap()
                                            .dyn_into::<rooting_forms::republish::HtmlSelectElement>()
                                            .unwrap()
                                            .value(),
                                    ).unwrap();
                                variant.set(index);
                                for (e_index, v) in variant_elements.iter().enumerate() {
                                    v.ref_modify_classes(&[(rooting_forms::CSS_CLASS_HIDDEN, e_index != index)]);
                                }
                            }
                        });
                        for v in &variant_elements[1..] {
                            v.ref_classes(&[rooting_forms::CSS_CLASS_HIDDEN]);
                        }
                        return Box::new(FormStateImpl {
                            select: select,
                            variant_parse: variant_parse,
                            variant_elements: variant_elements,
                            current_variant: variant,
                        });
                    }
                }
            }
        },
        syn::Data::Union(_) => panic!("Union types unsupported"),
    };
}

#[proc_macro_derive(Form, attributes(title))]
pub fn derive(body: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(body as syn::DeriveInput);
    return derive1(ast).into();
}

#[cfg(test)]
mod tests {
    use std::{
        str::FromStr,
    };
    use genemichaels::FormatConfig;
    use proc_macro2::TokenStream;
    use crate::derive1;
    use quote::quote;
    use similar::{
        TextDiff,
        ChangeTag,
    };

    fn comp(got: &'static str, expected: TokenStream) {
        let got = derive1(syn::parse2(TokenStream::from_str(got).unwrap()).unwrap());
        let cfg = FormatConfig::default();
        let mut s =
            [&got, &expected]
                .into_iter()
                .map(|s| genemichaels::format_str(&s.to_string(), &cfg))
                .collect::<Vec<_>>();
        let got = s.remove(0).expect(&format!("Failed to format got code:\n{}", got.to_string())).rendered;
        let expected = s.remove(0).expect("Failed to format expected code").rendered;
        let changes = TextDiff::from_lines(&got, &expected);
        let mut has_changes = false;
        let mut text = String::new();
        for change in changes.iter_all_changes() {
            let sign = match change.tag() {
                ChangeTag::Delete => {
                    has_changes = true;
                    "-"
                },
                ChangeTag::Insert => {
                    has_changes = true;
                    "+"
                },
                ChangeTag::Equal => " ",
            };
            text.push_str(&format!("{}{}", sign, change));
        }
        assert!(!has_changes, "{}", text);
    }

    #[test]
    fn simple_struct() {
        comp(r#"
struct Alpha {
    #[title("A")]
    a: i32,
}
"#, quote!(
            impl rooting_forms::Form for Alpha {
                fn new_form(field: &str) -> Box<dyn rooting_forms::FormState<Self>> {
                    use rooting_forms::FormState;
                    use std::str::FromStr;
                    use wasm_bindgen::JsCast;

                    struct FormStateImpl {
                        a: Box<dyn rooting_forms::FormState<i32>>,
                    }

                    impl rooting_forms::FormState<Alpha> for FormStateImpl {
                        fn elements(&self) -> rooting_forms::FormElements {
                            let mut elements = Vec::new();
                            {
                                let subelements = self.a.elements();
                                elements.extend(subelements.error.into_iter());
                                elements.push(
                                    rooting::el("span").classes(&[rooting_forms::CSS_CLASS_LABEL]).text("A"),
                                );
                                elements.extend(subelements.elements);
                            }
                            return rooting_forms::FormElements {
                                error: None,
                                elements: elements,
                            };
                        }

                        fn parse(&self) -> Result<Alpha, ()> {
                            let mut errored = false;
                            let a = match self.a.parse() {
                                Ok(v) => Some(v),
                                Err(e) => {
                                    errored = true;
                                    None
                                },
                            };
                            if errored {
                                return Err(());
                            }
                            return Alpha { a: a.unwrap() };
                        }
                    }

                    Box::new(FormStateImpl { a: i32::new_form("A") })
                }
            }
        ));
    }

    #[test]
    fn simple_enum() {
        comp(
            r#"
enum Alpha {
    #[title("A")]
    A,
    #[title("B")]
    B(i32),
    #[title("C")]
    C {
        #[title("C")]
        c: i32,
    },
}
"#,
            quote!{
                impl rooting_forms::Form for Alpha {
                    fn new_form(field: &str) -> Box<dyn rooting_forms::FormState<Self>> {
                        use rooting_forms::FormState;
                        use std::str::FromStr;
                        use wasm_bindgen::JsCast;

                        struct FormStateImpl {
                            select: rooting::El,
                            variant_parse: Vec<Box<dyn Fn() -> Result<Alpha, ()>>>,
                            variant_elements: Vec<rooting::El>,
                            current_variant: std::rc::Rc<std::cell::Cell<usize>>,
                        }

                        impl rooting_forms::FormState<Alpha> for FormStateImpl {
                            fn elements(&self) -> rooting_forms::FormElements {
                                let mut out = vec![];
                                out.push(self.select.clone());
                                out.extend(self.variant_elements.clone());
                                return rooting_forms::FormElements {
                                    error: None,
                                    elements: out,
                                };
                            }

                            fn parse(&self) -> Result<Alpha, ()> {
                                return self.variant_parse[self.current_variant.get()]();
                            }
                        }

                        let variant = std::rc::Rc::new(std::cell::Cell::new(0));
                        let mut elements = vec![];
                        let select =
                            rooting::el("select")
                                .classes(&[rooting_forms::CSS_CLASS_SMALL_INPUT])
                                .attr(rooting_forms::ATTR_LABEL, field);
                        elements.push(select.clone());
                        let mut variant_parse: Vec<Box<dyn Fn() -> Result<Alpha, ()>>> = vec![];
                        let mut variant_elements = vec![];
                        {
                            select.ref_push(
                                rooting::el("option").text("A").attr("value", "0").attr("selected", "selected"),
                            );
                            variant_parse.push(Box::new(|| Ok(Alpha::A)));
                            variant_elements.push(rooting::el("div").classes(&[rooting_forms::CSS_CLASS_SUBFORM]));
                        }
                        {
                            select.ref_push(rooting::el("option").text("B").attr("value", "1"));
                            let subform = i32::new_form("B");
                            let subform_elements = subform.elements();
                            let container = rooting::el("div").classes(&[rooting_forms::CSS_CLASS_SUBFORM]);
                            if let Some(error) = subform_elements.error {
                                container.ref_push(error);
                            }
                            container.ref_extend(subform_elements.elements);
                            variant_elements.push(container);
                            variant_parse.push(Box::new(move || subform.parse()));
                        }
                        {
                            select.ref_push(rooting::el("option").text("C").attr("value", "2"));
                            let subform = {
                                #[allow(non_camel_case_types)]
                                struct Alpha_C_FormState {
                                    c: Box<dyn rooting_forms::FormState<i32>>,
                                }

                                impl rooting_forms::FormState<Alpha> for Alpha_C_FormState {
                                    fn elements(&self) -> rooting_forms::FormElements {
                                        let mut elements = Vec::new();
                                        {
                                            let subelements = self.c.elements();
                                            elements.extend(subelements.error.into_iter());
                                            elements.push(
                                                rooting::el("span")
                                                    .classes(&[rooting_forms::CSS_CLASS_LABEL])
                                                    .text("C"),
                                            );
                                            elements.extend(subelements.elements);
                                        }
                                        return rooting_forms::FormElements {
                                            error: None,
                                            elements: elements,
                                        };
                                    }

                                    fn parse(&self) -> Result<Alpha, ()> {
                                        let mut errored = false;
                                        let c = match self.c.parse() {
                                            Ok(v) => Some(v),
                                            Err(e) => {
                                                errored = true;
                                                None
                                            },
                                        };
                                        if errored {
                                            return Err(());
                                        }
                                        return Ok(Alpha::C { c: c.unwrap() });
                                    }
                                }

                                Box::new(FormStateImpl { c: i32::new_form("C") })
                            };
                            let subform_elements = subform.elements();
                            let container = rooting::el("div").classes(&[rooting_forms::CSS_CLASS_SUBFORM]);
                            if let Some(error) = subform_elements.error {
                                container.ref_push(error);
                            }
                            container.ref_extend(subform_elements.elements);
                            variant_elements.push(container);
                            variant_parse.push(Box::new(move || subform.parse()));
                        }
                        select.ref_on("change", {
                            let variant_elements = variant_elements.clone();
                            let variant = variant.clone();
                            move |event| {
                                let index =
                                    usize::from_str(
                                        &event
                                            .target()
                                            .unwrap()
                                            .dyn_into::<rooting_forms::republish::HtmlSelectElement>()
                                            .unwrap()
                                            .value(),
                                    ).unwrap();
                                variant.set(index);
                                for (e_index, v) in variant_elements.iter().enumerate() {
                                    v.ref_modify_classes(&[(rooting_forms::CSS_CLASS_HIDDEN, e_index != index)]);
                                }
                            }
                        });
                        for v in &variant_elements[1..] {
                            v.ref_classes(&[rooting_forms::CSS_CLASS_HIDDEN]);
                        }
                        return Box::new(FormStateImpl {
                            select: select,
                            variant_parse: variant_parse,
                            variant_elements: variant_elements,
                            current_variant: variant,
                        });
                    }
                }
            },
        );
    }
}
