extern crate proc_macro;
extern crate core;

use proc_macro::TokenStream;
use std::collections::{HashMap, VecDeque};
use proc_macro2::Span;

use quote::quote;
use syn;
use syn::{Ident, Fields, FieldsNamed, File, GenericArgument, ItemStruct, Lit, Meta, MetaList, NestedMeta, PathArguments, Type};
use syn::visit::Visit;


// todo: generate enum with getters
// todo: return getter strings
// todo: return getter enum
// todo: pass through the original code, remove the macro attributes from it


#[derive(Debug)]
struct Struct {
    ident: String,
    fields: Vec<Field>,
    generate: bool,
    return_type: Option<String>,
    replacement_type: Option<String>,
}

fn get_nested_meta_params(meta: &MetaList) -> HashMap<String, String> {
    meta
        .nested
        .iter()
        .map(|nm| match nm {
            NestedMeta::Meta(m) => match m {
                Meta::NameValue(nameval) =>
                    match (nameval.path.get_ident(), &nameval.lit) {
                        (Some(ident), Lit::Str(lit)) => {
                            (ident.to_string(), lit.value())
                        }
                        _ => panic!("struct_deep_getter attributes are expected to be strings")
                    }
                _ => panic!("struct_deep_getter attributes should be named list")
            }
            NestedMeta::Lit(_) => panic!("struct_deep_getter attributes should be named list")
        })
        .collect()
}

fn get_meta(i: &ItemStruct) -> Option<MetaList> {
    for attr in &i.attrs {
        if let Ok(Meta::List(meta)) = attr.parse_meta() {
            if let Some(macro_name) = meta.path.get_ident() {
                if macro_name == "struct_deep_getter" {
                    return Some(meta);
                }
            }
        }
    }

    None
}

#[derive(Debug)]
struct Field {
    ident: String,
    types: Vec<String>,
}

fn extract_types_deque(start_type: &Type) -> Vec<String> {
    let mut res = Vec::new();
    let mut rec_ty = VecDeque::new();
    rec_ty.push_front(start_type.clone());

    while let Some(ty) = rec_ty.pop_front() {
        match ty {
            Type::Path(p) => {
                for seg in p.path.segments {
                    res.push(seg.ident.to_string());

                    if !seg.arguments.is_empty() {
                        match seg.arguments {
                            PathArguments::AngleBracketed(ab_arg) => {
                                for arg in ab_arg.args.iter() {
                                    match arg {
                                        GenericArgument::Type(next_ty) => {
                                            rec_ty.push_back(next_ty.clone());
                                        }
                                        GenericArgument::Lifetime(_) => {
                                            // ignore lifetimes
                                        }
                                        _ => panic!("only types as arguments are supported")
                                    }
                                }
                            }
                            _ => panic!("only angle-bracketed type segments are supported")
                        }
                    }
                }
            }
            Type::Reference(r) => {
                // ignore refs push concrete types
                rec_ty.push_back(r.elem.as_ref().clone());
            }
            _ => panic!("only type paths are supported")
        }
    }

    res
}

fn extract_fields(fields: &FieldsNamed) -> Vec<Field> {
    fields
        .named
        .iter()
        // .filter(|field| matches!(field.vis, Visibility::Public(_)))
        .map(|field| {
            let ident = field.ident.as_ref().unwrap().to_string();
            let types = extract_types_deque(&field.ty);
            Field { ident, types }
        })
        .collect()
}

#[derive(Default)]
struct StructVisitor {
    structs: HashMap<String, Struct>,
}

impl<'ast> Visit<'ast> for StructVisitor {
    fn visit_item_struct(&mut self, i: &'ast ItemStruct) {
        let ident = i.ident.to_string();

        let mut generate = false;
        let mut return_type = None;
        let mut replacement_type = None;
        if let Some(meta) = get_meta(i) {
            generate = true;

            let params = get_nested_meta_params(&meta);
            return_type = params.get("return_type").map(|v| v.clone());
            replacement_type = params.get("replacement_type").map(|v| v.clone());
        }

        let fields = match &i.fields {
            Fields::Named(named) => extract_fields(&named),
            _ => panic!("only named fields are supported")
        };

        self.structs.insert(
            ident.clone(),
            Struct {
                ident,
                fields,
                generate,
                return_type,
                replacement_type,
            });
    }
}

fn fields_to_path(fields: &[&Field]) -> String {
    fields
        .iter()
        .map(|field| {
            if field.types.contains(&"Vec".to_owned()) {
                format!("{}[0]", field.ident)
            } else if field.types.contains(&"BTreeMap".to_owned()) {
                format!("{}[\"en\"]", field.ident)
            } else {
                field.ident.to_string()
            }
        })
        .collect::<Vec<String>>()
        .join(".")
}

fn fields_to_getter(fields: &[&Field]) -> proc_macro2::TokenStream {
    let mut within_option = false;
    let size = fields.len();

    let mut tokens: Vec<proc_macro2::TokenStream> = Vec::new();

    for (i, field) in fields.iter().enumerate() {
        let ident = Ident::new(&field.ident, Span::call_site());
        let is_last = i == size - 1;

        let str_types: Vec<&str> = field.types.iter().map(|s| s.as_str()).collect();
        let res = match str_types.as_slice() {
            ["Option", "Vec", _] => {
                if !within_option {
                    within_option = true;
                    quote!(.#ident.as_ref().and_then(|x| x.first()))
                } else {
                    quote!(.and_then(|x| x.#ident.as_ref()).and_then(|x| x.first()))
                }
            },
            ["Option", "BTreeMap", _, _] => {
                if !within_option {
                    within_option = true;
                    quote!(.#ident.as_ref().and_then(|x| x.get("en").copied()))
                } else {
                    quote!(.and_then(|x| x.#ident.as_ref()).and_then(|x| x.get("en").copied()))
                }
            },
            ["Option", _] => {
                if !within_option {
                    within_option = true;
                    if is_last {
                        quote!(.#ident)
                    } else {
                        quote!(.#ident.as_ref())
                    }
                } else {
                    if is_last {
                        quote!(.and_then(|x| x.#ident))
                    } else {
                        quote!(.and_then(|x| x.#ident.as_ref()))
                    }
                }
            },
            &[] | &[..] => todo!(),
        };

        tokens.push(res);
    }
    let tokens = proc_macro2::TokenStream::from_iter(tokens);

    let path = fields_to_path(fields);
    let res = quote!(
      #path => self #tokens.into(),
    );

    proc_macro2::TokenStream::from(res)
}

struct GeneratorState<'a> {
    current_struct: &'a Struct,
    fields: Vec<&'a Field>
}

fn generate_getters(target: String, structs: &HashMap<String, Struct>) -> (Vec<String>, Vec<proc_macro2::TokenStream>) {
    let mut paths = Vec::new();
    let mut getters = Vec::new();

    let mut to_visit = VecDeque::new();
    to_visit.push_back(GeneratorState {
        current_struct: structs.get(&target).unwrap(),
        fields: Vec::new()
    });

    while let Some(state) = to_visit.pop_front() {
        for field in state.current_struct.fields.iter() {
            let ty = field.types.last().unwrap();
            let mut fields = state.fields.clone();
            fields.push(field);

            if structs.contains_key(ty) {
                to_visit.push_front(GeneratorState {
                    current_struct: structs.get(ty).unwrap(),
                    fields
                });
            } else {
                paths.push(fields_to_path(&fields));
                getters.push(fields_to_getter(&fields));
            }
        }
    }

    (paths, getters)
}

fn generate_impl(target: String, structs: &HashMap<String, Struct>) -> TokenStream {
    let target_struct = structs.get(&target).unwrap();

    let trgt = if let Some(rpl) = &target_struct.replacement_type {
        Ident::new(rpl, Span::mixed_site())
    } else {
        Ident::new(&target_struct.ident, Span::mixed_site())
    };

    let res_type = Ident::new(&target_struct.return_type.as_ref().unwrap(), Span::mixed_site());

    let (paths, getters) = generate_getters(target, structs);
    let getters = proc_macro2::TokenStream::from_iter(getters);
    let res = quote!(
        impl<'a> struct_deep_getter::StructDeepGetter<#res_type> for #trgt<'a> {
            fn deeper_structs() -> Vec<String> {
                let mut res = Vec::new();
                #(res.push(#paths.to_string());)*
                res
            }

            fn get_path(&self, path: &str) -> #res_type {
                match path {
                    #getters
                    _ => panic!("error"),
                }
            }
        }
    );
    println!("{}", res);
    TokenStream::from(res)
}

#[proc_macro]
pub fn make_paths(input: TokenStream) -> TokenStream {
    let ast: File = syn::parse(input).unwrap();
    let mut state = StructVisitor::default();
    state.visit_file(&ast);

    let mut impls = Vec::new();
    for (ident, strct) in state.structs.iter() {
        println!("{:?}", strct);
        if strct.generate {
            impls.push(generate_impl(ident.clone(), &state.structs));
        }
    }

    TokenStream::from_iter(impls)
}
