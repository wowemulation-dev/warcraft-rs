use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Expr, Fields, Type, parse::Parse, parse_macro_input};

#[proc_macro_derive(WowDataRV, attributes(wow_data))]
pub fn wow_data_rv_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let version_ty = match get_value_from_attrs::<Type>(&input.attrs, "wow_data", "version") {
        Ok(ty) => ty,
        Err(e) => return e.to_compile_error().into(),
    };

    // Ensure we are deriving for a struct with named fields.
    let fields = if let Data::Struct(s) = &input.data {
        if let Fields::Named(f) = &s.fields {
            &f.named
        } else {
            return syn::Error::new_spanned(
                &s.fields,
                "WowDataRV can only be derived for structs with named fields.",
            )
            .to_compile_error()
            .into();
        }
    } else {
        return syn::Error::new_spanned(&input, "WowDataRV can only be derived for structs.")
            .to_compile_error()
            .into();
    };

    // Generate the field initializers for the `wow_read` method body.
    let initializers = fields.iter().map(|field| {
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

    // Assemble the final `impl` block.
    let struct_name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics WowDataRV<#version_ty> for #struct_name #ty_generics #where_clause {
            fn wow_read<R: ::std::io::Read + ::std::io::Seek>(reader: &mut R, version: #version_ty) -> Result<Self> {
                Ok(Self {
                    #(#initializers),*
                })
            }
        }
    };

    TokenStream::from(expanded)
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

    let struct_name = &input.ident;

    let fields = if let Data::Struct(s) = &input.data {
        if let Fields::Named(f) = &s.fields {
            &f.named
        } else {
            return syn::Error::new_spanned(
                &s.fields,
                "WowDataW can only be derived for structs with named fields.",
            )
            .to_compile_error()
            .into();
        }
    } else {
        return syn::Error::new_spanned(&input, "WowDataW can only be derived for structs.")
            .to_compile_error()
            .into();
    };

    let write_statements = fields.iter().map(|field| {
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

    let size_expressions = fields.iter().map(|field| {
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

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics WowDataW for #struct_name #ty_generics #where_clause {
            fn wow_write<W: ::std::io::Write>(&self, writer: &mut W) -> Result<()> {
                #(#write_statements)*
                Ok(())
            }

            fn wow_size(&self) -> usize {
                0 #(+ #size_expressions)*
            }
        }
    };

    TokenStream::from(expanded)
}
