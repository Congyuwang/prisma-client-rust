use crate::generator::prelude::*;

pub fn builder_fn(field: &dml::RelationField) -> TokenStream {
    let relation_model_name_snake = snake_ident(&field.relation_info.referenced_model);

    quote! {
        pub fn with(mut self, params: impl Into<#relation_model_name_snake::WithParam>) -> Self {
            self.0 = self.0.with(params.into());
            self
        }
    }
}

fn enum_variant(field: &dml::RelationField) -> TokenStream {
    let field_name_pascal = pascal_ident(&field.name);
    let relation_model_name_snake = snake_ident(&field.relation_info.referenced_model);

    let args = match field.arity {
        dml::FieldArity::List => quote!(ManyArgs),
        _ => quote!(UniqueArgs),
    };

    quote!(#field_name_pascal(super::#relation_model_name_snake::#args))
}

fn into_selection_arm(field: &dml::RelationField) -> TokenStream {
    let field_name_snake = snake_ident(&field.name);
    let field_name_pascal = pascal_ident(&field.name);
    let relation_model_name_snake = snake_ident(&field.relation_info.referenced_model);

    let pcr = quote!(::prisma_client_rust);

    let body = match field.arity {
        dml::FieldArity::List => quote! {
            let (arguments, mut nested_selections) = args.to_graphql();
            nested_selections.extend(<super::#relation_model_name_snake::Types as #pcr::ModelTypes>::scalar_selections());

            #pcr::Selection::new(
                #field_name_snake::NAME,
                None,
                arguments,
                nested_selections
            )
        },
        _ => quote! {
            let mut selections = <super::#relation_model_name_snake::Types as #pcr::ModelTypes>::scalar_selections();
            selections.extend(args.with_params.into_iter().map(Into::<#pcr::Selection>::into));

            #pcr::Selection::new(
                #field_name_snake::NAME,
                None,
                [],
                selections
            )
        },
    };

    quote! {
        Self::#field_name_pascal(args) => {
            #body
        }
    }
}

pub fn enum_definition(model: &dml::Model) -> TokenStream {
    let variants = model.relation_fields().map(enum_variant);
    let into_selection_arms = model.relation_fields().map(into_selection_arm);

    quote! {
        #[derive(Clone)]
        pub enum WithParam {
            #(#variants),*
        }

        impl Into<::prisma_client_rust::Selection> for WithParam {
            fn into(self) -> ::prisma_client_rust::Selection {
                match self {
                    #(#into_selection_arms),*
                }
            }
        }
    }
}
