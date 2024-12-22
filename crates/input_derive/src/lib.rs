use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse_macro_input,
    token::{Bracket, Colon, Paren, Pound, Pub},
    AttrStyle, Attribute, Data, DeriveInput, Field, FieldMutability, Fields, Ident, MetaList, Path,
    PathArguments, PathSegment, Type, TypePath, Visibility,
};

// For the sanity of the reader!
// Know that this macro is quite bad :)
// Beware 💩💩💩
fn create_field(ident: Ident) -> Field {
    let bool = Ident::new("bool", Span::call_site());
    let serde = Ident::new("serde", Span::call_site());
    Field {
        attrs: vec![Attribute {
            pound_token: Pound::default(),
            style: AttrStyle::Outer,
            bracket_token: Bracket::default(),
            meta: syn::Meta::List(MetaList {
                path: Path {
                    leading_colon: None,
                    segments: [PathSegment {
                        ident: serde,
                        arguments: PathArguments::None,
                    }]
                    .into_iter()
                    .collect(),
                },
                delimiter: syn::MacroDelimiter::Paren(Paren::default()),
                tokens: quote!(default),
            }),
        }],
        vis: Visibility::Public(Pub {
            span: Span::call_site(),
        }),
        mutability: FieldMutability::None,
        ident: Some(ident),
        colon_token: Some(Colon {
            spans: [Span::call_site()],
        }),
        ty: Type::Path(TypePath {
            qself: None,
            path: Path {
                leading_colon: None,
                segments: [PathSegment {
                    ident: bool,
                    arguments: PathArguments::None,
                }]
                .into_iter()
                .collect(),
            },
        }),
    }
}

#[proc_macro_attribute]
pub fn derive_input(_attr: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let mut input = parse_macro_input!(input as DeriveInput);

    let name = input.ident.clone();
    let old_fields = match input.data.clone() {
        Data::Struct(ds) => ds
            .fields
            .into_iter()
            .filter_map(|f| f.ident)
            .collect::<Vec<_>>(),
        _ => panic!("this macro only supports structs!"),
    };

    let mut input_action = quote!();
    for field in &old_fields {
        let pressed = Ident::new(&format!("{field}_pressed"), Span::call_site());
        let just_pressed = Ident::new(&format!("{field}_just_pressed"), Span::call_site());
        let just_released = Ident::new(&format!("{field}_just_released"), Span::call_site());

        input_action = quote! {
            #input_action
            match input.#field {
                Key::Keyboard(key) => {
                    input.#pressed = keys.pressed(std::mem::transmute(key));
                    input.#just_pressed = keys.just_pressed(std::mem::transmute(key));
                    input.#just_released = keys.just_released(std::mem::transmute(key));
                }
                Key::Wheel(wheel) => {
                    if let Some(ev) = mouse_wheel {
                        input.#pressed = wheel.check(ev.y);
                    } else {
                        input.#pressed = false;
                    }

                    input.#just_pressed = false;
                    input.#just_released = false;
                }
                Key::Mouse(key) => {
                    input.#pressed = mouse_buttons.pressed(std::mem::transmute(key));
                    input.#just_pressed = mouse_buttons.just_pressed(std::mem::transmute(key));
                    input.#just_released = mouse_buttons.just_released(std::mem::transmute(key));
                }
            }
        };
    }

    let mut new_fields = Vec::new();
    for field in &old_fields {
        let pressed = Ident::new(&format!("{field}_pressed"), Span::call_site());
        let just_pressed = Ident::new(&format!("{field}_just_pressed"), Span::call_site());
        let just_released = Ident::new(&format!("{field}_just_released"), Span::call_site());
        new_fields.push(create_field(pressed));
        new_fields.push(create_field(just_pressed));
        new_fields.push(create_field(just_released));
    }

    if let Data::Struct(ds) = &mut input.data {
        if let Fields::Named(fs) = &mut ds.fields {
            for field in new_fields {
                fs.named.push(field);
            }
        }
    }

    let impl_block = quote! {
        impl #name {
            pub fn update(
                mut input: bevy::ecs::prelude::ResMut<PlayerInput>,
                mouse_buttons: bevy::ecs::prelude::Res<bevy::input::ButtonInput<bevy::input::mouse::MouseButton>>,
                mut mouse_wheel: bevy::ecs::prelude::EventReader<bevy::input::mouse::MouseWheel>,
                keys: bevy::ecs::prelude::Res<bevy::input::ButtonInput<bevy::input::keyboard::KeyCode>>,
            ) {
                let mouse_wheel = mouse_wheel.read().next();
                unsafe {
                    #input_action
                }
            }
        }
    };

    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        #input
        #impl_block
    };

    // Hand the output tokens back to the compiler
    TokenStream::from(expanded)
}
