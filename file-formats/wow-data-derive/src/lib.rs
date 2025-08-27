use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Expr, Fields, Ident, Type, parse_macro_input};

#[proc_macro_derive(WowHeaderR, attributes(wow_data))]
pub fn wow_header_r_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_wow_attrs = match parse_wow_data_attrs(&input.attrs) {
        Ok(value) => value,
        Err(err) => {
            return err.to_compile_error().into();
        }
    };

    let struct_name = &input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = if let Some(version_ty) = &struct_wow_attrs.version {
        let reader_body = match &input.data {
            Data::Struct(s) => generate_header_rv_struct_reader_body(&struct_wow_attrs, &s.fields),
            Data::Enum(e) => generate_header_rv_enum_reader_body(e),
            Data::Union(_) => {
                return syn::Error::new_spanned(&input, "WowHeaderR cannot be derived for unions.")
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
    } else if let Some(ty) = struct_wow_attrs.from_type {
        let Data::Enum(_) = &input.data else {
            return syn::Error::new_spanned(
                &input,
                "WowHeaderR with wow_data(from_type=TYPE) can only be derived for enums.",
            )
            .to_compile_error()
            .into();
        };

        quote! {
            impl #impl_generics wow_data::types::WowHeaderR for #struct_name #ty_generics #where_clause {
                fn wow_read<R: Read + Seek>(reader: &mut R) -> wow_data::error::Result<Self> {
                    Ok(#ty::wow_read(reader)?.try_into()?)
                }
            }
        }
    } else {
        let reader_body = if let Data::Struct(s) = &input.data {
            generate_header_rv_struct_reader_body(&struct_wow_attrs, &s.fields)
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

fn generate_header_rv_struct_reader_body(
    struct_wow_attrs: &WowDataAttrs,
    fields: &Fields,
) -> syn::Result<proc_macro2::TokenStream> {
    match fields {
        Fields::Named(f) => {
            let named_fields = &f.named;
            let mut read_lines = Vec::new();
            let mut initializers = Vec::new();

            for field in named_fields {
                let field_name = field.ident.as_ref().unwrap();
                let field_ty = &field.ty;

                let wow_data_attrs = parse_wow_data_attrs(&field.attrs)?;

                if let Some(val) = wow_data_attrs.override_read {
                    read_lines.push(quote! { let #field_name = #val; });
                } else {
                    if wow_data_attrs.versioned {
                        read_lines.push(
                    quote! { let #field_name: #field_ty = reader.wow_read_versioned(version)?; },
                );
                    } else {
                        read_lines
                            .push(quote! { let #field_name: #field_ty = reader.wow_read()?; });
                    }
                }

                initializers.push(quote! { #field_name });
            }

            Ok(quote! {{
                #(#read_lines)*
                Self {
                    #(#initializers),*
                }
            }})
        }
        Fields::Unnamed(_) => {
            let Some(_) = &struct_wow_attrs.bitflags else {
                return Err(syn::Error::new_spanned(
                    fields,
                    "WowHeaderR on structs with unnamed fields only supports bitflags.",
                ));
            };
            Ok(quote! {Self::from_bits_retain(reader.wow_read()?)})
        }
        Fields::Unit => {
            return Err(syn::Error::new_spanned(
                fields,
                "WowHeaderR on structs does't support unit fields.",
            ));
        }
    }
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

        let wow_data_attrs = parse_wow_data_attrs(&variant.attrs)?;

        if let Some(cond_expr) = wow_data_attrs.read_if {
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

#[derive(Debug)]
struct WowDataAttrs {
    versioned: bool,
    default: bool,
    version: Option<Type>,
    header: Option<Type>,
    read_if: Option<Expr>,
    override_read: Option<Expr>,
    from_type: Option<Type>,
    expr: Option<Expr>,
    bitflags: Option<Type>,
}

fn parse_wow_data_attrs(attrs: &[syn::Attribute]) -> syn::Result<WowDataAttrs> {
    let mut data_attrs = WowDataAttrs {
        default: false,
        versioned: false,
        version: None,
        header: None,
        read_if: None,
        override_read: None,
        from_type: None,
        expr: None,
        bitflags: None,
    };

    for attr in attrs {
        if !attr.path().is_ident("wow_data") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("versioned") {
                data_attrs.versioned = true;
            }

            if meta.path.is_ident("version") {
                let value = meta.value()?;
                data_attrs.version = Some(value.parse()?);
            }

            if meta.path.is_ident("header") {
                let value = meta.value()?;
                data_attrs.header = Some(value.parse()?);
            }

            if meta.path.is_ident("read_if") {
                let value = meta.value()?;
                data_attrs.read_if = Some(value.parse()?);
            }

            if meta.path.is_ident("override_read") {
                let value = meta.value()?;
                data_attrs.override_read = Some(value.parse()?);
            }

            if meta.path.is_ident("from_type") {
                let value = meta.value()?;
                data_attrs.from_type = Some(value.parse()?);
            }

            if meta.path.is_ident("expr") {
                let value = meta.value()?;
                data_attrs.expr = Some(value.parse()?);
            }

            if meta.path.is_ident("default") {
                data_attrs.default = true;
            }

            if meta.path.is_ident("bitflags") {
                let value = meta.value()?;
                data_attrs.bitflags = Some(value.parse()?);
            }

            Ok(())
        })?;
    }

    Ok(data_attrs)
}

#[proc_macro_derive(WowHeaderW, attributes(wow_data))]
pub fn wow_header_w_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_wow_attrs = match parse_wow_data_attrs(&input.attrs) {
        Ok(value) => value,
        Err(err) => {
            return err.to_compile_error().into();
        }
    };

    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let (writer_body, sizer_body) = match &input.data {
        Data::Struct(s) => (
            generate_struct_writer_body(&struct_wow_attrs, &s.fields),
            generate_struct_size_body(&struct_wow_attrs, &s.fields),
        ),
        Data::Enum(e) => (
            generate_enum_writer_body(&struct_wow_attrs, e),
            generate_enum_size_body(&struct_wow_attrs, e),
        ),
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

fn generate_struct_writer_body(
    struct_wow_attrs: &WowDataAttrs,
    fields: &Fields,
) -> syn::Result<proc_macro2::TokenStream> {
    match fields {
        Fields::Named(f) => {
            let mut write_statements = Vec::new();

            for field in &f.named {
                let wow_data_attrs = parse_wow_data_attrs(&field.attrs)?;

                if let Some(_) = wow_data_attrs.override_read {
                    write_statements.push(quote! {});
                } else {
                    let field_name = field.ident.as_ref().unwrap();
                    write_statements.push(quote! {
                        writer.wow_write(&self.#field_name)?;
                    });
                }
            }

            Ok(quote! {
                #(#write_statements)*
                Ok(())
            })
        }
        Fields::Unnamed(_) => {
            let Some(_) = &struct_wow_attrs.bitflags else {
                return Err(syn::Error::new_spanned(
                    fields,
                    "WowHeaderW on structs with unnamed fields only supports bitflags.",
                ));
            };
            Ok(quote! {
                writer.wow_write(&self.bits())?;
                Ok(())
            })
        }
        Fields::Unit => {
            return Err(syn::Error::new_spanned(
                fields,
                "WowHeaderW on structs does't support unit fields.",
            ));
        }
    }
}

fn generate_struct_size_body(
    struct_wow_attrs: &WowDataAttrs,
    fields: &Fields,
) -> syn::Result<proc_macro2::TokenStream> {
    match fields {
        Fields::Named(f) => {
            let mut size_expressions = Vec::new();

            for field in &f.named {
                let wow_data_attrs = parse_wow_data_attrs(&field.attrs)?;

                if let Some(_) = wow_data_attrs.override_read {
                    size_expressions.push(quote! {0});
                } else {
                    let field_name = field.ident.as_ref().unwrap();
                    size_expressions.push(quote! {
                        self.#field_name.wow_size()
                    });
                }
            }

            Ok(quote! {
                0 #(+ #size_expressions)*
            })
        }
        Fields::Unnamed(_) => {
            let Some(ty) = &struct_wow_attrs.bitflags else {
                return Err(syn::Error::new_spanned(
                    fields,
                    "WowHeaderW on structs with unnamed fields only supports bitflags.",
                ));
            };
            Ok(quote! {
                #ty::default().wow_size()
            })
        }
        Fields::Unit => {
            return Err(syn::Error::new_spanned(
                fields,
                "WowHeaderW on structs does't support unit fields.",
            ));
        }
    }
}

fn generate_enum_writer_body(
    struct_wow_attrs: &WowDataAttrs,
    data: &syn::DataEnum,
) -> syn::Result<proc_macro2::TokenStream> {
    if let Some(ty) = &struct_wow_attrs.from_type {
        Ok(quote! {
            #ty::wow_write(&(*self).into(), writer)?;
            Ok(())
        })
    } else {
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
}

fn generate_enum_size_body(
    struct_wow_attrs: &WowDataAttrs,
    data: &syn::DataEnum,
) -> syn::Result<proc_macro2::TokenStream> {
    if let Some(ty) = &struct_wow_attrs.from_type {
        Ok(quote! {
            #ty::wow_size(&(*self).into())
        })
    } else {
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
}

#[proc_macro_derive(WowDataR, attributes(wow_data))]
pub fn wow_data_r_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_wow_data_attrs = match parse_wow_data_attrs(&input.attrs) {
        Ok(data_attrs) => data_attrs,
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
        let wow_data_attrs = match parse_wow_data_attrs(&field.attrs) {
            Ok(data_attrs) => data_attrs,
            Err(e) => return e.to_compile_error().into(),
        };

        if let Some(expr) = wow_data_attrs.override_read {
            initializers.push(quote! { #field_name: #expr });
        } else {
            if wow_data_attrs.versioned {
                initializers
                    .push(quote! { #field_name: reader.v_new_from_header(&header.#field_name)? });
            } else {
                initializers
                    .push(quote! { #field_name: reader.new_from_header(&header.#field_name)? });
            }
        }
    }

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let Some(header_ty) = struct_wow_data_attrs.header else {
        return syn::Error::new_spanned(
            &input,
            "WowDataR needs at least #[wow_data(header = H)] definition.",
        )
        .to_compile_error()
        .into();
    };

    TokenStream::from(if struct_wow_data_attrs.version.is_none() {
        quote! {
            impl #impl_generics wow_data::types::WowDataR<#header_ty> for #struct_name #ty_generics #where_clause {
                fn new_from_header<R: Read + Seek>(reader: &mut R, header: &#header_ty) -> wow_data::error::Result<Self> {
                    Ok(Self{
                        #(#initializers),*
                    })
                }
            }
        }
    } else {
        let version_ty = struct_wow_data_attrs.version.unwrap();
        quote! {
            impl #impl_generics wow_data::types::VWowDataR<#version_ty, #header_ty> for #struct_name #ty_generics #where_clause {
                fn new_from_header<R: Read + Seek>(reader: &mut R, header: &#header_ty) -> wow_data::error::Result<Self> {
                    Ok(Self{
                        #(#initializers),*
                    })
                }
            }
        }
    })
}

fn generate_wow_enum_from_value_lines(
    data: &syn::DataEnum,
) -> syn::Result<Vec<proc_macro2::TokenStream>> {
    let mut lines = Vec::new();

    let mut has_default = None;

    for variant in &data.variants {
        let variant_ident = &variant.ident;
        let wow_data_attrs = parse_wow_data_attrs(&variant.attrs)?;

        if wow_data_attrs.default {
            has_default = Some(quote! {
                _ => Self::#variant_ident,
            });
        }

        if let Some(expr) = wow_data_attrs.expr {
            match &variant.fields {
                syn::Fields::Unit => {
                    lines.push(quote! { #expr => Self::#variant_ident, });
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        &variant,
                        "WowEnumFrom only supports unit variants.",
                    ));
                }
            }
        } else {
            return Err(syn::Error::new_spanned(
                variant,
                "WowEnumFrom requires a wow_data(expr=EXPR) attribute for each variant",
            ));
        };
    }

    lines.push(if let Some(has_default) = has_default {
        has_default
    }else {
        quote! {
            _ => {
                return Err(wow_data::error::WowDataError::InvalidEnumParsedValue("".into(), "".into()).into());
            }
        }
    });

    Ok(lines)
}

fn generate_wow_enum_to_value_lines(
    struct_name: &Ident,
    data: &syn::DataEnum,
) -> syn::Result<Vec<proc_macro2::TokenStream>> {
    let mut lines = Vec::new();

    for variant in &data.variants {
        let variant_ident = &variant.ident;

        let wow_data_attrs = parse_wow_data_attrs(&variant.attrs)?;
        if let Some(expr) = wow_data_attrs.expr {
            match &variant.fields {
                syn::Fields::Unit => {
                    lines.push(quote! { #struct_name::#variant_ident => #expr });
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        &variant,
                        "WowEnumFrom only supports unit variants.",
                    ));
                }
            }
        } else {
            return Err(syn::Error::new_spanned(
                variant,
                "WowEnumFrom requires a wow_data(expr=EXPR) attribute for each variant",
            ));
        };
    }

    Ok(lines)
}

#[proc_macro_derive(WowEnumFrom, attributes(wow_data))]
pub fn wow_enum_from_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_wow_attrs = match parse_wow_data_attrs(&input.attrs) {
        Ok(value) => value,
        Err(err) => {
            return err.to_compile_error().into();
        }
    };

    let Some(ty) = struct_wow_attrs.from_type else {
        return syn::Error::new_spanned(
            &input,
            "WowEnumFrom requires a wow_data(from_type=TYPE) attribute.",
        )
        .to_compile_error()
        .into();
    };

    let enum_name = &input.ident;

    let Data::Enum(enum_data) = &input.data else {
        return syn::Error::new_spanned(&input, "WowEnumFrom can only be derived for enums.")
            .to_compile_error()
            .into();
    };

    let from_value_lines = match generate_wow_enum_from_value_lines(&enum_data) {
        Ok(body) => body,
        Err(e) => return e.to_compile_error().into(),
    };

    let to_value_lines = match generate_wow_enum_to_value_lines(&enum_name, &enum_data) {
        Ok(body) => body,
        Err(e) => return e.to_compile_error().into(),
    };

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics TryFrom<#ty> for #enum_name #ty_generics #where_clause {
            type Error = wow_data::error::WowDataError;

            fn try_from(value: #ty) -> wow_data::error::Result<Self> {
                Ok(match value {
                    #(#from_value_lines)*
                })
            }
        }

        impl #impl_generics From<#enum_name #ty_generics> for #ty #where_clause {
            fn from(value: #enum_name) -> Self {
                match value {
                    #(#to_value_lines),*
                }
            }
        }
    };

    TokenStream::from(expanded)
}
