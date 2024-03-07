use rhaki_cw_plus::multi_test::custom_chains::osmosis::build_osmosis_app;

pub fn startup_osmosis() {
    let (app, db) = build_osmosis_app();
}
