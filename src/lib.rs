use proc_macro::{Delimiter, TokenStream, TokenTree};

/// generic after clearing constraints
/// <T:Default> => <T>
fn gen_clean(gen: &str) -> String {
    if gen.is_empty() {
        return String::new();
    }
    let mut gen = gen
        .split_terminator(',')
        .map(|g| g.split_terminator(':').nth(0).unwrap())
        .collect::<Vec<_>>()
        .join(",");
    if !gen.contains('>') {
        gen.push('>');
    }
    gen
}

/// structure fields
/// struct student {id:i64, name:String} => [id, name]
fn struct_fields(input: TokenStream) -> Vec<String> {
    let mut fields = Vec::new();
    let mut field = String::new();
    let mut colon = 0;
    for node in input {
        match node {
            TokenTree::Punct(punct) => {
                if punct.as_char() == ':' {
                    colon += 1;
                }
            }
            _ => {
                if colon == 1 {
                    fields.push(field);
                }
                field = node.to_string();
                colon = 0;
            }
        }
    }
    fields
}

/// implement index trait and not trait for structs
/// not trait gets struct and field names
fn struct_ret(name: &str, gen: &str, whr: &str, fields: &[String]) -> TokenStream {
    assert_ne!(name.len(), 0, "structure name not found");
    format!(
        "
        impl {gen} core::ops::Not for &{name} {cgn} {whr} {{
            type Output = (&'static str, &'static [&'static str]);

            /// return (structure name, field names)
            fn not(self) -> Self::Output {{
                (\"{name}\", &[{fnas}])
            }}
        }}

        impl {gen} core::ops::Index<usize> for {name} {cgn} {whr} {{
            type Output = dyn core::any::Any;

            fn index(&self, index: usize) -> &Self::Output {{
                ([{flds}] as [&Self::Output; {len}])[index]
            }}
        }}

        impl {gen} core::ops::IndexMut<usize> for {name} {cgn} {whr} {{
            fn index_mut(&mut self, index: usize) -> &mut Self::Output {{
                ([{fmus}] as [&mut Self::Output; {len}])[index]
            }}
        }}
        ",
        gen = gen,
        name = name,
        cgn = gen_clean(&gen),
        whr = whr,
        len = fields.len(),
        fnas = if fields.len() == 0 {
            "".to_string()
        } else {
            "\"".to_string() + &fields.join("\",\"") + "\""
        },
        flds = fields
            .iter()
            .map(|f| "&self.".to_string() + f)
            .collect::<Vec<_>>()
            .join(","),
        fmus = fields
            .iter()
            .map(|f| "&mut self.".to_string() + f)
            .collect::<Vec<_>>()
            .join(",")
    )
    .parse()
    .unwrap()
}

/// structs implement index
/// the order in which the structure is defined is the same
/// # Example
/// ```no_run
/// use core::{any::{Any, TypeId}, ops::{Index, IndexMut}};
/// use struct_index::StructIndex;
///
/// macro_rules! downcast_copy {
///     ($target:expr, $source:expr $(,$typ:ty)* $(,)?) => {
///         if $target.type_id() == $source.type_id() {
///             match $target.type_id(){
///                 $(t if t == TypeId::of::<$typ>() => {
///                     *$target.downcast_mut::<$typ>().unwrap() = $source.downcast_ref::<$typ>().unwrap().clone();
///                 },)*
///                 _=>()
///             }
///         }
///     };
/// }
///
/// pub fn like_copy(
///     target: &mut impl IndexMut<usize, Output = dyn Any>,
///     target_index: usize,
///     source: &impl Index<usize, Output = dyn Any>,
///     source_index: usize,
///     len: usize,
/// ) {
///     for i in 0..len {
///         downcast_copy!(
///             target[i + target_index],
///             source[i + source_index],
///             bool, i32, u32, i64, u64, f32, f64, String,
///         );
///     }
/// }
///
/// #[derive(Default, StructIndex)]
/// struct Student {
///     id: i64,
///     name: String,
/// }
///
/// fn main() {
///     let student = Student {
///         id: 18,
///         name: "rust".to_string(),
///     };
///     let len = (!&student).1.len();
///     let mut student1 = Student::default();
///     like_copy(&mut student1, 0, &student, 0, len);
///     assert!(student.id == student1.id && student.name == student1.name);
/// }
/// ```
#[proc_macro_derive(StructIndex)]
pub fn struct_index(input: TokenStream) -> TokenStream {
    let mut is_struct = false;
    let mut name = String::new();
    let mut is_gen = false;
    let mut gen = String::new();
    let mut whr = String::new();
    for node in input {
        match node {
            TokenTree::Ident(ident) => {
                let ident = ident.to_string();
                if ident == "struct" {
                    is_struct = true;
                    continue;
                }
                if is_struct {
                    if is_gen {
                        if ident == "where" {
                            whr.push_str("where ");
                        } else if whr.len() > 0 {
                            whr.push_str(&ident);
                        } else {
                            gen.push_str(&ident);
                        }
                    } else {
                        name = ident;
                    }
                }
            }
            TokenTree::Punct(punct) => {
                if is_struct {
                    let punct = punct.as_char();
                    if punct == '<' {
                        is_gen = true;
                    }
                    if is_gen {
                        if whr.len() > 0 {
                            whr.push(punct);
                        } else {
                            gen.push(punct);
                        }
                    }
                }
            }
            TokenTree::Group(group) => {
                if is_struct {
                    match group.delimiter() {
                        Delimiter::Parenthesis => {
                            if is_struct {
                                return struct_ret(
                                    &name,
                                    &gen,
                                    &whr,
                                    &group
                                        .to_string()
                                        .trim_matches(|c| c == '(' || c == ')')
                                        .split_terminator(',')
                                        .enumerate()
                                        .map(|(i, _)| i.to_string())
                                        .collect::<Vec<_>>(),
                                );
                            }
                        }
                        Delimiter::Brace => {
                            if is_struct {
                                return struct_ret(
                                    &name,
                                    &gen,
                                    &whr,
                                    &struct_fields(group.stream()),
                                );
                            }
                        }
                        _ => (),
                    }
                }
            }
            _ => (),
        }
    }
    unreachable!("can only be usual structures")
}
