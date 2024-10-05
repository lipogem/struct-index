# struct-index
structure implement index trait
the order in which the structure is defined is the same

*Examples*

```
use std::{
    any::Any,
    ops::{Index, Not},
};
use struct_index::StructIndex;

#[derive(StructIndex)]
struct Student {
    name: String,
    age: u32,
    height: f64,
    address: String,
}

fn fmt_str<'a, T>(ival: &'a T) -> String
where
    T: Index<usize, Output = dyn Any>,
    &'a T: Not<Output = (&'static str, &'static [&'static str])>,
{
    let (name, fnames) = !ival;
    let mut content = name.to_string() + "::";
    for i in 0..fnames.len() {
        if let Some(v) = ival[i].downcast_ref::<u32>() {
            content += fnames[i];
            content += ":";
            content += &v.to_string();
        } else if let Some(v) = ival[i].downcast_ref::<f64>() {
            content += fnames[i];
            content += ":";
            content += &v.to_string();
        } else if let Some(v) = ival[i].downcast_ref::<String>() {
            content += fnames[i];
            content += ":\"";
            content += &v;
            content += "\"";
        }
        if i < fnames.len() - 1 {
            content += ",";
        }
    }
    content
}

fn main() {
    let student = Student {
        name: "Alice".to_string(),
        age: 18,
        height: 1.72,
        address: "New York".to_string(),
    };
    let content = fmt_str(&student);
    assert_eq!(
        content,
        "Student::name:\"Alice\",age:18,height:1.72,address:\"New York\""
    );
}
```

## License

struct-index is provided under the MIT license. See [LICENSE](LICENSE).
