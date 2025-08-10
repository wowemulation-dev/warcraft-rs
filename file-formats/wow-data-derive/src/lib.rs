use std::{collections::HashMap, fmt};

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Expr, Fields, LitBool, Type, parse::Parse, parse_macro_input};

#[proc_macro_derive(WowHeaderR, attributes(wow_data))]
pub fn wow_header_r_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let version_ty = match get_values_from_attrs::<Type>(&input.attrs, "wow_data", &["version"]) {
        Ok(mut map) => map.remove("version"),
        Err(err) => return err.to_compile_error().into(),
    };

    let struct_name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = if let Some(version_ty) = version_ty {
        let reader_body = match &input.data {
            Data::Struct(s) => generate_header_rv_struct_reader_body(&s.fields),
            Data::Enum(e) => generate_header_rv_enum_reader_body(e),
            Data::Union(_) => {
                return syn::Error::new_spanned(
                    &input,
                    "VWowHeaderR cannot be derived for unions.",
                )
                .to_compile_error()
                .into();
            }
        };

        let reader_body = match reader_body {
            Ok(body) => body,
            Err(e) => return e.to_compile_error().into(),
        };

        quote! {
            impl #impl_generics wow_data::types::VWowHeaderR<#version_ty> for #struct_name #ty_generics #where_clause {
                fn wow_read<R: ::std::io::Read + ::std::io::Seek>(reader: &mut R, version: #version_ty) -> wow_data::error::Result<Self> {
                    Ok(#reader_body)
                }
            }
        }
    } else {
        let reader_body = if let Data::Struct(s) = &input.data {
            generate_header_rv_struct_reader_body(&s.fields)
        } else {
            return syn::Error::new_spanned(&input, "WowHeaderR can only be derived for structs.")
                .to_compile_error()
                .into();
        };

        let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

        let reader_body = match reader_body {
            Ok(body) => body,
            Err(e) => return e.to_compile_error().into(),
        };

        quote! {
            impl #impl_generics wow_data::types::WowHeaderR for #struct_name #ty_generics #where_clause {
                fn wow_read<R: Read + Seek>(reader: &mut R) -> wow_data::error::Result<Self> {
                    Ok(#reader_body)
                }
            }
        }
    };

    TokenStream::from(expanded)
}

fn generate_header_rv_struct_reader_body(fields: &Fields) -> syn::Result<proc_macro2::TokenStream> {
    let named_fields = if let Fields::Named(f) = fields {
        &f.named
    } else {
        return Err(syn::Error::new_spanned(
            fields,
            "VWowHeaderR on structs only supports named fields.",
        ));
    };

    let mut initializers = Vec::new();

    for field in named_fields {
        let field_name = field.ident.as_ref().unwrap();

        let mut is_versioned = false;
        for attr in &field.attrs {
            if !attr.path().is_ident("wow_data") {
                continue;
            }

            let mut found = false;
            let _ = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("versioned") {
                    found = true;
                }
                Ok(())
            });

            if found {
                is_versioned = true;
                break;
            }
        }

        match get_values_from_attrs::<Expr>(&field.attrs, "wow_data", &["skip"]) {
            Ok(map) => {
                initializers.push(if let Some(val) = map.get("skip") {
                    quote! { #field_name: #val }
                } else {
                    if is_versioned {
                        quote! { #field_name: reader.wow_read_versioned(version)? }
                    } else {
                        quote! { #field_name: reader.wow_read()? }
                    }
                });
            }
            Err(err) => {
                return Err(err);
            }
        }
    }

    Ok(quote! {
        Self {
            #(#initializers),*
        }
    })
}

fn generate_header_rv_enum_reader_body(
    data: &syn::DataEnum,
) -> syn::Result<proc_macro2::TokenStream> {
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

        match get_values_from_attrs::<Expr>(&variant.attrs, "wow_data", &["read_if"]) {
            Ok(map) => {
                if let Some(cond_expr) = map.get("read_if") {
                    conditional_arms.push(quote! { if #cond_expr { #constructor } });
                } else {
                    if default_arm.is_some() {
                        return Err(syn::Error::new_spanned(
                            variant,
                            "Only one enum variant can be the default (lacking a `read_if` attribute).",
                        ));
                    }
                    default_arm = Some(constructor);
                }
            }
            Err(err) => {
                return Err(err);
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

fn get_values_from_attrs<T: Parse + fmt::Debug>(
    attrs: &[syn::Attribute],
    attr_name: &str,
    attr_keys: &[&str],
) -> syn::Result<HashMap<String, T>> {
    let mut ret_val = HashMap::new();

    for attr in attrs {
        if !attr.path().is_ident(attr_name) {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            for attr_key in attr_keys {
                if meta.path.is_ident(attr_key) {
                    let value = meta.value()?;
                    let parsed: T = value.parse()?;
                    ret_val.insert((*attr_key).into(), parsed);
                }
            }
            Ok(())
        })?;
    }

    Ok(ret_val)
}

#[proc_macro_derive(WowHeaderW, attributes(wow_data))]
pub fn wow_header_w_derive(input: TokenStream) -> TokenStream {
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
            return syn::Error::new_spanned(&input, "WowHeaderW cannot be derived for unions.")
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
        impl #impl_generics wow_data::types::WowHeaderW for #ident #ty_generics #where_clause {
            fn wow_write<W: ::std::io::Write>(&self, writer: &mut W) -> wow_data::error::Result<()> {
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
            "WowHeaderW on structs only supports named fields.",
        ));
    };
    let write_statements = named_fields.iter().map(|field| {
        match get_values_from_attrs::<Expr>(&field.attrs, "wow_data", &["skip"]) {
            Ok(map) => {
                if let Some(_) = map.get("skip") {
                    quote! {}
                } else {
                    let field_name = field.ident.as_ref().unwrap();
                    quote! {
                        writer.wow_write(&self.#field_name)?;
                    }
                }
            }
            Err(_) => quote! {},
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
            "WowHeaderW on structs only supports named fields.",
        ));
    };

    let mut size_expressions = Vec::new();

    for field in named_fields {
        match get_values_from_attrs::<Expr>(&field.attrs, "wow_data", &["skip"]) {
            Ok(map) => {
                if let Some(_) = map.get("skip") {
                    size_expressions.push(quote! {0});
                } else {
                    let field_name = field.ident.as_ref().unwrap();
                    size_expressions.push(quote! {
                        self.#field_name.wow_size()
                    });
                }
            }
            Err(err) => {
                return Err(err);
            }
        }
    }

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

#[proc_macro_derive(WowDataR, attributes(wow_data))]
pub fn wow_data_r_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let wow_data_attrs_map =
        match get_values_from_attrs::<Type>(&input.attrs, "wow_data", &["version", "header"]) {
            Ok(ty) => ty,
            Err(e) => return e.to_compile_error().into(),
        };

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

    let mut initializers = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();

        match get_values_from_attrs::<Expr>(&field.attrs, "wow_data", &["skip"]) {
            Ok(map) => {
                if let Some(expr) = map.get("skip") {
                    initializers.push(quote! { #field_name: #expr });
                } else {
                    match get_values_from_attrs::<LitBool>(&field.attrs, "wow_data", &["versioned"])
                    {
                        Ok(submap) => {
                            if let Some(_) = submap.get("versioned") {
                                initializers.push(quote! { #field_name: reader.vnew_from_header(&header.#field_name)? });
                            } else {
                                initializers.push(quote! { #field_name: reader.new_from_header(&header.#field_name)? });
                            }
                        }
                        Err(err) => {
                            return err.to_compile_error().into();
                        }
                    }
                }
            }
            Err(err) => {
                return err.to_compile_error().into();
            }
        }
    }

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = match (
        wow_data_attrs_map.get("version"),
        wow_data_attrs_map.get("header"),
    ) {
        (Some(version_ty), Some(header_ty)) => {
            quote! {
                impl #impl_generics wow_data::types::WowDataR<#version_ty, #header_ty> for #struct_name #ty_generics #where_clause {
                    fn new_from_header<R: Read + Seek>(reader: &mut R, header: #header_ty) -> wow_data::error::Result<Self> {
                        Ok(Self{
                            #(#initializers),*
                        })
                    }
                }
            }
        }
        _ => {
            return syn::Error::new_spanned(
                &fields,
                "WowDataR needs #[wow_data(version = Version, header = Header)] definition.",
            )
            .to_compile_error()
            .into();
        }
    };

    TokenStream::from(expanded)
}
