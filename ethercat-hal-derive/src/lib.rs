use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields, GenericArgument, PathArguments, Type};

// Import the necessary type for procedural macros.
// Import the macro quoting utility to easily generate code.
// Import parsing utilities and types from syn.

// Define a derive procedural macro named PdoAssignmentDerive.
#[proc_macro_derive(PdoAssignmentDerive)]
pub fn derive_pdo_assignment(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a DeriveInput syntax tree.
    let input = parse_macro_input!(input as DeriveInput);

    // Extract the identifier (name) of the struct.
    let name = input.ident;
    // Split the generics into parts needed for the impl block.
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Match on the data inside the input to ensure it's a struct with named fields.
    let fields = match input.data {
        // Match specifically for a struct.
        Data::Struct(data) => match data.fields {
            // Accept only named fields in the struct.
            Fields::Named(fields) => fields.named,
            // Panic if the struct fields are not named.
            _ => panic!("Only named fields are supported"),
        },
        // Panic if the derive is applied to anything but a struct.
        _ => panic!("PdoAssignment can only be derived for structs"),
    };

    // Filter and collect the fields that are of type Option<T> into a vector.
    let pdo_fields = fields
        .iter() // Iterate over each field.
        .filter_map(|field| {
            // Get the name (identifier) of the field.
            let field_name = field.ident.as_ref()?;

            // Check if the type of the field is a path (like Option<T>).
            if let Type::Path(type_path) = &field.ty {
                // Get the last segment of the type path.
                if let Some(segment) = type_path.path.segments.last() {
                    // Check if the type is 'Option'.
                    if segment.ident == "Option" {
                        // Ensure that the generic arguments are enclosed in angle brackets.
                        if let PathArguments::AngleBracketed(args) = &segment.arguments {
                            // Check that there is a type inside the Option.
                            if let Some(GenericArgument::Type(_)) = args.args.first() {
                                // Return the field name if all conditions are met.
                                return Some(field_name);
                            }
                        }
                    }
                }
            }
            // Otherwise, skip this field.
            None
        })
        // Collect the resulting field names into a vector.
        .collect::<Vec<_>>();

    // Generate the implementation of the PdoAssignment trait for the struct.
    let expanded = quote! {
        use crate::pdo::PdoObject;

        impl #impl_generics PdoAssignment for #name #ty_generics #where_clause {

            // Define the get_objects method.
            fn get_objects(&self) -> &[Option<&dyn PdoObject>] {
                // Leak a boxed vector converted into a slice.
                Box::leak(Box::new(vec![
                    // For each relevant field, convert its Option value to an Option reference to a PdoObject trait.
                    #(
                        self.#pdo_fields.as_ref().map(|obj| obj as &dyn PdoObject),
                    )*
                ]))
            }
        }
    };

    // Convert the generated code back into a TokenStream.
    TokenStream::from(expanded)
}
