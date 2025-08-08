use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Expr, Fields, Type, parse::Parse, parse_macro_input};

#[proc_macro_derive(WowDataRV, attributes(wow_data))]
pub fn wow_data_rv_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let version_ty = match get_value_from_attrs::<Type>(&input.attrs, "wow_data", "version") {
        Ok(ty) => ty,
        Err(e) => return e.to_compile_error().into(),
    };

    let struct_name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let reader_body = match &input.data {
        Data::Struct(s) => generate_struct_reader_body(&s.fields),
        Data::Enum(e) => generate_enum_reader_body(e),
        Data::Union(_) => {
            return syn::Error::new_spanned(&input, "WowDataRV cannot be derived for unions.")
                .to_compile_error()
                .into();
        }
    };

    let reader_body = match reader_body {
        Ok(body) => body,
        Err(e) => return e.to_compile_error().into(),
    };

    let expanded = quote! {
        impl #impl_generics WowDataRV<#version_ty> for #struct_name #ty_generics #where_clause {
            fn wow_read<R: ::std::io::Read + ::std::io::Seek>(reader: &mut R, version: #version_ty) -> Result<Self> {
                Ok(#reader_body)
            }
        }
    };

    TokenStream::from(expanded)
}

fn generate_struct_reader_body(fields: &Fields) -> syn::Result<proc_macro2::TokenStream> {
    let named_fields = if let Fields::Named(f) = fields {
        &f.named
    } else {
        return Err(syn::Error::new_spanned(
            fields,
            "WowDataRV on structs only supports named fields.",
        ));
    };

    let initializers = named_fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();

        // Check for the `#[wow_data(versioned)]` attribute on the field.
        let is_versioned = field.attrs.iter().any(|attr| {
            if !attr.path().is_ident("wow_data") {
                return false;
            }
            // Check for `(versioned)`
            let mut found = false;
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("versioned") {
                    found = true;
                }
                Ok(()) // Ignore other attributes
            });
            found
        });

        if is_versioned {
            quote! { #field_name: reader.wow_read_versioned(version)? }
        } else {
            match get_value_from_attrs::<Expr>(&field.attrs, "wow_data", "skip") {
                Ok(val) => quote! { #field_name: #val },
                Err(_) => quote! { #field_name: reader.wow_read()? },
            }
        }
    });

    Ok(quote! {
        Self {
            #(#initializers),*
        }
    })
}

fn generate_enum_reader_body(data: &syn::DataEnum) -> syn::Result<proc_macro2::TokenStream> {
    let mut conditional_arms = Vec::new();
    let mut default_arm = None;

    for variant in &data.variants {
        let variant_ident = &variant.ident;

        let constructor = match &variant.fields {
            syn::Fields::Unit => {
                quote! { Self::#variant_ident }
            }
            syn::Fields::Unnamed(fields) => {
                let read_fields = fields.unnamed.iter().map(|_| {
                    quote! { reader.wow_read()? }
                });
                quote! { Self::#variant_ident(#(#read_fields),*) }
            }
            syn::Fields::Named(fields) => {
                let field_bindings = fields.named.iter().map(|f| f.ident.as_ref().unwrap());
                let read_fields =
                    field_bindings.map(|binding| quote! { #binding: reader.wow_read()? });
                quote! { Self::#variant_ident{#(#read_fields),*} }
            }
        };

        match get_value_from_attrs::<Expr>(&variant.attrs, "wow_data", "read_if") {
            Ok(cond_expr) => {
                conditional_arms.push(quote! { if #cond_expr { #constructor } });
            }
            Err(_) => {
                if default_arm.is_some() {
                    return Err(syn::Error::new_spanned(
                        variant,
                        "Only one enum variant can be the default (lacking a `read_if` attribute).",
                    ));
                }
                default_arm = Some(constructor);
            }
        }
    }

    let default_arm = default_arm.ok_or_else(|| {
        syn::Error::new_spanned(
            &data.variants,
            "An enum must have one default variant (lacking a `read_if` attribute).",
        )
    })?;

    let full_body = quote! {
        #(#conditional_arms)else*
        else {
            #default_arm
        }
    };

    Ok(full_body)
}

fn get_value_from_attrs<T: Parse>(
    attrs: &[syn::Attribute],
    attr_name: &str,
    attr_key: &str,
) -> syn::Result<T> {
    let mut ret_val = None;

    for attr in attrs {
        if !attr.path().is_ident(attr_name) {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident(attr_key) {
                let value = meta.value()?;
                let parsed: T = value.parse()?;
                ret_val = Some(parsed);
            }
            Ok(())
        })?;
    }

    ret_val.ok_or_else(|| {
        syn::Error::new(
            proc_macro2::Span::call_site(),
            "Missing required struct attribute `#[wow_data(version = YourVersionType)]`",
        )
    })
}

#[proc_macro_derive(WowDataR, attributes(wow_data))]
pub fn wow_data_r_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;

    let fields = if let Data::Struct(s) = &input.data {
        if let Fields::Named(f) = &s.fields {
            &f.named
        } else {
            return syn::Error::new_spanned(
                &s.fields,
                "WowDataR can only be derived for structs with named fields.",
            )
            .to_compile_error()
            .into();
        }
    } else {
        return syn::Error::new_spanned(&input, "WowDataR can only be derived for structs.")
            .to_compile_error()
            .into();
    };

    let initializers = fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();

        match get_value_from_attrs::<Expr>(&field.attrs, "wow_data", "skip") {
            Ok(val) => quote! { #field_name: #val },
            Err(_) => quote! { #field_name: reader.wow_read()? },
        }
    });

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics WowDataR for #struct_name #ty_generics #where_clause {
            fn wow_read<R: Read + Seek>(reader: &mut R) -> Result<Self> {
                Ok(Self{
                    #(#initializers),*
                })
            }
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_derive(WowDataW, attributes(wow_data))]
pub fn wow_data_w_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let (writer_body, sizer_body) = match &input.data {
        Data::Struct(s) => (
            generate_struct_writer_body(&s.fields),
            generate_struct_size_body(&s.fields),
        ),
        Data::Enum(e) => (generate_enum_writer_body(e), generate_enum_size_body(e)),
        Data::Union(_) => {
            return syn::Error::new_spanned(&input, "WowDataW cannot be derived for unions.")
                .to_compile_error()
                .into();
        }
    };

    // Check if body generation failed
    let writer_body = match writer_body {
        Ok(body) => body,
        Err(e) => return e.to_compile_error().into(),
    };
    let sizer_body = match sizer_body {
        Ok(body) => body,
        Err(e) => return e.to_compile_error().into(),
    };

    let expanded = quote! {
        impl #impl_generics WowDataW for #ident #ty_generics #where_clause {
            fn wow_write<W: ::std::io::Write>(&self, writer: &mut W) -> Result<()> {
                #writer_body
            }

            fn wow_size(&self) -> usize {
                #sizer_body
            }
        }
    };

    TokenStream::from(expanded)
}

fn generate_struct_writer_body(fields: &Fields) -> syn::Result<proc_macro2::TokenStream> {
    let named_fields = if let Fields::Named(f) = fields {
        &f.named
    } else {
        return Err(syn::Error::new_spanned(
            fields,
            "WowDataW on structs only supports named fields.",
        ));
    };
    let write_statements = named_fields.iter().map(|field| {
        match get_value_from_attrs::<Expr>(&field.attrs, "wow_data", "skip") {
            Ok(_) => quote! {},
            Err(_) => {
                let field_name = field.ident.as_ref().unwrap();
                quote! {
                    writer.wow_write(&self.#field_name)?;
                }
            }
        }
    });

    Ok(quote! {
        #(#write_statements)*
        Ok(())
    })
}

fn generate_struct_size_body(fields: &Fields) -> syn::Result<proc_macro2::TokenStream> {
    let named_fields = if let Fields::Named(f) = fields {
        &f.named
    } else {
        return Err(syn::Error::new_spanned(
            fields,
            "WowDataW on structs only supports named fields.",
        ));
    };
    let size_expressions = named_fields.iter().map(|field| {
        match get_value_from_attrs::<Expr>(&field.attrs, "wow_data", "skip") {
            Ok(_) => quote! {0},
            Err(_) => {
                let field_name = field.ident.as_ref().unwrap();
                quote! {
                    self.#field_name.wow_size()
                }
            }
        }
    });

    Ok(quote! {
        0 #(+ #size_expressions)*
    })
}

fn generate_enum_writer_body(data: &syn::DataEnum) -> syn::Result<proc_macro2::TokenStream> {
    let arms = data.variants.iter().map(|variant| {
        let variant_ident = &variant.ident;

        match &variant.fields {
            syn::Fields::Unit => {
                quote! {
                    Self::#variant_ident => {}
                }
            }

            syn::Fields::Unnamed(fields) => {
                let field_bindings = fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(i, _)| format_ident!("v{}", i));
                let field_bindings_clone = field_bindings.clone();

                let write_calls =
                    field_bindings.map(|binding| quote! { writer.wow_write(#binding)?; });

                quote! {
                    Self::#variant_ident(#(#field_bindings_clone),*) => {
                        #(#write_calls)*
                    }
                }
            }

            syn::Fields::Named(fields) => {
                let field_bindings = fields.named.iter().map(|f| f.ident.as_ref().unwrap());
                let field_bindings_clone = field_bindings.clone();

                let write_calls =
                    field_bindings.map(|binding| quote! { writer.wow_write(#binding)?; });

                quote! {
                    Self::#variant_ident { #(#field_bindings_clone),* } => {
                         #(#write_calls)*
                    }
                }
            }
        }
    });

    Ok(quote! {
        match self {
            #(#arms),*
        }
        Ok(())
    })
}

fn generate_enum_size_body(data: &syn::DataEnum) -> syn::Result<proc_macro2::TokenStream> {
    let arms = data.variants.iter().map(|variant| {
        let variant_ident = &variant.ident;

        match &variant.fields {
            syn::Fields::Unit => {
                quote! {
                    Self::#variant_ident => 0
                }
            }
            syn::Fields::Unnamed(fields) => {
                let field_bindings = fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(i, _)| format_ident!("v{}", i));
                let field_bindings_clone = field_bindings.clone();

                let size_calls = field_bindings.map(|binding| quote! { #binding.wow_size() });

                quote! {
                    Self::#variant_ident(#(#field_bindings_clone),*) => {
                        0 #(+ #size_calls)*
                    }
                }
            }
            syn::Fields::Named(fields) => {
                let field_bindings = fields.named.iter().map(|f| f.ident.as_ref().unwrap());
                let field_bindings_clone = field_bindings.clone();

                let size_calls = field_bindings.map(|binding| quote! { #binding.wow_size() });

                quote! {
                    Self::#variant_ident { #(#field_bindings_clone),* } => {
                        0 #(+ #size_calls)*
                    }
                }
            }
        }
    });

    Ok(quote! {
        match self {
            #(#arms),*
        }
    })
}
