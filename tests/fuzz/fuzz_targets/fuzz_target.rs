#![no_main]

use libfuzzer_sys::fuzz_target;
use tests::{Input, TestMarketFixture, make_oracle_prices_different};

fuzz_target!(|input: Input| {
    let fixture = TestMarketFixture::new();
    fixture.e.cost_estimate().budget().reset_unlimited();
    make_oracle_prices_different(&fixture.e, &fixture.oracle_client);

    for command in &input.commands {
        command.run(&fixture);
        fixture.assert_invariants();
    }
});
