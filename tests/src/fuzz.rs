#![cfg(test)]

use crate::{Input, TestFixture};

#[allow(unused)]
fn test_fuzzed_issue(input: &Input) {
    let test_fixture = TestFixture::new();
    test_fixture.e.cost_estimate().budget().reset_unlimited();

    for command in &input.commands {
        command.run(&test_fixture);
        test_fixture.assert_invariants();
    }
}
