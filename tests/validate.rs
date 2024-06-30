use avast::{built_in::{misc::{eq, validate}, str::{alphanumeric, ascii, length}}, Validate, Validity};

#[test]
fn strings() {
    struct Context {
        foo: String,
        bar: String,
    }

    let ctx = Context {
        foo: format!("Hello"),
        bar: format!("World!"),
    };

    fn uppercase<I: AsRef<str>, C>(item: I) -> Validity {
        item.as_ref().chars().all(char::is_uppercase).into()
    }

    #[derive(Validate)]
    #[avast(ctx(Context))]
    struct NestedStrings {
        #[avast(uppercase, alphanumeric, length(3, 4))]
        nested_a: String,
        nested_b: String,
        #[avast(eq(&ctx.foo))]
        nested_c: String,
    }

    #[derive(Validate)]
    #[avast(ctx(Context))]
    struct Strings {
        #[avast(alphanumeric, length(2, 4))]
        a: String,
        #[avast(eq(&self.a))]
        b: String,
        #[avast(eq(&ctx.bar), alphanumeric)]
        c: String,
        #[avast(uppercase, eq(&self.e.nested_b))]
        d: String,
        #[avast(validate(ctx))]
        e: NestedStrings,
    }

    let strings = Strings {
        a: format!("Avast"),
        b: format!("Rust"),
        c: format!("World!"),
        d: format!("UPPER"),
        e: NestedStrings {
            nested_a: format!("$99"),
            nested_b: format!("UPPER"),
            nested_c: format!("Hello"),
        },
    };

    let validity = strings.validate(&ctx);
}