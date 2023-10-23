This provides traits and a derive macro for generating rooting HTML forms from structs and enums.

**Example**

```
#[derive(rooting_form::Form)]
struct Creds {
    #[title("Your username")]
    username: String,
    #[title("Your password")]
    password: rooting_form::Password,
}

let creds_form_state = MyStruct::new_form();
modal.ref_push(el("div").extend(creds_form_state.elements().elements));
ok_button.ref_on("click", move |_| {
    let Some(creds) = creds_form_state.parse() else {
        return;
    };
    do_login(creds);
});
```

# Parsing

`parse()` returns `Some(..)` if there were no validation issues, otherwise `None`. Validation issues will automatically be displayed, and cleared the next time this is called.

# Styling

`elements` above will be a list of (by CSS selector):

- `.form_label` - input labels
- `.form_input_small` - single column inputs like single line entry, checkboxes, dropdowns
- `.form_input_big` - multi column inputs like textareas
- `.form_input_option` - a special case, the checkbox for optional elements
- `.form_error` - an element containing validation error text. This is always visible, but the text may be empty
- `.subform` - for nested forms, namely within variants
- `.disable_hide` - for inactive form elements (ex: controls for a variant that's not selected)

I imagine you'll place these in a grid, with the labels in column 1, option checkboxes in column 2, small inputs in column 3, and big inputs/subforms spanning all columns.

# Future

- A method for turning existing data into a form, rather than only new data
