use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{DeriveInput, Error};

#[proc_macro_derive(BuildEvent)]
pub fn build_event_derive(item: TokenStream) -> TokenStream {
    build_event_derive2(item.into()).unwrap().into()
}

fn build_event_derive2(item: TokenStream2) -> Result<TokenStream2, Error> {
    // Parse the input tokens into a syntax tree
    let ast: DeriveInput = syn::parse2(item)?;

    let ident = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    // Generate the BuildEvent impl
    let expanded = quote! {
        impl #impl_generics control_core::socketio::event::BuildEvent for #ident #ty_generics #where_clause {
            #[doc = "Implemented by the BuildEvent derive macro"]
            fn build(&self) -> control_core::socketio::event::Event<Self> {

                control_core::socketio::event::Event::new(stringify!(#ident), self.clone())

            }
        }
    };

    Ok(expanded)
}

#[proc_macro_derive(Machine)]
pub fn machine_derive(item: TokenStream) -> TokenStream {
    machine_derive2(item.into()).unwrap().into()
}

fn machine_derive2(item: TokenStream2) -> Result<TokenStream2, Error> {
    // Parse the input tokens into a syntax tree
    let ast: DeriveInput = syn::parse2(item)?;

    let ident = &ast.ident;

    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics control_core::machines::Machine for #ident #ty_generics #where_clause {
            fn get_machine_identification_unique(&self) -> control_core::machines::identification::MachineIdentificationUnique {
                self.machine_identification_unique.clone()
            }
        }

        impl #impl_generics control_core::machines::AnyGetters for #ident #ty_generics #where_clause {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }
        }
    };

    Ok(expanded)
}
