use std::path::PathBuf;
use swc_core::ecma::{
    parser::{EsSyntax, Syntax},
    transforms::testing::test_fixture,
    visit::visit_mut_pass,
};
use swc_plugin_transform_async_to_ng_generator::AsyncToNgGeneratorVisitor;

#[testing::fixture("tests/fixture/**/input.js")]
fn fixture(input: PathBuf) {
    let output = input.with_file_name("output.js");
    test_fixture(
        Syntax::Es(EsSyntax::default()),
        &|_| visit_mut_pass(AsyncToNgGeneratorVisitor::new()),
        &input,
        &output,
        Default::default(),
    );
}
