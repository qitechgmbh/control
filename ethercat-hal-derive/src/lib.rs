use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput};

#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(pdo_map_register))]
struct PdoMapIndex(u16);

fn extract_metedata_field_attributes(
    ast: &mut DeriveInput,
) -> deluxe::Result<(Vec<syn::Ident>, Vec<u16>)> {
    let mut field_names = Vec::new();
    let mut pdo_indices = Vec::new();
    if let Data::Struct(s) = &mut ast.data {
        for field in s.fields.iter_mut() {
            let field_name = field
                .ident
                .as_ref()
                .cloned()
                .expect("Field must have a name");
            let attrs: PdoMapIndex = deluxe::extract_attributes(field)?;
            field_names.push(field_name);
            pdo_indices.push(attrs.0);
        }
    }
    Ok((field_names, pdo_indices))
}

fn rxpdo_derive2(item: proc_macro2::TokenStream) -> deluxe::Result<proc_macro2::TokenStream> {
    let mut ast: DeriveInput = syn::parse2(item)?;

    // let MetaDataStructAttributes { author } = deluxe::extract_attributes(&mut ast)?;

    // extract field attributes
    let (field_name, pdo_index): (Vec<syn::Ident>, Vec<u16>) =
        extract_metedata_field_attributes(&mut ast)?;

    let ident = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics Configuration for #ident #ty_generics #where_clause {
                async fn write_config<'a>(
                &self,
                device: &'a EthercrabSubDevice<'a>,
            ) -> Result<(), anyhow::Error> {
                device.sdo_write(0x1C12, 0, 0u8).await?;
                let mut len = 0;

                #(
                     if let Some(_) = &self.#field_name {
                     len += 1;
                     device.sdo_write(0x1C12, len, #pdo_index).await?;
                 }
                )*

                device.sdo_write(0x1C12, 0, len).await?;
                Ok(())
            }
        }

        impl #impl_generics RxPdo for #ident #ty_generics #where_clause {
            fn get_objects(&self) -> &[Option<&dyn RxPdoObject>] {
                let objs = vec![
                    #(
                        self.#field_name.as_ref().map(|o| o as &dyn RxPdoObject),
                    )*
                ];
                Box::leak(objs.into_boxed_slice())
            }
        }
    };

    Ok(expanded)
}

#[proc_macro_derive(RxPdoDerive, attributes(pdo_map_register))]
pub fn rxpdo_derive(item: TokenStream) -> TokenStream {
    rxpdo_derive2(item.into()).unwrap().into()
}

fn txpdo_derive2(item: proc_macro2::TokenStream) -> deluxe::Result<proc_macro2::TokenStream> {
    let mut ast: DeriveInput = syn::parse2(item)?;

    // let MetaDataStructAttributes { author } = deluxe::extract_attributes(&mut ast)?;

    // extract field attributes
    let (field_name, pdo_index): (Vec<syn::Ident>, Vec<u16>) =
        extract_metedata_field_attributes(&mut ast)?;

    let ident = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics Configuration for #ident #ty_generics #where_clause {
                async fn write_config<'a>(
                &self,
                device: &'a EthercrabSubDevice<'a>,
            ) -> Result<(), anyhow::Error> {
                device.sdo_write(0x1C13, 0, 0u8).await?;
                let mut len = 0;

                #(
                     if let Some(_) = &self.#field_name {
                     len += 1;
                     device.sdo_write(0x1C13, len, #pdo_index).await?;
                 }
                )*

                device.sdo_write(0x1C13, 0, len).await?;
                Ok(())
            }
        }

        impl #impl_generics TxPdo for #ident #ty_generics #where_clause {
            fn get_objects(&self) -> &[Option<&dyn TxPdoObject>] {
                let objs = vec![
                    #(
                        self.#field_name.as_ref().map(|o| o as &dyn TxPdoObject),
                    )*
                ];
                Box::leak(objs.into_boxed_slice())
            }

            fn get_objects_mut(&mut self) -> &mut [Option<&mut dyn TxPdoObject>] {
                let objs = vec![
                    #(
                        self.#field_name.as_mut().map(|o| o as &mut dyn TxPdoObject),
                    )*
                ];
                Box::leak(objs.into_boxed_slice())
            }
        }
    };

    Ok(expanded)
}

#[proc_macro_derive(TxPdoDerive, attributes(pdo_map_register))]
pub fn txpdo_derive(item: TokenStream) -> TokenStream {
    txpdo_derive2(item.into()).unwrap().into()
}
