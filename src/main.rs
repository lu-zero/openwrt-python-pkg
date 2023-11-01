use clap::Parser;
use jaq_core::{parse, Ctx, Definitions, Error, RcIter, Val};
use minijinja::{context, Environment};
use serde_json::Value;

static TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/templates/Makefile.jinja"
));

#[derive(Debug, Parser)]
struct Opts {
    package_name: String,
}

fn filter(query: &str, data: &Value) -> Option<Val> {
    let mut errs = Vec::new();
    let mut defs = Definitions::core();
    jaq_std::std()
        .into_iter()
        .for_each(|def| defs.insert(def, &mut errs));
    let f = parse::parse(&query, parse::main()).0.unwrap();
    let f = defs.finish(f, Vec::new(), &mut errs);

    let inputs = RcIter::new(core::iter::empty());
    let mut out = f.run(Ctx::new([], &inputs), Val::from(data.to_owned()));

    out.next().transpose().expect("Parse failure")
}

fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();
    let uri = format!("https://pypi.org/pypi/{}/json", &opts.package_name);
    let pkg: Value = reqwest::blocking::get(uri)?.json()?;
    /*
        let sha256 = filter(
            ".urls[] | select( .packagetype == \"sdist\").digests.sha256",
            &pkg,
        )
        .expect("Cannot find sha256");

        let summary = filter(".info.summary", &pkg);
        let license = filter(".info.license", &pkg);
    */
    //    println!("summary {:?}", summary);
    //    println!("license {:?}", license);
    //    println!("sha256 {:?}", sha256);
    //    println!("deps {:#?}", deps);
    // let deps: Value = filter(".info.requires_dist", &pkg).unwrap().into();

    // for dep in deps.as_array().unwrap() {
    //    println!("{dep:?}");
    //}

    let ctx: Value = filter(
        "{\
        name: .info.name,
        reqs: .info.requires_dist,\
        summary: .info.summary,\
        license: .info.license,\
        sha256: .urls[] | select( .packagetype == \"sdist\").digests.sha256,\
        home_page: .info.home_page,\
        version: .info.version,\
        deps: .info.requires_dist,\
        }",
        &pkg,
    )
    .unwrap()
    .into();

    //    println!("{ctx:?}");

    let mut env = Environment::new();
    env.add_template("Makefile", TEMPLATE).unwrap();
    let t = env.get_template("Makefile").unwrap();

    println!("{}", t.render(ctx).unwrap());

    Ok(())
}
