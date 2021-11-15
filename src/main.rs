use tracing::info;

mod adb;
mod args;
mod command;

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_target(false)
        .without_time()
        .with_max_level(tracing::Level::DEBUG)
        .init();
}

fn main() {
    let start = std::time::Instant::now();
    init_tracing();

    let pkg = args::target_package();
    let so_file = args::local_lib();
    info!("pkg: {:?}, file: {:?}", pkg, so_file);

    let app_lib = adb::app_lib_location(pkg).unwrap();
    let app_so_lib = format!("{}/lib/{}", app_lib, args::app_arch());
    adb::push(so_file, &[&app_so_lib, &format!("/data/data/{}/lib", pkg)]).unwrap();
    info!("done!  cost={:.2?}", start.elapsed());
}
