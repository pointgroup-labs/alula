#![no_main]

use libfuzzer_sys::fuzz_target;
use tests::{make_oracle_prices_different, Input, TestFixture};

fuzz_target!(|input: Input| {
    let fixture = TestFixture::new();
    fixture.e.cost_estimate().budget().reset_unlimited();
    // By default, TestFixture's mock oracle client uses the same
    // prices for all assets
    make_oracle_prices_different(&fixture.e, &fixture.oracle_client);

    for command in &input.commands {
        command.run(&fixture);
        fixture.assert_invariants();
    }
});
