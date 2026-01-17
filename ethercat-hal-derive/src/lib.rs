use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, parse_macro_input};
extern crate proc_macro;

#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(pdo_object_index))]
struct PdoObjectIndexAttribute(u16);

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
            let attrs: PdoObjectIndexAttribute = deluxe::extract_attributes(field)?;
            field_names.push(field_name);
            pdo_indices.push(attrs.0);
        }
    }
    Ok((field_names, pdo_indices))
}

#[proc_macro_derive(RxPdo, attributes(pdo_object_index))]
pub fn rxpdo_derive(item: TokenStream) -> TokenStream {
    rxpdo_derive2(item.into()).unwrap().into()
}

fn rxpdo_derive2(item: proc_macro2::TokenStream) -> deluxe::Result<proc_macro2::TokenStream> {
    let mut ast: DeriveInput = syn::parse2(item)?;

    let (field_name, pdo_index): (Vec<syn::Ident>, Vec<u16>) =
        extract_metedata_field_attributes(&mut ast)?;

    let ident = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics crate::coe::Configuration for #ident #ty_generics #where_clause {
            #[doc="Implemented by the ethercat_hal_derive::RxPdo derive macro"]
            async fn write_config<'a>(
                &self,
                device: &EthercrabSubDevicePreoperational<'a>,
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

        impl #impl_generics crate::pdo::RxPdo for #ident #ty_generics #where_clause {
            #[doc="Implemented by the ethercat_hal_derive::RxPdo derive macro"]
            fn get_objects(&self) -> Box<[Option<&dyn crate::pdo::RxPdoObject>]> {
                Box::new([
                    #(
                        self.#field_name.as_ref().map(|o| o as &dyn crate::pdo::RxPdoObject),
                    )*
                ])
            }
        }
    };

    Ok(expanded)
}

#[proc_macro_derive(TxPdo, attributes(pdo_object_index))]
pub fn txpdo_derive(item: TokenStream) -> TokenStream {
    txpdo_derive2(item.into()).unwrap().into()
}

fn txpdo_derive2(item: proc_macro2::TokenStream) -> deluxe::Result<proc_macro2::TokenStream> {
    let mut ast: DeriveInput = syn::parse2(item)?;

    let (field_name, pdo_index): (Vec<syn::Ident>, Vec<u16>) =
        extract_metedata_field_attributes(&mut ast)?;

    let ident = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics crate::coe::Configuration for #ident #ty_generics #where_clause {
            #[doc="Implemented by the ethercat_hal_derive::TxPdo derive macro"]
            async fn write_config<'a>(
                &self,
                device: &EthercrabSubDevicePreoperational<'a>,
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

        impl #impl_generics crate::pdo::TxPdo for #ident #ty_generics #where_clause {
            #[doc="Implemented by the ethercat_hal_derive::TxPdo derive macro"]
            fn get_objects(&self) -> Box<[Option<&dyn crate::pdo::TxPdoObject>]> {
                Box::new([
                    #(
                        self.#field_name.as_ref().map(|o| o as &dyn crate::pdo::TxPdoObject),
                    )*
                ])
            }

            #[doc="Implemented by the ethercat_hal_derive::TxPdo derive macro"]
            fn get_objects_mut(&mut self) -> Box<[Option<&mut dyn crate::pdo::TxPdoObject>]> {
                Box::new([
                    #(
                        self.#field_name.as_mut().map(|o| o as &mut dyn crate::pdo::TxPdoObject),
                    )*
                ])
            }
        }
    };

    Ok(expanded)
}

#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(pdo_object))]
struct PdoObjectAttribute {
    pub bits: usize,
}

fn pdo_object_derive2(item: proc_macro2::TokenStream) -> deluxe::Result<proc_macro2::TokenStream> {
    let mut ast: DeriveInput = syn::parse2(item)?;

    let PdoObjectAttribute { bits } = deluxe::extract_attributes(&mut ast)?;

    let ident = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics crate::pdo::PdoObject for #ident #ty_generics #where_clause {
            #[doc="Implemented by the ethercat_hal_derive::PdoObject macro"]
            fn size(&self) -> usize {
                #bits
            }
        }
    };

    Ok(expanded)
}

#[proc_macro_derive(PdoObject, attributes(pdo_object))]
pub fn pdo_object_derive(item: TokenStream) -> TokenStream {
    pdo_object_derive2(item.into()).unwrap().into()
}

#[proc_macro_derive(EthercatDevice)]
pub fn ethercat_device_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    #[allow(unused_assignments)]
    let mut output_impl = quote! {};
    #[allow(unused_assignments)]
    let mut input_impl = quote! {};
    let mut has_rxpdo = false;
    let mut has_txpdo = false;

    if let Data::Struct(data_struct) = input.data {
        for field in data_struct.fields.iter() {
            if let Some(ident) = &field.ident {
                if ident == "rxpdo" {
                    has_rxpdo = true;
                }
                if ident == "txpdo" {
                    has_txpdo = true;
                }
            }
        }
    }

    if has_rxpdo {
        output_impl = quote! {
            #[doc="Implemented by the ethercat_hal_derive::EthercatDevice derive macro"]
            fn output(&self, output: &mut bitvec::prelude::BitSlice<u8, bitvec::prelude::Lsb0>) -> Result<(), anyhow::Error> {
                self.rxpdo.write(output)
            }
            #[doc="Implemented by the ethercat_hal_derive::EthercatDevice derive macro"]
            fn output_len(&self) -> usize {
                self.rxpdo.size()
            }
        };
    } else {
        output_impl = quote! {
            #[doc="Implemented by the ethercat_hal_derive::EthercatDevice derive macro"]
            fn output(&self, _output: &mut bitvec::prelude::BitSlice<u8, bitvec::prelude::Lsb0>) -> Result<(), anyhow::Error> {
                Ok(())
            }
            #[doc="Implemented by the ethercat_hal_derive::EthercatDevice derive macro"]
            fn output_len(&self) -> usize {
                0
            }
        };
    }

    if has_txpdo {
        input_impl = quote! {
            #[doc="Implemented by the ethercat_hal_derive::EthercatDevice derive macro"]
            fn input(&mut self, input: & bitvec::prelude::BitSlice<u8, bitvec::prelude::Lsb0>) -> Result<(), anyhow::Error> {
                self.txpdo.read(input)
            }
            #[doc="Implemented by the ethercat_hal_derive::EthercatDevice derive macro"]
            fn input_len(&self) -> usize {
                self.txpdo.size()
            }
        };
    } else {
        input_impl = quote! {
            #[doc="Implemented by the ethercat_hal_derive::EthercatDevice derive macro"]
            fn input(&mut self, _input: & bitvec::prelude::BitSlice<u8, bitvec::prelude::Lsb0>) -> Result<(), anyhow::Error> {
                Ok(())
            }
            #[doc="Implemented by the ethercat_hal_derive::EthercatDevice derive macro"]
            fn input_len(&self) -> usize {
                0
            }
        };
    }

    let expanded = quote! {
        impl crate::devices::EthercatDevice for #name {
            #output_impl
            #input_impl

            #[doc="Implemented by the ethercat_hal_derive::EthercatDevice derive macro"]
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            #[doc="Implemented by the ethercat_hal_derive::EthercatDevice derive macro"]
            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }

            #[doc="Implemented by the ethercat_hal_derive::EthercatDevice derive macro"]
            fn is_module(&self) -> bool {
                false
            }

            #[doc="Implemented by the ethercat_hal_derive::EthercatDevice derive macro"]
            fn get_module(&self) -> Option<crate::devices::Module> {
                None
            }

            #[doc="Implemented by the ethercat_hal_derive::EthercatDevice derive macro"]
            fn set_module(&mut self,module : crate::devices::Module){
                ()
            }

        }

        impl crate::devices::EthercatDeviceUsed for #name {
            #[doc="Implemented by the ethercat_hal_derive::EthercatDevice derive macro"]
            fn is_used(&self) -> bool {
                self.is_used
            }

            #[doc="Implemented by the ethercat_hal_derive::EthercatDevice derive macro"]
            fn set_used(&mut self, used: bool) {
                self.is_used = used;
            }
        }
    };

    TokenStream::from(expanded)
}
